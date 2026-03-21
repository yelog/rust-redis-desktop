mod config;
mod error;
mod manager;
mod pool;

pub use config::*;
pub use error::*;
pub use manager::*;
pub use pool::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}
