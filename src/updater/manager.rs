use crate::updater::{
    get_current_version, InstallResult, Result, UpdateChecker, UpdateConfig, UpdateDownloader,
    UpdateInfo, UpdateInstaller,
};
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct UpdateManager {
    checker: UpdateChecker,
    downloader: UpdateDownloader,
    config: UpdateConfig,
}

#[derive(Debug, Clone)]
pub enum UpdateState {
    Idle,
    Checking,
    Available(UpdateInfo),
    Downloading(u64, u64),
    ReadyToInstall(PathBuf),
    Installing,
    Completed,
    Error(String),
}

impl UpdateManager {
    pub fn new() -> Result<Self> {
        let current_version = get_current_version();
        let checker = UpdateChecker::new(&current_version);
        let downloader = UpdateDownloader::new()?;
        let config = UpdateConfig::load()?;

        Ok(Self {
            checker,
            downloader,
            config,
        })
    }

    pub async fn check_for_updates(&mut self) -> Result<Option<UpdateInfo>> {
        self.config.mark_checked();
        self.config.save()?;

        let result = self.checker.check().await?;

        if let Some(ref info) = result {
            if self.config.is_skipped(&info.version) {
                return Ok(None);
            }
        }

        Ok(result)
    }

    pub fn should_auto_check(&self) -> bool {
        self.config.should_check()
    }

    pub async fn download_update(
        &mut self,
        info: &UpdateInfo,
        progress_tx: Option<mpsc::Sender<(u64, u64)>>,
    ) -> Result<PathBuf> {
        let path = self
            .downloader
            .download(&info.download_url, &info.asset_name, progress_tx)
            .await?;

        Ok(path)
    }

    pub fn install_update(&self, update_path: &PathBuf) -> Result<InstallResult> {
        UpdateInstaller::install(update_path)
    }

    pub fn skip_version(&mut self, version: &str) -> Result<()> {
        self.config.skip_version(version);
        self.config.save()?;
        Ok(())
    }
}
