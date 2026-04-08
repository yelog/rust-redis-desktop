#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_linux_desktop_entry_contains_exec_line() {
        let entry = crate::autostart::linux::desktop_entry_contents(
            "Rust Redis Desktop",
            Path::new("/tmp/Rust Redis Desktop"),
        );

        assert!(entry.contains("Exec=\"/tmp/Rust Redis Desktop\""));
        assert!(entry.contains("Name=Rust Redis Desktop"));
    }

    #[test]
    fn test_macos_plist_contains_label_and_program() {
        let plist = crate::autostart::macos::plist_contents(
            "dev.yelog.rust-redis-desktop",
            Path::new("/Applications/Rust Redis Desktop.app/Contents/MacOS/rust-redis-desktop"),
        );

        assert!(plist.contains("dev.yelog.rust-redis-desktop"));
        assert!(plist.contains("rust-redis-desktop"));
        assert!(plist.contains("ProgramArguments"));
    }

    #[test]
    fn test_windows_command_quotes_executable_path() {
        let command = crate::autostart::windows::command_for(Path::new(
            r"C:\Program Files\Rust Redis Desktop\rust-redis-desktop.exe",
        ));

        assert_eq!(
            command,
            r#""C:\Program Files\Rust Redis Desktop\rust-redis-desktop.exe""#
        );
    }
}
