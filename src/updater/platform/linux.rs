use crate::updater::{InstallResult, Result};
use std::path::PathBuf;

pub struct LinuxInstaller;

impl LinuxInstaller {
    pub fn install(_: &PathBuf) -> Result<InstallResult> {
        Ok(InstallResult::OpenExternal(
            "https://github.com/yelog/rust-redis-desktop/releases".to_string(),
        ))
    }
}
