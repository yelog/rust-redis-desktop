mod config;
mod error;
mod manager;
mod pool;
mod ssh_tunnel;

pub use config::*;
pub use error::*;
pub use manager::*;
pub use pool::*;
pub use ssh_tunnel::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}
