mod config;
mod connection;
mod redis;
mod serialization;
mod theme;
mod ui;

use dioxus::desktop::{
    muda::{accelerator::Accelerator, Menu, MenuItem, PredefinedMenuItem, Submenu},
    Config,
};
use ui::App;

fn create_menu() -> Menu {
    let menu = Menu::new();

    let app_menu = Submenu::new("Redis Desktop", true);
    let settings_accelerator = Accelerator::try_from("CmdOrCtrl+Comma").ok();
    app_menu
        .append_items(&[
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

    let menu = create_menu();

    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().with_menu(menu))
        .launch(App);
}
