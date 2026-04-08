use std::path::Path;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn command_for(executable: &Path) -> String {
    let executable = executable.to_string_lossy().replace('"', "\\\"");
    format!("\"{executable}\"")
}

#[cfg(target_os = "windows")]
pub(crate) fn set_enabled(
    value_name: &str,
    executable: &Path,
    enabled: bool,
) -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = current_user
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|err| format!("Failed to open Run registry key: {err}"))?;

    if enabled {
        run_key
            .set_value(value_name, &command_for(executable))
            .map_err(|err| format!("Failed to write Run registry value: {err}"))?;
    } else if let Err(err) = run_key.delete_value(value_name) {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(format!("Failed to remove Run registry value: {err}"));
        }
    }

    Ok(())
}
