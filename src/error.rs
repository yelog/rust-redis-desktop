use thiserror::Error;

/// 应用顶层错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Startup failed: {0}")]
    Startup(#[from] StartupError),

    #[error("Connection error: {0}")]
    Connection(#[from] crate::connection::ConnectionError),

    #[error("Update error: {0}")]
    Update(#[from] crate::updater::UpdateError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("{0}")]
    Other(String),
}

/// 启动错误（核心功能，失败需要退出）
#[derive(Error, Debug)]
pub enum StartupError {
    #[error("Failed to create menu: {source}")]
    MenuCreation {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to initialize runtime: {source}")]
    RuntimeInit {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to create window: {source}")]
    WindowCreation {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// 配置错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to access config directory: {0}")]
    DirectoryAccess(String),

    #[error("Failed to read config file: {0}")]
    ReadError(String),

    #[error("Failed to write config file: {0}")]
    WriteError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// 全局Result类型别名
pub type Result<T> = std::result::Result<T, AppError>;
