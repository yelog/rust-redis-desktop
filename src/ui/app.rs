use crate::config::ConfigStorage;
use crate::connection::{ConnectionConfig, ConnectionManager, ConnectionPool};
use crate::ui::{ConnectionForm, KeyBrowser, Sidebar, ValueViewer};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn App() -> Element {
    let mut connections = use_signal(Vec::new);
    let mut show_form = use_signal(|| false);
    let mut selected_connection = use_signal(|| None::<Uuid>);
    let connection_manager = use_signal(ConnectionManager::new);
    let config_storage = use_signal(|| ConfigStorage::new().ok());
    let mut selected_key = use_signal(String::new);
    let mut connection_pools = use_signal(std::collections::HashMap::<Uuid, ConnectionPool>::new);

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

            Sidebar {
                connections: connections(),
                on_add_connection: move |_| show_form.set(true),
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
            }

            if let Some(conn_id) = selected_connection() {
                if let Some(pool) = connection_pools.read().get(&conn_id).cloned() {
                    KeyBrowser {
                        connection_id: conn_id,
                        connection_pool: pool.clone(),
                        selected_key: selected_key,
                        on_key_select: move |key: String| {
                            selected_key.set(key);
                        },
                    }

                    if !selected_key.read().is_empty() {
                        ValueViewer {
                            connection_pool: pool,
                            selected_key: selected_key(),
                        }
                    } else {
                        div {
                            flex: "1",
                            display: "flex",
                            align_items: "center",
                            justify_content: "center",
                            color: "#888",
                            font_size: "18px",

                            "Select a key to view its value"
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
            } else if show_form() {
                div {
                    flex: "1",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",

                    ConnectionForm {
                        on_save: move |config: ConnectionConfig| {
                            let id = config.id;
                            let name = config.name.clone();

                            spawn(async move {
                                connection_manager.read().add_connection(config.clone()).await.ok();

                                if let Some(storage) = config_storage.read().as_ref() {
                                    storage.save_connection(config).ok();
                                }

                                connections.write().push((id, name));
                            });

                            show_form.set(false);
                        },
                        on_cancel: move |_| show_form.set(false),
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
