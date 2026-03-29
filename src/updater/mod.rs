mod checker;
mod error;
mod types;

pub use checker::{get_current_version, UpdateChecker};
pub use error::{Result, UpdateError};
pub use types::{InstallResult, Platform, UpdateInfo};
