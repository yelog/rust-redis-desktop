use crate::connection::ConnectionPool;
use dioxus::prelude::*;

#[component]
pub fn StringEditor(
    connection_pool: ConnectionPool,
    key: String,
    initial_value: String,
    on_save: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut value = use_signal(|| initial_value.clone());
    let mut saving = use_signal(|| false);

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100%",

            textarea {
                flex: "1",
                padding: "12px",
                background: "#1e1e1e",
                border: "1px solid #3c3c3c",
                border_radius: "4px",
                color: "white",
                font_family: "Consolas, 'Courier New', monospace",
                font_size: "14px",
                resize: "none",
                value: "{value}",
                oninput: move |e| value.set(e.value()),
            }

            div {
                display: "flex",
                gap: "8px",
                padding: "12px 0",

                button {
                    flex: "1",
                    padding: "8px",
                    background: "#0e639c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    disabled: saving(),
                    onclick: move |_| {
                        let pool = connection_pool.clone();
                        let key = key.clone();
                        let val = value();

                        spawn(async move {
                            saving.set(true);

                            match pool.set_string_value(&key, &val).await {
                                Ok(_) => {
                                    on_save.call(val);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to save: {}", e);
                                }
                            }

                            saving.set(false);
                        });
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

                    "✖ Cancel"
                }
            }
        }
    }
}
