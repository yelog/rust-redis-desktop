#[cfg(test)]
mod tests {
    use crate::startup::{webview2_runtime_error_message, WEBVIEW2_DOWNLOAD_URL};

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

        assert!(message.contains("Technical details:"));
        assert!(message.contains(details));
    }
}
