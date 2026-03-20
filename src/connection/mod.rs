mod config;
mod pool;
mod manager;
mod error;

pub use config::*;
pub use pool::*;
pub use manager::*;
pub use error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}