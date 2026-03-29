use crate::updater::{Platform, Result, UpdateError, UpdateInfo};
use semver::Version;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct UpdateChecker {
    repo: String,
    pub current_version: String,
    pub check_beta: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: String,
    assets: Vec<GitHubAsset>,
    prerelease: bool,
    published_at: String,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

impl UpdateChecker {
    pub fn new(current_version: &str) -> Self {
        let is_beta = current_version.contains("-beta");
        Self {
            repo: "yelog/rust-redis-desktop".to_string(),
            current_version: current_version.to_string(),
            check_beta: is_beta,
        }
    }

    pub async fn check(&self) -> Result<Option<UpdateInfo>> {
        let client = reqwest::Client::builder()
            .user_agent("rust-redis-desktop")
            .build()
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let url = format!(
            "https://api.github.com/repos/{}/releases",
            self.repo
        );

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpdateError::NetworkError(format!(
                "GitHub API returned error: {}",
                response.status()
            )));
        }

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        for release in releases {
            if release.prerelease != self.check_beta {
                continue;
            }

            let release_version = release.tag_name.trim_start_matches('v');

            if self.is_newer_version(release_version)? {
                let asset = self.find_asset(&release.assets)?;

                return Ok(Some(UpdateInfo {
                    version: release_version.to_string(),
                    release_notes: release.body,
                    download_url: asset.browser_download_url.clone(),
                    asset_name: asset.name.clone(),
                    is_beta: release.prerelease,
                    published_at: release.published_at,
                }));
            }
        }

        Ok(None)
    }

    fn is_newer_version(&self, new_version: &str) -> Result<bool> {
        let current = Version::parse(&self.clean_version(&self.current_version))
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        let new = Version::parse(&self.clean_version(new_version))
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        Ok(new > current)
    }

    pub fn clean_version(&self, version: &str) -> String {
        version
            .split('-')
            .next()
            .unwrap_or(version)
            .to_string()
    }

    fn find_asset<'a>(&self, assets: &'a [GitHubAsset]) -> Result<&'a GitHubAsset> {
        let suffix = Platform::current().asset_suffix();

        assets
            .iter()
            .find(|a| a.name.ends_with(suffix))
            .ok_or_else(|| UpdateError::PlatformNotSupported)
    }
}

pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}