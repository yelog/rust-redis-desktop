use crate::updater::{Platform, UpdateChecker};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "x86_64")]
            assert_eq!(platform, Platform::MacOSX86);
            #[cfg(target_arch = "aarch64")]
            assert_eq!(platform, Platform::MacOSArm);
        }
        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);
        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux);
    }

    #[test]
    fn test_platform_asset_suffix() {
        assert_eq!(Platform::MacOSX86.asset_suffix(), "x86_64.dmg");
        assert_eq!(Platform::MacOSArm.asset_suffix(), "aarch64.dmg");
        assert_eq!(Platform::Windows.asset_suffix(), "x86_64-setup.exe");
        assert_eq!(Platform::Linux.asset_suffix(), "x86_64.AppImage");
    }

    #[test]
    fn test_platform_manifest_keys() {
        assert_eq!(Platform::MacOSX86.manifest_key(), "macos-x86_64");
        assert_eq!(Platform::MacOSArm.manifest_key(), "macos-aarch64");
        assert_eq!(Platform::Windows.manifest_key(), "windows-x86_64");
        assert_eq!(Platform::Linux.manifest_key(), "linux-x86_64");
    }

    #[test]
    fn test_update_checker_new() {
        let checker = UpdateChecker::new("0.1.0");
        assert_eq!(checker.current_version, "0.1.0");
        assert!(!checker.check_beta);

        let checker_beta = UpdateChecker::new("0.1.0-beta.1");
        assert!(checker_beta.check_beta);
    }

    #[test]
    fn test_version_cleaning() {
        let checker = UpdateChecker::new("0.1.0");
        assert_eq!(checker.clean_version("0.1.0-beta.1"), "0.1.0-beta.1");
        assert_eq!(checker.clean_version("0.2.0"), "0.2.0");
        assert_eq!(checker.clean_version("v0.2.1-beta.3"), "0.2.1-beta.3");
    }

    #[test]
    fn test_stable_channel_manifest_parsing() {
        let checker = UpdateChecker::new("0.1.0");
        let info = checker
            .parse_manifest_json(sample_manifest())
            .expect("manifest should parse")
            .expect("should find newer stable update");

        assert_eq!(info.version, "0.1.2");
        assert!(!info.is_beta);
        assert_eq!(info.published_at, "2026-04-08T00:00:00Z");
        assert_eq!(info.release_notes, "stable release notes");
        assert_eq!(info.asset_name, expected_asset_name());
        assert_eq!(info.download_url, expected_asset_url());
    }

    #[test]
    fn test_beta_channel_manifest_parsing() {
        let checker = UpdateChecker::new("0.1.2-beta.1");
        let info = checker
            .parse_manifest_json(sample_manifest())
            .expect("manifest should parse")
            .expect("should find newer beta update");

        assert_eq!(info.version, "0.1.3-beta.2");
        assert!(info.is_beta);
        assert_eq!(info.release_notes, "beta release notes");
    }

    #[test]
    fn test_beta_channel_detects_newer_prerelease_with_same_base_version() {
        let checker = UpdateChecker::new("0.1.1-beta.6");
        let manifest = same_base_beta_manifest("0.1.1-beta.7");
        let info = checker
            .parse_manifest_json(&manifest)
            .expect("manifest should parse")
            .expect("should find newer beta update");

        assert_eq!(info.version, "0.1.1-beta.7");
        assert!(info.is_beta);
    }

    #[test]
    fn test_beta_channel_returns_none_for_same_prerelease_version() {
        let checker = UpdateChecker::new("0.1.1-beta.7");
        let manifest = same_base_beta_manifest("0.1.1-beta.7");
        let info = checker
            .parse_manifest_json(&manifest)
            .expect("manifest should parse");

        assert!(info.is_none());
    }

    #[test]
    fn test_manifest_returns_none_for_current_version() {
        let checker = UpdateChecker::new("0.1.2");
        let info = checker
            .parse_manifest_json(sample_manifest())
            .expect("manifest should parse");

        assert!(info.is_none());
    }

    #[test]
    fn test_manifest_missing_platform_is_not_supported() {
        let checker = UpdateChecker::new("0.1.0");
        let error = checker
            .parse_manifest_json(&manifest_without_current_platform())
            .expect_err("current platform should be required");

        assert!(matches!(
            error,
            crate::updater::UpdateError::PlatformNotSupported
        ));
    }

    fn sample_manifest() -> &'static str {
        r#"
        {
          "channels": {
            "stable": {
              "version": "0.1.2",
              "published_at": "2026-04-08T00:00:00Z",
              "release_notes": "stable release notes",
              "platforms": {
                "macos-x86_64": {
                  "url": "https://example.com/rust-redis-desktop-x86_64.dmg",
                  "asset_name": "rust-redis-desktop-x86_64.dmg"
                },
                "macos-aarch64": {
                  "url": "https://example.com/rust-redis-desktop-aarch64.dmg",
                  "asset_name": "rust-redis-desktop-aarch64.dmg"
                },
                "windows-x86_64": {
                  "url": "https://example.com/rust-redis-desktop-x86_64-setup.exe",
                  "asset_name": "rust-redis-desktop-x86_64-setup.exe"
                },
                "linux-x86_64": {
                  "url": "https://example.com/rust-redis-desktop-x86_64.AppImage",
                  "asset_name": "rust-redis-desktop-x86_64.AppImage"
                }
              }
            },
            "beta": {
              "version": "0.1.3-beta.2",
              "published_at": "2026-04-09T00:00:00Z",
              "release_notes": "beta release notes",
              "platforms": {
                "macos-x86_64": {
                  "url": "https://example.com/rust-redis-desktop-x86_64-beta.dmg",
                  "asset_name": "rust-redis-desktop-x86_64.dmg"
                },
                "macos-aarch64": {
                  "url": "https://example.com/rust-redis-desktop-aarch64-beta.dmg",
                  "asset_name": "rust-redis-desktop-aarch64.dmg"
                },
                "windows-x86_64": {
                  "url": "https://example.com/rust-redis-desktop-x86_64-beta-setup.exe",
                  "asset_name": "rust-redis-desktop-x86_64-setup.exe"
                },
                "linux-x86_64": {
                  "url": "https://example.com/rust-redis-desktop-x86_64-beta.AppImage",
                  "asset_name": "rust-redis-desktop-x86_64.AppImage"
                }
              }
            }
          }
        }
        "#
    }

    fn manifest_without_current_platform() -> String {
        match Platform::current() {
            Platform::MacOSX86 => sample_manifest().replace("\"macos-x86_64\": {\n                  \"url\": \"https://example.com/rust-redis-desktop-x86_64.dmg\",\n                  \"asset_name\": \"rust-redis-desktop-x86_64.dmg\"\n                },\n", ""),
            Platform::MacOSArm => sample_manifest().replace("\"macos-aarch64\": {\n                  \"url\": \"https://example.com/rust-redis-desktop-aarch64.dmg\",\n                  \"asset_name\": \"rust-redis-desktop-aarch64.dmg\"\n                },\n", ""),
            Platform::Windows => sample_manifest().replace("\"windows-x86_64\": {\n                  \"url\": \"https://example.com/rust-redis-desktop-x86_64-setup.exe\",\n                  \"asset_name\": \"rust-redis-desktop-x86_64-setup.exe\"\n                },\n", ""),
            Platform::Linux => sample_manifest().replace("\"linux-x86_64\": {\n                  \"url\": \"https://example.com/rust-redis-desktop-x86_64.AppImage\",\n                  \"asset_name\": \"rust-redis-desktop-x86_64.AppImage\"\n                }\n", ""),
        }
    }

    fn same_base_beta_manifest(version: &str) -> String {
        sample_manifest().replace("0.1.3-beta.2", version)
    }

    fn expected_asset_name() -> &'static str {
        match Platform::current() {
            Platform::MacOSX86 => "rust-redis-desktop-x86_64.dmg",
            Platform::MacOSArm => "rust-redis-desktop-aarch64.dmg",
            Platform::Windows => "rust-redis-desktop-x86_64-setup.exe",
            Platform::Linux => "rust-redis-desktop-x86_64.AppImage",
        }
    }

    fn expected_asset_url() -> &'static str {
        match Platform::current() {
            Platform::MacOSX86 => "https://example.com/rust-redis-desktop-x86_64.dmg",
            Platform::MacOSArm => "https://example.com/rust-redis-desktop-aarch64.dmg",
            Platform::Windows => "https://example.com/rust-redis-desktop-x86_64-setup.exe",
            Platform::Linux => "https://example.com/rust-redis-desktop-x86_64.AppImage",
        }
    }
}
