use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Network request failed: {0}")]
    NetworkError(String),

    #[error("Parse failed: {0}")]
    ParseError(String),

    #[error("Download failed: {0}")]
    DownloadError(String),

    #[error("Install failed: {0}")]
    InstallError(String),

    #[error("Platform does not support auto-update")]
    PlatformNotSupported,

    #[error("No update available")]
    NoUpdateAvailable,

    #[error("Update cancelled by user")]
    UserCancelled,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, UpdateError>;
