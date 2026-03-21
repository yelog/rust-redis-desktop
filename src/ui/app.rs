use crate::config::{AppSettings, ConfigStorage};
use crate::connection::{ConnectionConfig, ConnectionManager, ConnectionPool, ConnectionState};
use crate::theme::{ThemeColors, ThemeMode};
use crate::ui::{
    ClientsPanel, ConnectionForm, FlushConfirmDialog, KeyBrowser, MonitorPanel, ResizableDivider,
    ServerInfoPanel, SettingsDialog, Sidebar, SlowLogPanel, Terminal, ValueViewer,
};
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Data,
    Terminal,
    Monitor,
    SlowLog,
    Clients,
}

#[derive(Clone, PartialEq)]
pub enum FormMode {
    New,
    Edit(ConnectionConfig),
}

#[component]
pub fn App() -> Element {
    let mut connections = use_signal(Vec::new);
    let mut form_mode = use_signal(|| None::<FormMode>);
    let mut selected_connection = use_signal(|| None::<Uuid>);
    let connection_manager = use_signal(ConnectionManager::new);
    let config_storage = use_signal(|| ConfigStorage::new().ok());
    let mut selected_key = use_signal(String::new);
    let mut connection_pools = use_signal(HashMap::<Uuid, ConnectionPool>::new);
    let mut refresh_trigger = use_signal(|| 0u32);
    let mut current_tab = use_signal(|| Tab::Data);
    let mut reconnecting_ids = use_signal(HashSet::<Uuid>::new);
    let mut connection_versions = use_signal(HashMap::<Uuid, u32>::new);
    let mut connection_states = use_signal(HashMap::<Uuid, ConnectionState>::new);
    let mut app_settings = use_signal(AppSettings::default);
    let mut show_settings = use_signal(|| false);
    let mut show_flush_dialog = use_signal(|| None::<Uuid>);
    let mut current_db = use_signal(|| 0u8);
    let sidebar_width = use_signal(|| 250.0);
    let key_browser_width = use_signal(|| 300.0);
    let mut theme_mode = use_signal(ThemeMode::default);

    let colors = match theme_mode.read().clone() {
        ThemeMode::Dark => ThemeColors::dark(),
        ThemeMode::Light => ThemeColors::light(),
        ThemeMode::System => {
            #[cfg(target_os = "macos")]
            {
                let is_dark = std::process::Command::new("defaults")
                    .args(["read", "-g", "AppleInterfaceStyle"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dark"))
                    .unwrap_or(false);
                if is_dark {
                    ThemeColors::dark()
                } else {
                    ThemeColors::light()
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                ThemeColors::dark()
            }
        }
    };

    use_effect(move || {
        if let Some(storage) = config_storage.read().as_ref() {
            if let Ok(saved) = storage.load_connections() {
                connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
            }
            if let Ok(settings) = storage.load_settings() {
                app_settings.set(settings.clone());
                theme_mode.set(settings.theme_mode);
            }
        }
    });

    use_effect(|| {
        let _ = document::eval("document.body.style.margin = '0'; document.body.style.padding = '0'; document.documentElement.style.margin = '0'; document.documentElement.style.padding = '0';");
    });

    let save_settings = {
        let config_storage = config_storage.clone();
        let mut theme_mode = theme_mode.clone();
        move |settings: AppSettings| {
            app_settings.set(settings.clone());
            theme_mode.set(settings.theme_mode);
            if let Some(storage) = config_storage.read().as_ref() {
                let _ = storage.save_settings(&settings);
            }
        }
    };

    rsx! {
        div {
            display: "flex",
            height: "100vh",
            background: "{colors.background}",
            color: "{colors.text}",
            overflow: "hidden",
            onkeydown: move |e| {
                let key = e.data().key();
                let modifiers = e.data().modifiers();
                if key == Key::Character(",".to_string()) && modifiers.contains(Modifiers::SUPER) {
                    show_settings.set(true);
                }
            },

            Sidebar {
                width: sidebar_width(),
                connections: connections(),
                connection_states: connection_states(),
                selected_connection: selected_connection(),
                colors: colors.clone(),
                on_add_connection: move |_| form_mode.set(Some(FormMode::New)),
                on_select_connection: move |id: Uuid| {
                    selected_connection.set(Some(id));
                    selected_key.set(String::new());

                    spawn(async move {
                        if connection_pools.read().contains_key(&id) {
                            let version = connection_versions.read().get(&id).copied().unwrap_or(0);
                            connection_versions.write().insert(id, version + 1);
                            return;
                        }

                        connection_states.write().insert(id, ConnectionState::Connecting);

                        if let Some(pool) = connection_manager.read().get_connection(id).await {
                            connection_pools.write().insert(id, pool);
                            connection_states.write().insert(id, ConnectionState::Connected);
                            return;
                        }

                        if let Some(storage) = config_storage.read().as_ref() {
                            if let Ok(saved) = storage.load_connections() {
                                if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                    match ConnectionPool::new(config.clone()).await {
                                        Ok(pool) => {
                                            let _ = connection_manager.read().add_connection(config).await;
                                            connection_pools.write().insert(id, pool);
                                            connection_states.write().insert(id, ConnectionState::Connected);
                                        }
                                        Err(_) => {
                                            connection_states.write().insert(id, ConnectionState::Error);
                                        }
                                    }
                                }
                            }
                        }
                    });
                },
                on_reconnect_connection: move |id: Uuid| {
                    spawn(async move {
                        reconnecting_ids.write().insert(id);
                        connection_states.write().insert(id, ConnectionState::Connecting);

                        if let Some(storage) = config_storage.read().as_ref() {
                            if let Ok(saved) = storage.load_connections() {
                                if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                    match ConnectionPool::new(config.clone()).await {
                                        Ok(pool) => {
                                            connection_pools.write().insert(id, pool);
                                            let _ = connection_manager.read().add_connection(config).await;

                                            let version = connection_versions.read().get(&id).copied().unwrap_or(0);
                                            connection_versions.write().insert(id, version + 1);
                                            connection_states.write().insert(id, ConnectionState::Connected);
                                        }
                                        Err(_) => {
                                            connection_states.write().insert(id, ConnectionState::Error);
                                        }
                                    }
                                }
                            }
                        }

                        reconnecting_ids.write().remove(&id);
                    });
                },
                on_close_connection: move |id: Uuid| {
                    spawn(async move {
                        connection_pools.write().remove(&id);
                        connection_manager.read().remove_connection(id).await;
                        connection_states.write().insert(id, ConnectionState::Disconnected);

                        if selected_connection() == Some(id) {
                            selected_connection.set(None);
                            selected_key.set(String::new());
                        }
                    });
                },
                on_edit_connection: move |id: Uuid| {
                    if let Some(storage) = config_storage.read().as_ref() {
                        if let Ok(saved) = storage.load_connections() {
                            if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                form_mode.set(Some(FormMode::Edit(config)));
                            }
                        }
                    }
                },
                on_delete_connection: move |id: Uuid| {
                    spawn(async move {
                        if let Some(storage) = config_storage.read().as_ref() {
                            let _ = storage.delete_connection(id);
                        }

                        connection_pools.write().remove(&id);
                        connection_manager.read().remove_connection(id).await;
                        connection_states.write().remove(&id);

                        if let Some(storage) = config_storage.read().as_ref() {
                            if let Ok(saved) = storage.load_connections() {
                                connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
                            }
                        }

                        if selected_connection() == Some(id) {
                            selected_connection.set(None);
                            selected_key.set(String::new());
                        }
                    });
                },
                on_flush_connection: move |id: Uuid| {
                    show_flush_dialog.set(Some(id));
                },
                on_open_settings: move |_| show_settings.set(true),
            }

            ResizableDivider {
                width: sidebar_width,
                min_width: 150.0,
                max_width: 400.0,
            }

            if let Some(conn_id) = selected_connection() {
                if reconnecting_ids.read().contains(&conn_id) {
                    div {
                        flex: "1",
                        display: "flex",
                        flex_direction: "column",
                        align_items: "center",
                        justify_content: "center",
                        gap: "16px",

                        style { {r#"
                            @keyframes spin {
                                from { transform: rotate(0deg); }
                                to { transform: rotate(360deg); }
                            }
                        "#} }

                        div {
                            width: "40px",
                            height: "40px",
                            border: "3px solid {colors.accent}",
                            border_top_color: "transparent",
                            border_radius: "50%",
                            animation: "spin 0.8s linear infinite",
                        }

                        div {
                            color: "{colors.text_secondary}",
                            font_size: "14px",

                            "Reconnecting..."
                        }
                    }
                } else if let Some(pool) = connection_pools.read().get(&conn_id).cloned() {
                    KeyBrowser {
                        key: "{conn_id}-{connection_versions.read().get(&conn_id).copied().unwrap_or(0)}",
                        width: key_browser_width(),
                        connection_id: conn_id,
                        connection_pool: pool.clone(),
                        connection_version: connection_versions.read().get(&conn_id).copied().unwrap_or(0),
                        selected_key: selected_key,
                        current_db: current_db,
                        refresh_trigger: refresh_trigger,
                        on_key_select: move |key: String| {
                            selected_key.set(key);
                            current_tab.set(Tab::Data);
                        },
                    }

                    ResizableDivider {
                        width: key_browser_width,
                        min_width: 200.0,
                        max_width: 500.0,
                    }

                    div {
                        flex: "1",
                        min_height: "0",
                        display: "flex",
                        flex_direction: "column",
                        overflow: "hidden",

                        // Tab bar
                        div {
                            display: "flex",
                            flex_shrink: "0",
                            border_bottom: "1px solid {colors.border}",
                            background: "{colors.background_secondary}",

                            button {
                                padding: "10px 20px",
                                background: if current_tab() == Tab::Data { colors.background } else { "transparent" },
                                color: if current_tab() == Tab::Data { colors.text } else { colors.text_secondary },
                                border: "none",
                                border_bottom: if current_tab() == Tab::Data { "2px solid {colors.accent}" } else { "none" },
                                cursor: "pointer",
                                font_size: "13px",
                                onclick: move |_| current_tab.set(Tab::Data),

                                "📊 Data"
                            }

                            button {
                                padding: "10px 20px",
                                background: if current_tab() == Tab::Terminal { colors.background } else { "transparent" },
                                color: if current_tab() == Tab::Terminal { colors.text } else { colors.text_secondary },
                                border: "none",
                                border_bottom: if current_tab() == Tab::Terminal { "2px solid {colors.accent}" } else { "none" },
                                cursor: "pointer",
                                font_size: "13px",
                                onclick: move |_| current_tab.set(Tab::Terminal),

                                "💻 Terminal"
                            }

                            button {
                                padding: "10px 20px",
                                background: if current_tab() == Tab::Monitor { colors.background } else { "transparent" },
                                color: if current_tab() == Tab::Monitor { colors.text } else { colors.text_secondary },
                                border: "none",
                                border_bottom: if current_tab() == Tab::Monitor { "2px solid {colors.accent}" } else { "none" },
                                cursor: "pointer",
                                font_size: "13px",
                                onclick: move |_| current_tab.set(Tab::Monitor),

                                "📈 Monitor"
                            }

                            button {
                                padding: "10px 20px",
                                background: if current_tab() == Tab::SlowLog { colors.background } else { "transparent" },
                                color: if current_tab() == Tab::SlowLog { colors.text } else { colors.text_secondary },
                                border: "none",
                                border_bottom: if current_tab() == Tab::SlowLog { "2px solid {colors.accent}" } else { "none" },
                                cursor: "pointer",
                                font_size: "13px",
                                onclick: move |_| current_tab.set(Tab::SlowLog),

                                "🐌 SlowLog"
                            }

                            button {
                                padding: "10px 20px",
                                background: if current_tab() == Tab::Clients { colors.background } else { "transparent" },
                                color: if current_tab() == Tab::Clients { colors.text } else { colors.text_secondary },
                                border: "none",
                                border_bottom: if current_tab() == Tab::Clients { "2px solid {colors.accent}" } else { "none" },
                                cursor: "pointer",
                                font_size: "13px",
                                onclick: move |_| current_tab.set(Tab::Clients),

                                "👥 Clients"
                            }
                        }

                        // Tab content
                        div {
                            flex: "1",
                            overflow: "hidden",

                            if current_tab() == Tab::Data {
                                if !selected_key.read().is_empty() {
                                    ValueViewer {
                                        key: "{conn_id}",
                                        connection_pool: pool,
                                        selected_key: selected_key,
                                        on_refresh: move |_| {
                                            refresh_trigger.set(refresh_trigger() + 1);
                                        },
                                    }
                                } else {
                                    ServerInfoPanel {
                                        key: "{conn_id}",
                                        connection_pool: pool,
                                        connection_version: connection_versions.read().get(&conn_id).copied().unwrap_or(0),
                                        auto_refresh_interval: app_settings.read().auto_refresh_interval,
                                    }
                                }
                            } else if current_tab() == Tab::Terminal {
                                Terminal {
                                    key: "{conn_id}",
                                    connection_pool: pool,
                                }
                            } else if current_tab() == Tab::Monitor {
                                MonitorPanel {
                                    key: "{conn_id}",
                                    connection_pool: pool,
                                    auto_refresh_interval: app_settings.read().auto_refresh_interval,
                                }
                            } else if current_tab() == Tab::SlowLog {
                                SlowLogPanel {
                                    key: "{conn_id}",
                                    connection_pool: pool,
                                }
                            } else {
                                ClientsPanel {
                                    key: "{conn_id}",
                                    connection_pool: pool,
                                }
                            }
                        }
                    }
                } else {
                    div {
                        flex: "1",
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        color: "{colors.text_secondary}",

                        "Loading connection..."
                    }
                }
            } else {
                div {
                    flex: "1",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    color: "{colors.text_secondary}",
                    font_size: "24px",

                    "Select a connection or create a new one"
                }
            }
        }

        if let Some(mode) = form_mode() {
            ConnectionForm {
                editing_config: match mode {
                    FormMode::Edit(config) => Some(config),
                    FormMode::New => None,
                },
                on_save: move |config: ConnectionConfig| {
                    let id = config.id;
                    let name = config.name.clone();

                    spawn(async move {
                        tracing::info!("=== Save Connection Start ===");
                        tracing::info!("Connection: {} ({})", name, id);

                        let storage = config_storage.read();
                        if storage.is_none() {
                            tracing::error!("ConfigStorage is None!");
                            form_mode.set(None);
                            return;
                        }

                        let storage = storage.as_ref().unwrap();

                        tracing::info!("Saving to storage...");
                        match storage.save_connection(config.clone()) {
                            Ok(_) => tracing::info!("✓ Config saved successfully"),
                            Err(e) => {
                                tracing::error!("✗ Save failed: {}", e);
                                form_mode.set(None);
                                return;
                            }
                        }

                        tracing::info!("Reloading connections...");
                        match storage.load_connections() {
                            Ok(saved) => {
                                let list: Vec<(Uuid, String)> = saved.into_iter().map(|c| (c.id, c.name)).collect();
                                tracing::info!("✓ Loaded {} connections: {:?}", list.len(), list);
                                connections.set(list);
                            }
                            Err(e) => {
                                tracing::error!("✗ Load failed: {}", e);
                            }
                        }

                        let _ = connection_manager.read().add_connection(config).await;

                        tracing::info!("=== Save Connection End ===");
                        form_mode.set(None);
                    });
                },
                on_cancel: move |_| form_mode.set(None),
            }
        }

        if show_settings() {
            SettingsDialog {
                settings: app_settings.read().clone(),
                colors: colors.clone(),
                on_save: {
                    let mut save_settings = save_settings.clone();
                    move |settings: AppSettings| {
                        save_settings(settings);
                    }
                },
                on_close: move |_| show_settings.set(false),
            }
        }

        if let Some(flush_id) = show_flush_dialog() {
            if let Some(pool) = connection_pools.read().get(&flush_id).cloned() {
                FlushConfirmDialog {
                    connection_pool: pool,
                    current_db: current_db(),
                    on_confirm: move |_| {
                        show_flush_dialog.set(None);
                        refresh_trigger.set(refresh_trigger() + 1);
                    },
                    on_cancel: move |_| show_flush_dialog.set(None),
                }
            }
        }
    }
}
