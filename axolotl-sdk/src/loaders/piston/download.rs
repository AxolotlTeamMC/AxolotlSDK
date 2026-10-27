use std::collections::HashMap;
use crate::core::client::PistonClient;
use crate::core::download::{DownloadFromApi, Downloadable};
use crate::core::errors::SdkError;
use crate::core::progress::DownloadEvent;
use crate::loaders::piston::models::version_details::PistonVersionDetails;
use futures_util::{StreamExt, stream};
use std::path::Path;
use tokio::fs;
use tokio::sync::mpsc;
use crate::loaders::piston::models::assets::AssetObject;

impl PistonClient {
    pub async fn download_client<P>(
        &self,
        version: &PistonVersionDetails,
        minecraft_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let _permit = self.limiter.acquire().await?;

        let download_info = version.downloads().client();

        let file_name = version.id().to_string() + "-client.jar";

        Ok(download_info
            .download_from_api(minecraft_dir, &file_name, tx)
            .await?)
    }

    pub async fn download_client_mappings<P>(
        &self,
        version: &PistonVersionDetails,
        minecraft_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<Option<()>, SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let _permit = self.limiter.acquire().await?;

        let download_info = version.downloads().client_mappings();

        let Some(client_mappings) = download_info else {
            return Ok(None);
        };

        let file_name = version.id().to_string() + "-client_mappings.txt";

        Ok(Some(
            client_mappings
                .download_from_api(minecraft_dir, &file_name, tx)
                .await?,
        ))
    }

    pub async fn download_server<P>(
        &self,
        version: &PistonVersionDetails,
        minecraft_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<Option<()>, SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let _permit = self.limiter.acquire().await?;

        let download_info = version.downloads().server();

        let Some(server) = download_info else {
            return Ok(None);
        };

        let file_name = version.id().to_string() + "-server.jar";

        Ok(Some(
            server
                .download_from_api(minecraft_dir, &file_name, tx)
                .await?,
        ))
    }

    pub async fn download_server_mappings<P>(
        &self,
        version: &PistonVersionDetails,
        minecraft_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<Option<()>, SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let _permit = self.limiter.acquire().await?;

        let download_info = version.downloads().server_mappings();

        let Some(server_mappings) = download_info else {
            return Ok(None);
        };

        let file_name = version.id().to_string() + "-server_mappings.txt";

        Ok(Some(
            server_mappings
                .download_from_api(minecraft_dir, &file_name, tx)
                .await?,
        ))
    }

    pub async fn download_libraries<P>(
        &self,
        version: &PistonVersionDetails,
        libraries_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let libraries = version.libraries();

        let artifacts = libraries
            .iter()
            .filter(|lib| lib.should_download())
            .filter_map(|lib| lib.downloads().artifact())
            .collect::<Vec<_>>();

        if artifacts.is_empty() {
            tx.send(DownloadEvent::Finished).await?;
            return Ok(());
        }

        tx.send(DownloadEvent::Started {
            total_files: artifacts.len(),
        })
        .await?;

        let base_target_dir = libraries_dir.as_ref().to_path_buf();

        let download_stream = stream::iter(artifacts).map(|artifact| {
            let limiter = self.limiter.clone();
            let target_dir = base_target_dir.clone();
            let tx = tx.clone();

            async move {
                let _permit = limiter.acquire().await?;

                let artifact_path = Path::new(artifact.path());

                let Some(os_file_name) = artifact_path.file_name() else {
                    return Err(SdkError::InvalidPath);
                };
                let file_name = os_file_name.to_string_lossy();

                let parent_dir = artifact_path.parent().unwrap_or(Path::new(""));
                let full_target_dir = target_dir.join(parent_dir);

                artifact
                    .download_from_api(full_target_dir, &file_name, tx)
                    .await?;

                Ok::<(), SdkError>(())
            }
        });

        let mut buffered_stream = download_stream.buffer_unordered(32);

        while let Some(result) = buffered_stream.next().await {
            result?;
        }

        tx.send(DownloadEvent::Finished).await?;

        Ok(())
    }

    pub async fn download_and_extract_natives<P>(
        &self,
        version: &PistonVersionDetails,
        natives_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let natives_dir = natives_dir.as_ref().to_path_buf();

        if !natives_dir.exists() {
            fs::create_dir_all(&natives_dir).await?;
        }

        let native_artifacts: Vec<_> = version
            .libraries()
            .iter()
            .filter_map(|lib| {
                if !lib.should_download() {
                    return None;
                }

                let classifier_key = lib.natives().as_ref()?.get_current_classifier()?;
                let artifact = lib.downloads().classifiers()?.get(classifier_key)?;

                Some(artifact)
            })
            .collect();

        if native_artifacts.is_empty() {
            let _ = tx.send(DownloadEvent::Finished).await;
            return Ok(());
        }

        let _ = tx.send(DownloadEvent::Started { total_files: native_artifacts.len() }).await;

        let client = self.client.clone();

        let download_stream = stream::iter(native_artifacts).map(|artifact| {
            let limiter = self.limiter.clone();
            let natives_dir = natives_dir.clone();
            let tx = tx.clone();
            let client = client.clone();
            let url = artifact.url();

            async move {
                let _permit = limiter.acquire().await?;

                let response = client.get(url).send().await?;
                let bytes = response.bytes().await?;

                tokio::task::spawn_blocking(move || {
                    let reader = std::io::Cursor::new(bytes);
                    let mut archive = zip::ZipArchive::new(reader).map_err(|e| SdkError::ExtractionError(e.to_string()))?;

                    // 3. Достаем из архива только файлы нативных библиотек (.dll, .so, .dylib)
                    for i in 0..archive.len() {
                        let mut file = archive.by_index(i).map_err(|e| {
                            SdkError::DownloadError(format!("Failed to read zip entry: {}", e))
                        })?;

                        let enclosed_path = match file.enclosed_name() {
                            Some(path) => path.to_owned(),
                            None => continue,
                        };

                        let name_str = enclosed_path.to_string_lossy();

                        if name_str.ends_with(".dll")
                            || name_str.ends_with(".so")
                            || name_str.ends_with(".dylib")
                        {
                            let target_path = natives_dir.join(&enclosed_path);

                            if let Some(parent) = target_path.parent() {
                                std::fs::create_dir_all(parent).map_err(SdkError::Io)?;
                            }

                            let mut out_file = std::fs::File::create(&target_path).map_err(SdkError::Io)?;
                            std::io::copy(&mut file, &mut out_file).map_err(SdkError::Io)?;
                        }
                    }
                    Ok::<(), SdkError>(())
                })
                .await
                .map_err(|e| SdkError::DownloadError(e.to_string()))??;

                let file_name = artifact
                    .path()
                    .split('/')
                    .last()
                    .unwrap_or("native")
                    .to_string();

                let _ = tx.send(DownloadEvent::FileFinished { name: file_name }).await;

                Ok::<(), SdkError>(())
            }
        });

        let mut buffered_stream = download_stream.buffer_unordered(32);

        while let Some(result) = buffered_stream.next().await {
            result?;
        }

        tx.send(DownloadEvent::Finished).await?;

        Ok(())
    }

    pub async fn download_asset_index<P>(
        &self,
        version: &PistonVersionDetails,
        assets_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let assets_index = version.asset_index();

        let assets_index_dir = assets_dir.as_ref().join("indexes");

        let file_name = assets_index.id().to_string() + ".json";

        Ok(assets_index
            .download_from_api(assets_index_dir, &file_name, tx)
            .await?)
    }

    pub async fn download_asset_objects<P>(
        &self,
        assets_objects: HashMap<String, AssetObject>,
        assets_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        if assets_objects.is_empty() {
            tx.send(DownloadEvent::Finished).await?;
            return Ok(());
        }

        let _ = tx.send(DownloadEvent::Started { total_files: assets_objects.len() }).await;

        let objects_dir = assets_dir.as_ref().join("objects");

        let download_stream =
            stream::iter(assets_objects.into_iter()).map(|(_obj_name, obj_data)| {
                let limiter = self.limiter.clone();
                let objects_dir = objects_dir.clone();
                let tx = tx.clone();

                async move {
                    let _permit = limiter.acquire().await?;

                    let file_name = obj_data.hash();
                    let prefix = obj_data.hash().get(0..2).unwrap_or("xx");
                    let target_sub_dir = objects_dir.join(prefix);

                    obj_data
                        .download_from_api(target_sub_dir, &file_name, tx)
                        .await?;

                    Ok::<(), SdkError>(())
                }
            });

        let mut buffered_stream = download_stream.buffer_unordered(32);

        while let Some(result) = buffered_stream.next().await {
            result?;
        }

        tx.send(DownloadEvent::Finished).await?;

        Ok(())
    }
}
