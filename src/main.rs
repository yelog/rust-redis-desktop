mod config;
mod connection;
mod crypto;
mod formatter;
mod i18n;
mod protobuf_schema;
mod redis;
mod serialization;
mod theme;
mod tray;
mod ui;
mod updater;

use config::ConfigStorage;
use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::{
    muda::{accelerator::Accelerator, Menu, MenuItem, PredefinedMenuItem, Submenu},
    Config, WindowBuilder,
};
use theme::preferred_window_theme;
use tray::{create_shared_state, init_tray};
use ui::App;
use updater::{set_pending_update, trigger_manual_check, UpdateManager};

#[cfg(target_os = "macos")]
use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;

#[cfg(target_os = "macos")]
fn configure_window_builder(window_builder: WindowBuilder) -> WindowBuilder {
    window_builder
        .with_titlebar_transparent(true)
        .with_title_hidden(true)
        .with_fullsize_content_view(true)
}

#[cfg(not(target_os = "macos"))]
fn configure_window_builder(window_builder: WindowBuilder) -> WindowBuilder {
    window_builder
}

fn create_menu() -> Menu {
    let menu = Menu::new();

    let app_menu = Submenu::new("Redis Desktop", true);
    let settings_accelerator = Accelerator::try_from("CmdOrCtrl+Comma").ok();
    let update_accelerator = Accelerator::try_from("CmdOrCtrl+U").ok();
    app_menu
        .append_items(&[
            &MenuItem::with_id("check_updates", "检查更新...", true, update_accelerator),
            &MenuItem::with_id("preferences", "Settings...", true, settings_accelerator),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ])
        .unwrap();

    let edit_menu = Submenu::new("Edit", true);
    edit_menu
        .append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::select_all(None),
        ])
        .unwrap();

    let window_menu = Submenu::new("Window", true);
    window_menu
        .append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::close_window(None),
        ])
        .unwrap();

    menu.append_items(&[&app_menu, &edit_menu, &window_menu])
        .unwrap();

    menu
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .pretty()
        .init();

    tracing::info!("Starting Redis Desktop Manager");

    if let Ok(mut manager) = UpdateManager::new() {
        if manager.should_auto_check() {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    if let Ok(Some(info)) = manager.check_for_updates().await {
                        tracing::info!("Found new version: {}", info.version);
                        set_pending_update(Some(info));
                    }
                });
            });
        }
    }

    let menu = create_menu();

    let settings = ConfigStorage::new()
        .ok()
        .and_then(|s| s.load_settings().ok())
        .unwrap_or_default();

    let window_builder = configure_window_builder(
        WindowBuilder::new()
            .with_title("Redis Desktop")
            .with_inner_size(LogicalSize::new(1200, 800))
            .with_theme(preferred_window_theme(settings.theme_preference))
            .with_visible(true),
    );

    #[cfg(not(target_os = "linux"))]
    {
        let tray_state = create_shared_state();
        init_tray(tray_state);
    }

    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().with_menu(menu).with_window(window_builder))
        .launch(App);
}
