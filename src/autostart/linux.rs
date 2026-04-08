use std::path::{Path, PathBuf};

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn autostart_file_path(config_dir: &Path, desktop_entry_name: &str) -> PathBuf {
    config_dir.join("autostart").join(desktop_entry_name)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn desktop_entry_contents(app_name: &str, executable: &Path) -> String {
    let executable = executable.to_string_lossy().replace('"', "\\\"");

    format!(
        "[Desktop Entry]\nType=Application\nVersion=1.0\nName={app_name}\nExec=\"{executable}\"\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn set_enabled(
    app_name: &str,
    desktop_entry_name: &str,
    executable: &Path,
    enabled: bool,
) -> Result<(), String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "Failed to determine config directory".to_string())?;
    let path = autostart_file_path(&config_dir, desktop_entry_name);

    if enabled {
        let parent = path
            .parent()
            .ok_or_else(|| "Invalid autostart directory".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create autostart directory: {err}"))?;
        std::fs::write(&path, desktop_entry_contents(app_name, executable))
            .map_err(|err| format!("Failed to write desktop entry: {err}"))?;
    } else if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|err| format!("Failed to remove desktop entry: {err}"))?;
    }

    Ok(())
}
