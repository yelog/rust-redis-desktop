use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ConnectionError {
    #[error("Failed to connect to Redis: {0}")]
    ConnectionFailed(String),

    #[error("Invalid connection configuration: {0}")]
    InvalidConfig(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Connection closed")]
    Closed,
}

pub type Result<T> = std::result::Result<T, ConnectionError>;
