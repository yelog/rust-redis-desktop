use crate::updater::{InstallResult, Result};
use std::path::PathBuf;

pub struct LinuxInstaller;

impl LinuxInstaller {
    pub fn install(update_path: &PathBuf) -> Result<InstallResult> {
        Ok(InstallResult::OpenExternal(
            update_path.to_string_lossy().into_owned(),
        ))
    }
}
