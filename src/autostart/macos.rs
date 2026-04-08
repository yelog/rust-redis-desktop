use std::path::{Path, PathBuf};

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn launch_agent_path(home_dir: &Path, label: &str) -> PathBuf {
    home_dir
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{label}.plist"))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn plist_contents(label: &str, executable: &Path) -> String {
    let executable = escape_xml(&executable.to_string_lossy());

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n    <key>Label</key>\n    <string>{label}</string>\n    <key>ProgramArguments</key>\n    <array>\n        <string>{executable}</string>\n    </array>\n    <key>RunAtLoad</key>\n    <true/>\n</dict>\n</plist>\n"
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(target_os = "macos")]
pub(crate) fn set_enabled(label: &str, executable: &Path, enabled: bool) -> Result<(), String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "Failed to determine home directory".to_string())?;
    let path = launch_agent_path(&home_dir, label);

    if enabled {
        let parent = path
            .parent()
            .ok_or_else(|| "Invalid LaunchAgents path".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create LaunchAgents directory: {err}"))?;
        std::fs::write(&path, plist_contents(label, executable))
            .map_err(|err| format!("Failed to write launch agent plist: {err}"))?;
    } else if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|err| format!("Failed to remove launch agent plist: {err}"))?;
    }

    Ok(())
}
