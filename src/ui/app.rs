use crate::config::ConfigStorage;
use crate::connection::{ConnectionConfig, ConnectionManager, ConnectionPool};
use crate::ui::{ConnectionForm, KeyBrowser, Sidebar, Terminal, ValueViewer};
use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Data,
    Terminal,
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
    let mut connection_pools = use_signal(std::collections::HashMap::<Uuid, ConnectionPool>::new);
    let mut refresh_trigger = use_signal(|| 0u32);
    let mut current_tab = use_signal(|| Tab::Data);

    use_effect(move || {
        if let Some(storage) = config_storage.read().as_ref() {
            if let Ok(saved) = storage.load_connections() {
                connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
            }
        }
    });

    rsx! {
                        div {
                            display: "flex",
                            height: "100vh",
                            background: "#1e1e1e",
                            color: "white",
                            overflow: "hidden",

                            Sidebar {
                                connections: connections(),
                                on_add_connection: move |_| form_mode.set(Some(FormMode::New)),
                                on_select_connection: move |id: Uuid| {
                                    selected_connection.set(Some(id));
                                    selected_key.set(String::new());

                                    spawn(async move {
                                        if !connection_pools.read().contains_key(&id) {
                                            if let Some(pool) = connection_manager.read().get_connection(id).await {
                                                connection_pools.write().insert(id, pool);
                                            }
                                        }
                                    });
                                },
                                on_edit_connection: move |id: Uuid| {
                                    // Load connection config
                                    if let Some(storage) = config_storage.read().as_ref() {
                                        if let Ok(saved) = storage.load_connections() {
                                            if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                                form_mode.set(Some(FormMode::Edit(config)));
                                            }
                                        }
                                    }
                                },
                                on_delete_connection: move |id: Uuid| {
                                    // Delete connection
                                    spawn(async move {
                                        // Remove from storage
                                        if let Some(storage) = config_storage.read().as_ref() {
                                            let _ = storage.delete_connection(id);
                                        }

                                        // Remove from memory
                                        connection_pools.write().remove(&id);
                                        connection_manager.read().remove_connection(id).await;

                                        // Reload connections list
                                        if let Some(storage) = config_storage.read().as_ref() {
                                            if let Ok(saved) = storage.load_connections() {
                                                connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
                                            }
                                        }

                                        // Clear selection if deleted
                                        if selected_connection() == Some(id) {
                                            selected_connection.set(None);
                                            selected_key.set(String::new());
                                        }
                                    });
                                },
                            }

                            if let Some(conn_id) = selected_connection() {
                                if let Some(pool) = connection_pools.read().get(&conn_id).cloned() {
                                    KeyBrowser {
                                        connection_id: conn_id,
                                        connection_pool: pool.clone(),
                                        selected_key: selected_key,
                                        on_key_select: move |key: String| {
                                            selected_key.set(key);
                                            current_tab.set(Tab::Data);
                                        },
                                    }

                                    div {
                                        flex: "1",
                                        display: "flex",
                                        flex_direction: "column",

                                        // Tab bar
                                        div {
                                            display: "flex",
                                            border_bottom: "1px solid #3c3c3c",
                                            background: "#252526",

                                            button {
                                                padding: "10px 20px",
                                                background: if current_tab() == Tab::Data { "#1e1e1e" } else { "transparent" },
                                                color: if current_tab() == Tab::Data { "white" } else { "#888" },
                                                border: "none",
                                                border_bottom: if current_tab() == Tab::Data { "2px solid #4ec9b0" } else { "none" },
                                                cursor: "pointer",
                                                font_size: "13px",
                                                onclick: move |_| current_tab.set(Tab::Data),

                                                "📊 Data"
                                            }

                                            button {
                                                padding: "10px 20px",
                                                background: if current_tab() == Tab::Terminal { "#1e1e1e" } else { "transparent" },
                                                color: if current_tab() == Tab::Terminal { "white" } else { "#888" },
                                                border: "none",
                                                border_bottom: if current_tab() == Tab::Terminal { "2px solid #4ec9b0" } else { "none" },
                                                cursor: "pointer",
                                                font_size: "13px",
                                                onclick: move |_| current_tab.set(Tab::Terminal),

                                                "💻 Terminal"
                                            }
                                        }

                                        // Tab content
                                        div {
                                            flex: "1",
                                            overflow: "hidden",

                                            if current_tab() == Tab::Data {
                                                if !selected_key.read().is_empty() {
                                                    ValueViewer {
                                                        connection_pool: pool,
                                                        selected_key: selected_key,
                                                        on_refresh: move |_| {
                                                            refresh_trigger.set(refresh_trigger() + 1);
                                                        },
                                                    }
                                                } else {
                                                    div {
                                                        height: "100%",
                                                        display: "flex",
                                                        align_items: "center",
                                                        justify_content: "center",
                                                        color: "#888",
                                                        font_size: "18px",

                                                        "Select a key to view its value"
                                                    }
                                                }
                                            } else {
                                                Terminal {
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
                                        color: "#888",

                                        "Loading connection..."
                                    }
                                }
                            } else if let Some(mode) = form_mode() {
                                div {
                                    flex: "1",
                                    display: "flex",
                                    align_items: "center",
                                    justify_content: "center",

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

                                    // Get storage
                                    let storage = config_storage.read();
                                    if storage.is_none() {
                                        tracing::error!("ConfigStorage is None!");
                                        form_mode.set(None);
                                        return;
                                    }

                                    let storage = storage.as_ref().unwrap();

                                    // Save to storage
                                    tracing::info!("Saving to storage...");
                                    match storage.save_connection(config.clone()) {
                                        Ok(_) => tracing::info!("✓ Config saved successfully"),
                                        Err(e) => {
                                            tracing::error!("✗ Save failed: {}", e);
                                            form_mode.set(None);
                                            return;
                                        }
                                    }

                                    // Reload list
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

                                    // Try to connect
                                    let _ = connection_manager.read().add_connection(config).await;

                                    tracing::info!("=== Save Connection End ===");
                                    form_mode.set(None);
                                });
                            },
                                        on_cancel: move |_| form_mode.set(None),
                                    }
                                }
                            } else {
                                div {
                                    flex: "1",
                                    display: "flex",
                                    align_items: "center",
                                    justify_content: "center",
                                    color: "#888",
                                    font_size: "24px",

                                    "Select a connection or create a new one"
                                }
                            }
                        }
                    }
}
