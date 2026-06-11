use crate::updater::{InstallResult, Result, UpdateError};
use std::path::PathBuf;

pub struct MacOSInstaller;

const RELEASES_URL: &str = "https://github.com/yelog/rust-redis-desktop/releases";

impl MacOSInstaller {
    pub fn install(_update_path: &PathBuf) -> Result<InstallResult> {
        #[cfg(target_os = "macos")]
        {
            match Self::launch_sparkle_update() {
                Ok(()) => Ok(InstallResult::RestartInProgress),
                Err(err) => {
                    tracing::warn!("Failed to start Sparkle updater: {}", err);
                    Ok(InstallResult::OpenExternal(RELEASES_URL.to_string()))
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(InstallResult::OpenExternal(RELEASES_URL.to_string()))
        }
    }

    pub fn check_for_updates() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            Self::launch_sparkle_update().or_else(|err| {
                tracing::warn!("Failed to start Sparkle updater: {}", err);
                open::that(RELEASES_URL).map_err(|e| UpdateError::InstallError(e.to_string()))
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    fn launch_sparkle_update() -> Result<()> {
        use objc2::msg_send;
        use objc2::runtime::{AnyClass, AnyObject};
        use objc2_foundation::{NSBundle, NSString};

        let app_bundle = Self::current_app_bundle()?;
        let sparkle_framework = app_bundle
            .join("Contents")
            .join("Frameworks")
            .join("Sparkle.framework");

        if !sparkle_framework.exists() {
            return Err(UpdateError::InstallError(format!(
                "Sparkle framework not found at {}",
                sparkle_framework.display()
            )));
        }

        let framework_path = sparkle_framework
            .to_str()
            .ok_or_else(|| UpdateError::InstallError("Sparkle framework path is invalid".into()))?;

        let framework_path = NSString::from_str(framework_path);
        let bundle = NSBundle::bundleWithPath(&framework_path).ok_or_else(|| {
            UpdateError::InstallError("Unable to load Sparkle framework bundle".into())
        })?;

        if !unsafe { bundle.load() } {
            return Err(UpdateError::InstallError(
                "Sparkle framework bundle failed to load".into(),
            ));
        }

        let updater_class = AnyClass::get(c"SUUpdater").ok_or_else(|| {
            UpdateError::InstallError("Sparkle SUUpdater class is unavailable".into())
        })?;
        let updater: *mut AnyObject = unsafe { msg_send![updater_class, sharedUpdater] };
        if updater.is_null() {
            return Err(UpdateError::InstallError(
                "Sparkle shared updater is unavailable".into(),
            ));
        }

        let _: () =
            unsafe { msg_send![updater, checkForUpdates: std::ptr::null_mut::<AnyObject>()] };

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn current_app_bundle() -> Result<PathBuf> {
        let executable = std::env::current_exe().map_err(UpdateError::IoError)?;
        let app_bundle = executable
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .ok_or_else(|| UpdateError::InstallError("无法找到应用目录".to_string()))?;

        if app_bundle.extension().and_then(|ext| ext.to_str()) != Some("app") {
            return Err(UpdateError::InstallError(format!(
                "Current executable is not running from an app bundle: {}",
                executable.display()
            )));
        }

        Ok(app_bundle.to_path_buf())
    }
}
