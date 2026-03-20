use crate::connection::ConnectionPool;
use dioxus::prelude::*;

#[component]
pub fn KeyActions(
    connection_pool: ConnectionPool,
    key: String,
    on_delete: EventHandler<()>,
    on_rename: EventHandler<String>,
) -> Element {
    let mut show_delete_confirm = use_signal(|| false);
    let mut show_rename_dialog = use_signal(|| false);
    let mut new_key_name = use_signal(String::new);
    let mut processing = use_signal(|| false);

    rsx! {
        div {
            display: "flex",
            gap: "8px",

            button {
                padding: "4px 12px",
                background: "#c53030",
                color: "white",
                border: "none",
                border_radius: "4px",
                cursor: "pointer",
                font_size: "12px",
                onclick: move |_| show_delete_confirm.set(true),

                "🗑️ Delete"
            }

            button {
                padding: "4px 12px",
                background: "#805ad5",
                color: "white",
                border: "none",
                border_radius: "4px",
                cursor: "pointer",
                font_size: "12px",
                onclick: {
                    let key = key.clone();
                    move |_| {
                        new_key_name.set(key.clone());
                        show_rename_dialog.set(true);
                    }
                },

                "✏️ Rename"
            }
        }

        if show_delete_confirm() {
            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                background: "rgba(0, 0, 0, 0.7)",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                z_index: "1000",

                div {
                    background: "#252526",
                    padding: "24px",
                    border_radius: "8px",
                    max_width: "400px",

                    h3 {
                        color: "white",
                        margin_bottom: "16px",

                        "⚠️ Confirm Delete"
                    }

                    p {
                        color: "#888",
                        margin_bottom: "24px",

                        "Are you sure you want to delete '{key}'?"
                    }

                    div {
                        display: "flex",
                        gap: "8px",

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#c53030",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            disabled: processing(),
                            onclick: {
                                let pool = connection_pool.clone();
                                let key = key.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let key = key.clone();
                                    spawn(async move {
                                        processing.set(true);

                                        match pool.delete_key(&key).await {
                                            Ok(_) => {
                                                show_delete_confirm.set(false);
                                                on_delete.call(());
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to delete: {}", e);
                                            }
                                        }

                                        processing.set(false);
                                    });
                                }
                            },

                            if processing() { "Deleting..." } else { "🗑️ Delete" }
                        }

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#5a5a5a",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            onclick: move |_| show_delete_confirm.set(false),

                            "Cancel"
                        }
                    }
                }
            }
        }

        if show_rename_dialog() {
            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                background: "rgba(0, 0, 0, 0.7)",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                z_index: "1000",

                div {
                    background: "#252526",
                    padding: "24px",
                    border_radius: "8px",
                    max_width: "400px",

                    h3 {
                        color: "white",
                        margin_bottom: "16px",

                        "✏️ Rename Key"
                    }

                    input {
                        width: "100%",
                        padding: "8px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        margin_bottom: "16px",
                        value: "{new_key_name}",
                        oninput: move |e| new_key_name.set(e.value()),
                    }

                    div {
                        display: "flex",
                        gap: "8px",

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#805ad5",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            disabled: processing() || new_key_name.read().is_empty(),
                            onclick: {
                                let pool = connection_pool.clone();
                                let old_key = key.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let old_key = old_key.clone();
                                    let new_key = new_key_name();
                                    spawn(async move {
                                        processing.set(true);

                                        match pool.rename_key(&old_key, &new_key).await {
                                            Ok(_) => {
                                                show_rename_dialog.set(false);
                                                on_rename.call(new_key);
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to rename: {}", e);
                                            }
                                        }

                                        processing.set(false);
                                    });
                                }
                            },

                            if processing() { "Renaming..." } else { "✓ Rename" }
                        }

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#5a5a5a",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            onclick: move |_| show_rename_dialog.set(false),

                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}
