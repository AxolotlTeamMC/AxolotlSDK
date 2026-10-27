use async_trait::async_trait;
use futures_util::StreamExt;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use url::Url;
use crate::core::errors::SdkError;
use crate::core::progress::DownloadEvent;

#[async_trait]
pub trait DownloadFromApi {
    async fn download_from_api<P>(
        &self,
        target_dir: P,
        file_name: &str,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + 'static;
}

#[async_trait]
impl<T: Downloadable + ?Sized> DownloadFromApi for T {
    async fn download_from_api<P>(
        &self,
        target_dir: P,
        file_name: &str,
        tx: mpsc::Sender<DownloadEvent>,
    ) -> Result<(), SdkError>
    where
        P: AsRef<Path> + Send + 'static,
    {
        let dir: PathBuf = target_dir.as_ref().to_path_buf();

        if !dir.exists() {
            fs::create_dir_all(&dir).await?;
        }

        let output_path = dir.join(file_name);

        if tokio::fs::metadata(&output_path).await.is_ok() {
            let mut file = File::open(&output_path).await?;
            let mut hasher = Sha1::new();
            let mut buffer = vec![0; 8192];

            loop {
                let bytes_read = file.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }

            let existing_sha1 = hex::encode(hasher.finalize());

            if existing_sha1 == self.sha1() {
                tx.send(DownloadEvent::FileFinished {
                    name: file_name.to_string(),
                })
                    .await?;
                return Ok(());
            } else {
                let _ = fs::remove_file(&output_path).await;
            }
        }

        tx.send(DownloadEvent::FileStarted {
            name: file_name.to_string(),
            size: self.size(),
        })
            .await?;

        let client = reqwest::Client::new();
        let mut attempts = 0;

        loop {
            attempts += 1;

            let response = match client.get(self.url().as_str()).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(resp) => resp,
                    Err(e) => {
                        if attempts >= 3 {
                            let err_msg = e.to_string();
                            tx.send(DownloadEvent::FileFailed {
                                name: file_name.to_string(),
                                error: err_msg.clone(),
                            })
                                .await?;
                            return Err(e.into());
                        }
                        continue;
                    }
                },
                Err(e) => {
                    if attempts >= 3 {
                        let err_msg = e.to_string();
                        tx.send(DownloadEvent::FileFailed {
                            name: file_name.to_string(),
                            error: err_msg.clone(),
                        })
                            .await?;
                        return Err(e.into());
                    }
                    continue;
                }
            };

            let mut file = File::create(&output_path).await?;
            let mut hasher = Sha1::new();
            let mut stream = response.bytes_stream();
            let mut download_failed = false;

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        download_failed = true;
                        let err_msg = e.to_string();
                        tx.send(DownloadEvent::FileFailed {
                            name: file_name.to_string(),
                            error: err_msg.clone(),
                        })
                            .await?;
                        break;
                    }
                };

                if let Err(write_err) = file.write_all(&chunk).await {
                    download_failed = true;
                    tx.send(DownloadEvent::FileFailed {
                        name: file_name.to_string(),
                        error: write_err.to_string(),
                    })
                        .await?;
                    break;
                }

                hasher.update(&chunk);

                tx.send(DownloadEvent::FileChunk {
                    name: file_name.to_string(),
                    bytes_written: chunk.len(),
                })
                    .await?;
            }

            if download_failed {
                let _ = fs::remove_file(&output_path).await;
                if attempts >= 3 {
                    return Err(SdkError::MaxAttemptsExceeded(file_name.to_string()));
                }
                continue;
            }

            file.flush().await?;

            let downloaded_sha1 = hex::encode(hasher.finalize());
            if downloaded_sha1 == self.sha1() {
                tx.send(DownloadEvent::FileFinished {
                    name: file_name.to_string(),
                })
                    .await?;
                break;
            }

            let _ = fs::remove_file(&output_path).await;

            if attempts >= 3 {
                let err_msg = format!(
                    "Файл {} поврежден после 3 попыток скачивания. Хэш не совпадает.",
                    file_name
                );
                tx.send(DownloadEvent::FileFailed {
                    name: file_name.to_string(),
                    error: err_msg.clone(),
                })
                    .await?;
                return Err(SdkError::DownloadError(err_msg));
            }
        }

        Ok(())
    }
}

pub trait Downloadable: Send + Sync {
    fn url(&self) -> Url;
    fn sha1(&self) -> &str;
    fn size(&self) -> u64;
}