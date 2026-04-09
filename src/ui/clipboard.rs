use arboard::Clipboard;
use std::sync::{Mutex, OnceLock};

static APP_CLIPBOARD: OnceLock<Mutex<Option<Clipboard>>> = OnceLock::new();

fn clipboard_slot() -> &'static Mutex<Option<Clipboard>> {
    APP_CLIPBOARD.get_or_init(|| Mutex::new(None))
}

pub fn copy_text_to_clipboard(value: &str) -> Result<(), String> {
    let mut clipboard = clipboard_slot()
        .lock()
        .map_err(|_| "Clipboard is currently unavailable".to_string())?;

    if clipboard.is_none() {
        *clipboard = Some(Clipboard::new().map_err(|e| e.to_string())?);
    }

    clipboard
        .as_mut()
        .ok_or_else(|| "Failed to initialize clipboard".to_string())?
        .set_text(value.to_string())
        .map_err(|e| e.to_string())
}
