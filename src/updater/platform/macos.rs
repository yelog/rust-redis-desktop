use crate::updater::{InstallResult, Result, UpdateError};
use std::path::PathBuf;

pub struct MacOSInstaller;

impl MacOSInstaller {
    pub fn install(_: &PathBuf) -> Result<InstallResult> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;

            let app_path = std::env::current_exe().map_err(UpdateError::IoError)?;

            let app_dir = app_path
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .ok_or_else(|| UpdateError::InstallError("无法找到应用目录".to_string()))?;

            let autoupdate_path = app_dir
                .join("Frameworks")
                .join("Sparkle.framework")
                .join("Versions")
                .join("A")
                .join("Resources")
                .join("Autoupdate.app")
                .join("Contents")
                .join("MacOS")
                .join("Autoupdate");

            if autoupdate_path.exists() {
                Command::new(&autoupdate_path)
                    .arg(&app_dir)
                    .spawn()
                    .map_err(|e| UpdateError::InstallError(e.to_string()))?;

                Ok(InstallResult::RestartInProgress)
            } else {
                Ok(InstallResult::OpenExternal(
                    "https://github.com/yelog/rust-redis-desktop/releases".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(InstallResult::OpenExternal(
                "https://github.com/yelog/rust-redis-desktop/releases".to_string(),
            ))
        }
    }

    pub fn check_for_updates() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            use open::that as open_url;
            use std::process::Command;

            let app_path = std::env::current_exe().map_err(UpdateError::IoError)?;

            let app_dir = app_path
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .ok_or_else(|| UpdateError::InstallError("无法找到应用目录".to_string()))?;

            let autoupdate_path = app_dir
                .join("Frameworks")
                .join("Sparkle.framework")
                .join("Versions")
                .join("A")
                .join("Resources")
                .join("Autoupdate.app")
                .join("Contents")
                .join("MacOS")
                .join("Autoupdate");

            if autoupdate_path.exists() {
                Command::new(&autoupdate_path)
                    .arg(&app_dir)
                    .spawn()
                    .map_err(|e| UpdateError::InstallError(e.to_string()))?;
            } else {
                open_url("https://github.com/yelog/rust-redis-desktop/releases")
                    .map_err(|e| UpdateError::InstallError(e.to_string()))?;
            }

            Ok(())
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }
}
