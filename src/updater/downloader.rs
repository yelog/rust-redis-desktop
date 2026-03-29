use crate::updater::{Result, UpdateError};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct UpdateDownloader {
    temp_dir: PathBuf,
}

pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send>;

impl UpdateDownloader {
    pub fn new() -> Result<Self> {
        let temp_dir = dirs::cache_dir()
            .ok_or_else(|| {
                UpdateError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "无法找到缓存目录",
                ))
            })?
            .join("rust-redis-desktop")
            .join("updates");

        std::fs::create_dir_all(&temp_dir).map_err(UpdateError::IoError)?;

        Ok(Self { temp_dir })
    }

    pub async fn download(
        &self,
        url: &str,
        asset_name: &str,
        progress: Option<mpsc::Sender<(u64, u64)>>,
    ) -> Result<PathBuf> {
        let client = reqwest::Client::builder()
            .user_agent("rust-redis-desktop")
            .build()
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpdateError::DownloadError(format!(
                "下载失败: HTTP {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let file_path = self.temp_dir.join(asset_name);

        let mut file = File::create(&file_path).map_err(UpdateError::IoError)?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| UpdateError::DownloadError(e.to_string()))?;

            file.write_all(&chunk).map_err(UpdateError::IoError)?;

            downloaded += chunk.len() as u64;

            if let Some(ref sender) = progress {
                let _ = sender.try_send((downloaded, total_size));
            }
        }

        Ok(file_path)
    }

    pub fn calculate_sha256(&self, file_path: &PathBuf) -> Result<String> {
        let mut file = File::open(file_path).map_err(UpdateError::IoError)?;
        let mut hasher = Sha256::new();

        std::io::copy(&mut file, &mut hasher).map_err(UpdateError::IoError)?;

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    pub fn cleanup(&self) -> Result<()> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir).map_err(UpdateError::IoError)?;
        }
        Ok(())
    }

    pub fn temp_dir(&self) -> &PathBuf {
        &self.temp_dir
    }
}

impl Default for UpdateDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create downloader")
    }
}