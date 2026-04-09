use crate::config::ConfigStorage;
use crate::error::Result;
use crate::i18n::I18n;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::error;

#[cfg(not(target_os = "linux"))]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub id: String,
    pub name: String,
    pub connected: bool,
    pub memory: Option<String>,
    pub keys: Option<u64>,
}

pub struct TrayState {
    pub servers: Vec<ServerStatus>,
    pub active_server_id: Option<String>,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            active_server_id: None,
        }
    }
}

pub type SharedTrayState = Arc<Mutex<TrayState>>;

pub fn create_shared_state() -> SharedTrayState {
    Arc::new(Mutex::new(TrayState::default()))
}

#[cfg(not(target_os = "linux"))]
fn load_icon() -> std::result::Result<Icon, Box<dyn std::error::Error>> {
    let icon_bytes = include_bytes!("../icons/icon.png");
    let img = image::load_from_memory(icon_bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(Icon::from_rgba(rgba.into_raw(), width, height)?)
}

#[cfg(not(target_os = "linux"))]
pub fn init_tray(state: SharedTrayState) -> Result<()> {
    let settings = ConfigStorage::new()
        .ok()
        .and_then(|storage| storage.load_settings().ok())
        .unwrap_or_default();
    let i18n = I18n::new(settings.language_preference.resolve());
    let icon = load_icon()
        .map_err(|e| crate::error::AppError::Other(format!("Failed to load tray icon: {}", e)))?;

    let menu = Menu::new();

    let _ = menu.append(&MenuItem::with_id(
        "show",
        &i18n.t("Show window"),
        true,
        None,
    ));
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id("quit", &i18n.t("Quit"), true, None));

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(&i18n.t("Redis Desktop"))
        .with_icon(icon)
        .build();

    match tray {
        Ok(_tray) => {
            std::thread::spawn(move || {
                let receiver = MenuEvent::receiver();
                loop {
                    let Ok(event) = receiver.recv() else {
                        return;
                    };
                    let id = event.id().0.clone();
                    match id.as_str() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "show" => {
                            // TODO: bring window to front
                        }
                        id if id.starts_with("server:") => {
                            if let Some(server_id) = id.strip_prefix("server:") {
                                match state.lock() {
                                    Ok(mut s) => {
                                        s.active_server_id = Some(server_id.to_string());
                                    }
                                    Err(poisoned) => {
                                        let mut s = poisoned.into_inner();
                                        s.active_server_id = Some(server_id.to_string());
                                        tracing::warn!("Tray state mutex was poisoned, recovered");
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });
        }
        Err(e) => {
            error!("Failed to create tray icon: {}", e);
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn init_tray(_state: SharedTrayState) -> Result<()> {
    Ok(())
}
