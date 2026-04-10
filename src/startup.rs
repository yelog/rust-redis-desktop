use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, StartupError};

pub const WEBVIEW2_DOWNLOAD_URL: &str =
    "https://developer.microsoft.com/en-us/microsoft-edge/webview2/";

pub(crate) fn webview2_runtime_error_message(details: &str) -> String {
    format!(
        "Microsoft Edge WebView2 Runtime is required to start Rust Redis Desktop on Windows.\nWindows 启动 Rust Redis Desktop 需要 Microsoft Edge WebView2 Runtime。\n\nInstall it from / 下载地址:\n{WEBVIEW2_DOWNLOAD_URL}\n\nTechnical details / 技术细节:\n{details}"
    )
}

pub(crate) fn webview2_data_directory(base_dir: &Path) -> PathBuf {
    base_dir.join("rust-redis-desktop").join("webview2")
}

pub(crate) fn ensure_webview2_data_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|error| {
        AppError::Startup(StartupError::WebViewDataDirectoryUnavailable(format!(
            "Failed to prepare a writable Microsoft Edge WebView2 data directory.\nWindows 无法准备可写的 Microsoft Edge WebView2 数据目录。\n\nExpected writable path / 期望可写目录:\n{}\n\nTechnical details / 技术细节:\n{}",
            path.display(),
            error
        )))
    })?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn ensure_windows_webview_runtime() -> Result<()> {
    dioxus::desktop::wry::webview_version().map_err(|error| {
        AppError::Startup(StartupError::WebViewRuntimeUnavailable(
            webview2_runtime_error_message(&error.to_string()),
        ))
    })?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn prepare_windows_webview_data_directory() -> Result<PathBuf> {
    let base_dir = dirs::data_local_dir().ok_or_else(|| {
        AppError::Startup(StartupError::WebViewDataDirectoryUnavailable(
            "Failed to determine the Windows local app data directory.\nWindows 无法确定本地应用数据目录。".to_string(),
        ))
    })?;
    let data_dir = webview2_data_directory(&base_dir);
    ensure_webview2_data_directory(&data_dir)?;
    Ok(data_dir)
}

#[cfg(not(target_os = "windows"))]
pub fn prepare_windows_webview_data_directory() -> Result<PathBuf> {
    Ok(PathBuf::new())
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_windows_webview_runtime() -> Result<()> {
    Ok(())
}
