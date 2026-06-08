use crate::updater::{InstallResult, Result, UpdateError};
use std::path::PathBuf;
use std::process::Command;

pub struct WindowsInstaller;

impl WindowsInstaller {
    pub fn install(installer_path: &PathBuf) -> Result<InstallResult> {
        Command::new(installer_path)
            .arg("/S")
            .spawn()
            .map_err(|e| UpdateError::InstallError(e.to_string()))?;

        Ok(InstallResult::RestartInProgress)
    }
}
