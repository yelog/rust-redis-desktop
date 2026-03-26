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
fn load_icon() -> Icon {
    let icon_bytes = include_bytes!("../icons/icon.png");
    let img = image::load_from_memory(icon_bytes).expect("Failed to load tray icon");
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height).expect("Failed to create tray icon")
}

#[cfg(not(target_os = "linux"))]
pub fn init_tray(state: SharedTrayState) {
    let icon = load_icon();
    let menu = Menu::new();

    let _ = menu.append(&MenuItem::with_id("show", "显示窗口", true, None));
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id("quit", "退出", true, None));

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Redis Desktop")
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
                                let mut s = state.lock().unwrap();
                                s.active_server_id = Some(server_id.to_string());
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
}

#[cfg(target_os = "linux")]
pub fn init_tray(_state: SharedTrayState) {
    // System tray is not supported on Linux in this implementation
}
