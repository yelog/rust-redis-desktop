use crate::error::Result;

#[cfg(target_os = "windows")]
use crate::error::{AppError, StartupError};

pub const WEBVIEW2_DOWNLOAD_URL: &str =
    "https://developer.microsoft.com/en-us/microsoft-edge/webview2/";

pub(crate) fn webview2_runtime_error_message(details: &str) -> String {
    format!(
        "Microsoft Edge WebView2 Runtime is required to start Rust Redis Desktop on Windows.\nWindows 启动 Rust Redis Desktop 需要 Microsoft Edge WebView2 Runtime。\n\nInstall it from / 下载地址:\n{WEBVIEW2_DOWNLOAD_URL}\n\nTechnical details / 技术细节:\n{details}"
    )
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

#[cfg(not(target_os = "windows"))]
pub fn ensure_windows_webview_runtime() -> Result<()> {
    Ok(())
}
