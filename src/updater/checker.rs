use crate::updater::{Platform, Result, UpdateError, UpdateInfo};
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;

const UPDATE_MANIFEST_URL: &str = "https://yelog.github.io/rust-redis-desktop/update.json";

#[derive(Debug, Clone)]
pub struct UpdateChecker {
    pub current_version: String,
    pub check_beta: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateManifest {
    channels: HashMap<String, ManifestChannel>,
}

#[derive(Debug, Deserialize)]
struct ManifestChannel {
    version: String,
    published_at: String,
    #[serde(default)]
    release_notes: String,
    platforms: HashMap<String, ManifestPlatformAsset>,
}

#[derive(Debug, Deserialize)]
struct ManifestPlatformAsset {
    url: String,
    asset_name: String,
}

impl UpdateChecker {
    pub fn new(current_version: &str) -> Self {
        let is_beta = current_version.contains("-beta");
        Self {
            current_version: current_version.to_string(),
            check_beta: is_beta,
        }
    }

    pub async fn check(&self) -> Result<Option<UpdateInfo>> {
        let client = reqwest::Client::builder()
            .user_agent("rust-redis-desktop")
            .build()
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let response = client
            .get(UPDATE_MANIFEST_URL)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpdateError::NetworkError(format!(
                "Update manifest returned error: {}",
                response.status()
            )));
        }

        let manifest: UpdateManifest = response
            .json()
            .await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        self.update_from_manifest(&manifest)
    }

    fn update_from_manifest(&self, manifest: &UpdateManifest) -> Result<Option<UpdateInfo>> {
        let channel_name = self.channel_name();
        let Some(channel) = manifest.channels.get(channel_name) else {
            return Ok(None);
        };

        if !self.is_newer_version(&channel.version)? {
            return Ok(None);
        }

        let platform = channel
            .platforms
            .get(Platform::current().manifest_key())
            .ok_or(UpdateError::PlatformNotSupported)?;

        Ok(Some(UpdateInfo {
            version: channel.version.clone(),
            release_notes: channel.release_notes.clone(),
            download_url: platform.url.clone(),
            asset_name: platform.asset_name.clone(),
            is_beta: self.check_beta,
            published_at: channel.published_at.clone(),
        }))
    }

    fn is_newer_version(&self, new_version: &str) -> Result<bool> {
        let current = Version::parse(&self.clean_version(&self.current_version))
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        let new = Version::parse(&self.clean_version(new_version))
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        Ok(new > current)
    }

    pub fn clean_version(&self, version: &str) -> String {
        version.split('-').next().unwrap_or(version).to_string()
    }

    fn channel_name(&self) -> &'static str {
        if self.check_beta {
            "beta"
        } else {
            "stable"
        }
    }

    #[cfg(test)]
    pub(crate) fn parse_manifest_json(&self, manifest_json: &str) -> Result<Option<UpdateInfo>> {
        let manifest: UpdateManifest = serde_json::from_str(manifest_json)
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        self.update_from_manifest(&manifest)
    }
}

pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
