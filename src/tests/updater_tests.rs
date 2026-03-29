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
        assert_eq!(Platform::Windows.asset_suffix(), "x86_64-windows.zip");
        assert_eq!(Platform::Linux.asset_suffix(), "x86_64.AppImage");
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
        assert_eq!(checker.clean_version("0.1.0-beta.1"), "0.1.0");
        assert_eq!(checker.clean_version("0.2.0"), "0.2.0");
    }
}
