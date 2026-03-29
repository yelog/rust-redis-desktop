use crate::updater::platform::*;
use crate::updater::{InstallResult, Result, UpdateInfo};
use std::path::PathBuf;

pub struct UpdateInstaller;

impl UpdateInstaller {
    pub fn install(update_path: &PathBuf) -> Result<InstallResult> {
        #[cfg(target_os = "windows")]
        {
            WindowsInstaller::install(update_path)
        }

        #[cfg(target_os = "macos")]
        {
            MacOSInstaller::install(update_path)
        }

        #[cfg(target_os = "linux")]
        {
            LinuxInstaller::install(update_path)
        }
    }

    pub fn get_download_url(info: &UpdateInfo) -> String {
        info.download_url.clone()
    }
}
