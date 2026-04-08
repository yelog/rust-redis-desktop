use std::path::PathBuf;

pub(crate) mod linux;
pub(crate) mod macos;
pub(crate) mod windows;

#[cfg(target_os = "macos")]
const APP_ID: &str = "dev.yelog.rust-redis-desktop";
#[cfg(target_os = "linux")]
const APP_NAME: &str = "Rust Redis Desktop";
#[cfg(target_os = "linux")]
const DESKTOP_ENTRY_NAME: &str = "rust-redis-desktop.desktop";
#[cfg(target_os = "windows")]
const WINDOWS_RUN_VALUE_NAME: &str = "Rust Redis Desktop";

pub struct AutostartManager;

impl AutostartManager {
    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        platform::set_enabled(enabled)
    }
}

fn current_executable() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|err| format!("Failed to determine current executable: {err}"))
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{current_executable, macos, APP_ID};

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let executable = current_executable()?;
        macos::set_enabled(APP_ID, &executable, enabled)
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::{current_executable, windows, WINDOWS_RUN_VALUE_NAME};

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let executable = current_executable()?;
        windows::set_enabled(WINDOWS_RUN_VALUE_NAME, &executable, enabled)
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{current_executable, linux, APP_NAME, DESKTOP_ENTRY_NAME};

    pub fn set_enabled(enabled: bool) -> Result<(), String> {
        let executable = current_executable()?;
        linux::set_enabled(APP_NAME, DESKTOP_ENTRY_NAME, &executable, enabled)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod platform {
    pub fn set_enabled(_enabled: bool) -> Result<(), String> {
        Err("Autostart is not supported on this platform".to_string())
    }
}
