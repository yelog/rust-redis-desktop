use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("网络请求失败: {0}")]
    NetworkError(String),

    #[error("解析失败: {0}")]
    ParseError(String),

    #[error("下载失败: {0}")]
    DownloadError(String),

    #[error("安装失败: {0}")]
    InstallError(String),

    #[error("平台不支持自动更新")]
    PlatformNotSupported,

    #[error("无可用更新")]
    NoUpdateAvailable,

    #[error("用户取消更新")]
    UserCancelled,

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, UpdateError>;
