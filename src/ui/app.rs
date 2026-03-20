use crate::config::ConfigStorage;
use crate::connection::{ConnectionConfig, ConnectionManager};
use crate::ui::{ConnectionForm, Sidebar};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn App() -> Element {
    let mut connections = use_signal(Vec::new);
    let mut show_form = use_signal(|| false);
    let connection_manager = use_signal(ConnectionManager::new);
    let config_storage = use_signal(|| ConfigStorage::new().ok());

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
            background: "#252526",
            color: "white",

            Sidebar {
                connections: connections(),
                on_add_connection: move |_| show_form.set(true),
                on_select_connection: move |id: Uuid| {
                    tracing::info!("Selected connection: {}", id);
                },
            }

            div {
                flex: "1",
                display: "flex",
                align_items: "center",
                justify_content: "center",

                if show_form() {
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
                } else {
                    div {
                        color: "#888",
                        font_size: "24px",

                        "Select a connection or create a new one"
                    }
                }
            }
        }
    }
}
