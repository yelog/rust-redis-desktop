use crate::config::{AppSettings, ConfigStorage, KeyScanMode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default_launch_at_startup_disabled() {
        let settings = AppSettings::default();

        assert!(!settings.launch_at_startup);
    }

    #[test]
    fn test_app_settings_deserializes_without_launch_at_startup() {
        let json = r#"{
            "auto_refresh_interval": 0,
            "theme_preference": { "dark": "tokyo_night" },
            "auto_check_updates": true
        }"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();

        assert!(!settings.launch_at_startup);
        assert_eq!(settings.scan_confirmation_threshold, 100_000);
        assert_eq!(settings.progressive_scan_limit, 100_000);
        assert_eq!(settings.key_scan_mode, KeyScanMode::Progressive);
    }

    #[test]
    fn test_save_and_load_settings_preserves_launch_at_startup() {
        let storage = ConfigStorage::new_temp().unwrap();
        let settings = AppSettings {
            launch_at_startup: true,
            ..AppSettings::default()
        };

        storage.save_settings(&settings).unwrap();
        let loaded = storage.load_settings().unwrap();

        assert!(loaded.launch_at_startup);
    }

    #[test]
    fn test_save_and_load_settings_preserves_scan_controls() {
        let storage = ConfigStorage::new_temp().unwrap();
        let settings = AppSettings {
            scan_confirmation_threshold: 50_000,
            progressive_scan_limit: 25_000,
            key_scan_mode: KeyScanMode::Complete,
            ..AppSettings::default()
        };

        storage.save_settings(&settings).unwrap();
        let loaded = storage.load_settings().unwrap();

        assert_eq!(loaded.scan_confirmation_threshold, 50_000);
        assert_eq!(loaded.progressive_scan_limit, 25_000);
        assert_eq!(loaded.key_scan_mode, KeyScanMode::Complete);
    }
}
