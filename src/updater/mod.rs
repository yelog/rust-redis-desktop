mod checker;
mod config;
mod downloader;
mod error;
mod installer;
mod manager;
mod platform;
mod types;

pub use checker::{get_current_version, UpdateChecker};
pub use config::UpdateConfig;
pub use downloader::{ProgressCallback, UpdateDownloader};
pub use error::{Result, UpdateError};
pub use installer::UpdateInstaller;
pub use manager::{UpdateManager, UpdateState};
pub use platform::*;
pub use types::{InstallResult, Platform, UpdateInfo};

use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

static UPDATE_CHECK_TRIGGER: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Default)]
pub struct UpdateStatus {
    pub pending_update: Option<UpdateInfo>,
    pub checking: bool,
}

pub static UPDATE_STATUS: GlobalSignal<UpdateStatus> = Signal::global(|| UpdateStatus::default());

pub fn set_pending_update(info: Option<UpdateInfo>) {
    *UPDATE_STATUS.write() = UpdateStatus {
        pending_update: info,
        checking: false,
    };
}

pub fn set_checking(checking: bool) {
    let mut status = UPDATE_STATUS.write();
    status.checking = checking;
}

pub fn trigger_manual_check() {
    UPDATE_CHECK_TRIGGER.store(true, Ordering::SeqCst);
}

pub fn should_trigger_manual_check() -> bool {
    UPDATE_CHECK_TRIGGER.swap(false, Ordering::SeqCst)
}
