use std::path::{Path};
use futures_util::{stream, StreamExt};
use tokio::fs;
use tokio::sync::mpsc;
use crate::core::client::PistonClient;
use crate::core::download::{DownloadFromApi, Downloadable};
use crate::core::errors::SdkError;
use crate::core::progress::{DownloadEvent};
use crate::loaders::piston::models::version_details::PistonVersionDetails;

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
        let _permit = self
            .limiter
            .acquire()
            .await?;

        let download_info = version.downloads().client();

        let file_suffix = "-client";

        let file_extension = ".jar";

        let mut file_name =
            String::with_capacity(version.id().len() + file_suffix.len() + file_extension.len());

        file_name.push_str(version.id());
        file_name.push_str(file_suffix);
        file_name.push_str(file_extension);

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
        let _permit = self
            .limiter
            .acquire()
            .await?;

        let download_info = version.downloads().client_mappings();

        let Some(client_mappings) = download_info else {
            return Ok(None);
        };

        let file_suffix = "-client_mappings";

        let file_extension = ".txt";

        let mut file_name =
            String::with_capacity(version.id().len() + file_suffix.len() + file_extension.len());

        file_name.push_str(version.id());
        file_name.push_str(file_suffix);
        file_name.push_str(file_extension);

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
        let _permit = self
            .limiter
            .acquire()
            .await?;

        let download_info = version.downloads().server();

        let Some(server) = download_info else {
            return Ok(None);
        };

        let file_suffix = "-server";

        let file_extension = ".jar";

        let mut file_name =
            String::with_capacity(version.id().len() + file_suffix.len() + file_extension.len());

        file_name.push_str(version.id());
        file_name.push_str(file_suffix);
        file_name.push_str(file_extension);

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
        let _permit = self
            .limiter
            .acquire()
            .await?;

        let download_info = version.downloads().server_mappings();

        let Some(server_mappings) = download_info else {
            return Ok(None);
        };

        let file_suffix = "-server_mappings";

        let file_extension = ".txt";

        let mut file_name =
            String::with_capacity(version.id().len() + file_suffix.len() + file_extension.len());

        file_name.push_str(version.id());
        file_name.push_str(file_suffix);
        file_name.push_str(file_extension);

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
        }).await?;

        let base_target_dir = libraries_dir.as_ref().to_path_buf();

        let max_parallel = self.limiter.available_permits();

        let max_parallel = if max_parallel == 0 { 64 } else { max_parallel };

        let download_stream = stream::iter(artifacts).map(|artifact| {
            let limiter = self.limiter.clone();
            let target_dir = base_target_dir.clone();
            let tx = tx.clone();

            async move {
                let _permit = limiter
                    .acquire()
                    .await?;

                let artifact_path = Path::new(&artifact.path);

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

        let mut buffered_stream = download_stream.buffer_unordered(max_parallel);

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
        let max_parallel = self.limiter.available_permits();
        let max_parallel = if max_parallel == 0 { 64 } else { max_parallel };

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

        let client = self.client.clone();

        let download_stream = stream::iter(native_artifacts).map(|artifact| {
            let limiter = self.limiter.clone();
            let natives_dir = natives_dir.clone();
            let tx = tx.clone();
            let client = client.clone();
            let url = artifact.url();

            async move {
                let _permit = limiter.acquire().await?;

                let response = client.get(url.clone()).send().await.map_err(|e| SdkError::Network(e))?;
                let bytes = response.bytes().await.map_err(|e| SdkError::Network(e))?;

                let reader = std::io::Cursor::new(bytes);
                let mut archive = zip::ZipArchive::new(reader).map_err(|e| {
                    SdkError::DownloadError(format!("Failed to read zip archive for native: {}", e))
                })?;

                // 3. Достаем из архива только файлы нативных библиотек (.dll, .so, .dylib)
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).map_err(|e| {
                        SdkError::DownloadError(format!("Failed to read zip entry: {}", e))
                    })?;

                    let file_name = match file.enclosed_name() {
                        Some(path) => match path.file_name() {
                            Some(name) => name.to_owned(),
                            None => continue,
                        },
                        None => continue,
                    };

                    let name_str = file_name.to_string_lossy();

                    if name_str.ends_with(".dll") || name_str.ends_with(".so") || name_str.ends_with(".dylib") {
                        let target_path = natives_dir.join(&file_name);

                        let mut out_file = std::fs::File::create(&target_path)?;
                        std::io::copy(&mut file, &mut out_file)?;
                    }
                }

                let _ = tx.send(DownloadEvent::FileFinished {
                    name: artifact.path.split('/').last().unwrap_or("native").to_string()
                }).await;

                Ok::<(), SdkError>(())
            }
        });

        let mut buffered_stream = download_stream.buffer_unordered(max_parallel);

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

        let file_extension = ".json";
        let mut file_name = String::with_capacity(assets_index.id().len() + file_extension.len());
        file_name.push_str(&assets_index.id());
        file_name.push_str(file_extension);

        Ok(assets_index
            .download_from_api(assets_index_dir, &file_name, tx)
            .await?)
    }

    pub async fn download_asset_objects<P>(
        &self,
        version: &PistonVersionDetails,
        assets_dir: P,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let max_parallel = self.limiter.available_permits();

        let max_parallel = if max_parallel == 0 { 64 } else { max_parallel };

        // Убрать
        let assets_objects = self.fetch_asset_index(version).await?;

        if assets_objects.is_empty() {
            tx.send(DownloadEvent::Finished).await?;
            return Ok(());
        }

        let objects_dir = assets_dir.as_ref().join("objects");

        let download_stream =
            stream::iter(assets_objects.into_iter()).map(|(_obj_name, obj_data)| {
                let limiter = self.limiter.clone();
                let objects_dir = objects_dir.clone();
                let tx = tx.clone();

                async move {
                    let _permit = limiter
                        .acquire()
                        .await?;

                    let file_name = obj_data.hash.clone();
                    let prefix = &obj_data.hash[0..2];
                    let target_sub_dir = objects_dir.join(prefix);

                    obj_data
                        .download_from_api(target_sub_dir, &file_name, tx)
                        .await?;

                    Ok::<(), SdkError>(())
                }
            });

        let mut buffered_stream = download_stream.buffer_unordered(max_parallel);

        while let Some(result) = buffered_stream.next().await {
            result?;
        }

        tx.send(DownloadEvent::Finished).await?;

        Ok(())
    }
}