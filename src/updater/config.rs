use crate::updater::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub last_check: Option<DateTime<Utc>>,
    pub skip_version: Option<String>,
    pub check_interval_hours: u64,
    pub check_beta: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            last_check: None,
            skip_version: None,
            check_interval_hours: 24,
            check_beta: false,
        }
    }
}

impl UpdateConfig {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "无法找到配置目录"))
            .map_err(crate::updater::UpdateError::IoError)?;

        Ok(config_dir.join("rust-redis-desktop").join("update.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            std::fs::read_to_string(&path).map_err(crate::updater::UpdateError::IoError)?;

        let config: Self = toml::from_str(&content)
            .map_err(|e| crate::updater::UpdateError::ParseError(e.to_string()))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        let parent = path.parent().ok_or_else(|| {
            crate::updater::UpdateError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "无法找到父目录",
            ))
        })?;

        std::fs::create_dir_all(parent).map_err(crate::updater::UpdateError::IoError)?;

        let content = toml::to_string_pretty(&self)
            .map_err(|e| crate::updater::UpdateError::ParseError(e.to_string()))?;

        std::fs::write(&path, content).map_err(crate::updater::UpdateError::IoError)?;

        Ok(())
    }

    pub fn should_check(&self) -> bool {
        if let Some(last) = self.last_check {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last);
            elapsed.num_hours() >= self.check_interval_hours as i64
        } else {
            true
        }
    }

    pub fn mark_checked(&mut self) {
        self.last_check = Some(Utc::now());
    }

    pub fn skip_version(&mut self, version: &str) {
        self.skip_version = Some(version.to_string());
    }

    pub fn is_skipped(&self, version: &str) -> bool {
        self.skip_version.as_ref() == Some(&version.to_string())
    }
}
