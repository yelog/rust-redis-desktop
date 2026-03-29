mod checker;
mod config;
mod downloader;
mod error;
mod types;

pub use checker::{get_current_version, UpdateChecker};
pub use config::UpdateConfig;
pub use downloader::{ProgressCallback, UpdateDownloader};
pub use error::{Result, UpdateError};
pub use types::{InstallResult, Platform, UpdateInfo};
