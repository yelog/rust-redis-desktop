use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub release_notes: String,
    pub download_url: String,
    pub asset_name: String,
    pub is_beta: bool,
    pub published_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOSX86,
    MacOSArm,
    Windows,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "x86_64")]
            return Platform::MacOSX86;
            #[cfg(target_arch = "aarch64")]
            return Platform::MacOSArm;
        }
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        #[cfg(target_os = "linux")]
        return Platform::Linux;
    }

    pub fn asset_suffix(&self) -> &'static str {
        match self {
            Platform::MacOSX86 => "x86_64.dmg",
            Platform::MacOSArm => "aarch64.dmg",
            Platform::Windows => "x86_64-windows.zip",
            Platform::Linux => "x86_64.AppImage",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallResult {
    RestartRequired,
    RestartInProgress,
    OpenExternal(String),
}
