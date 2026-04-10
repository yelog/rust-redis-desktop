#[cfg(test)]
mod tests {
    use crate::startup::{
        ensure_webview2_data_directory, webview2_data_directory, webview2_runtime_error_message,
        WEBVIEW2_DOWNLOAD_URL,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn test_webview2_runtime_error_message_contains_download_url() {
        let message = webview2_runtime_error_message("WebView2 unavailable");

        assert!(message.contains(WEBVIEW2_DOWNLOAD_URL));
        assert!(message.contains("WebView2 Runtime is required"));
    }

    #[test]
    fn test_webview2_runtime_error_message_contains_technical_details() {
        let details = "GetAvailableCoreWebView2BrowserVersionString failed";
        let message = webview2_runtime_error_message(details);

        assert!(message.contains("Technical details / 技术细节:"));
        assert!(message.contains(details));
    }

    #[test]
    fn test_webview2_data_directory_uses_expected_subdirectories() {
        let base_dir = Path::new("C:\\Users\\demo\\AppData\\Local");
        let expected = PathBuf::from("C:\\Users\\demo\\AppData\\Local")
            .join("rust-redis-desktop")
            .join("webview2");

        assert_eq!(webview2_data_directory(base_dir), expected);
    }

    #[test]
    fn test_ensure_webview2_data_directory_creates_directory() {
        let unique = format!(
            "rust-redis-desktop-startup-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique).join("webview2");

        ensure_webview2_data_directory(&dir).expect("should create missing webview2 data dir");
        assert!(dir.is_dir());

        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }
}
