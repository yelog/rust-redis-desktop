use crate::connection::ConnectionPool;
use crate::ui::icons::*;
use dioxus::prelude::*;

#[component]
pub fn TTLEditor(
    connection_pool: ConnectionPool,
    key: String,
    current_ttl: String,
    has_ttl: bool,
    on_save: EventHandler<String>,
    on_persist: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut ttl_value = use_signal(|| current_ttl.clone());
    let mut no_expiry = use_signal(|| !has_ttl);
    let mut saving = use_signal(|| false);

    rsx! {
        div {
            padding: "16px",
            background: "#252526",
            border_radius: "8px",

            h3 {
                color: "white",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "6px",

                IconRefresh { size: Some(16) }
                " TTL Settings"
            }

            div {
                margin_bottom: "16px",

                label {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    color: "white",
                    cursor: "pointer",

                    input {
                        r#type: "checkbox",
                        checked: no_expiry(),
                        onchange: move |e| no_expiry.set(e.checked()),
                    }

                    "No expiry (persist)"
                }
            }

            if !no_expiry() {
                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        margin_bottom: "8px",

                        "TTL (seconds)"
                    }

                    input {
                        width: "100%",
                        padding: "8px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        r#type: "number",
                        min: "1",
                        value: "{ttl_value}",
                        oninput: move |e| ttl_value.set(e.value()),
                    }
                }
            }

            div {
                display: "flex",
                gap: "8px",

                button {
                    flex: "1",
                    padding: "8px",
                    background: "#0e639c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    disabled: saving(),
                    onclick: {
                        let pool = connection_pool.clone();
                        let key = key.clone();
                        move |_| {
                            let pool = pool.clone();
                            let key = key.clone();
                            let no_exp = no_expiry();
                            let ttl = ttl_value();

                            spawn(async move {
                                saving.set(true);

                                if no_exp {
                                    match pool.remove_ttl(&key).await {
                                        Ok(_) => on_persist.call(()),
                                        Err(e) => tracing::error!("Failed to remove TTL: {}", e),
                                    }
                                } else {
                                    match pool.set_ttl(&key, ttl.parse::<i64>().unwrap_or(3600)).await {
                                        Ok(_) => on_save.call(ttl),
                                        Err(e) => tracing::error!("Failed to set TTL: {}", e),
                                    }
                                }

                                saving.set(false);
                            });
                        }
                    },

                    if saving() { "Saving..." } else { "💾 Save" }
                }

                button {
                    flex: "1",
                    padding: "8px",
                    background: "#5a5a5a",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    onclick: move |_| on_cancel.call(()),

                    div {
                    display: "flex",
                    align_items: "center",
                    gap: "4px",

                    IconX { size: Some(12) }
                    " Cancel"
                }
                }
            }
        }
    }
}
