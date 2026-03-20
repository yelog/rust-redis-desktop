use crate::connection::ConnectionConfig;
use dioxus::prelude::*;

#[component]
pub fn ConnectionForm(
    on_save: EventHandler<ConnectionConfig>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut host = use_signal(|| "127.0.0.1".to_string());
    let mut port = use_signal(|| 6379u16);
    let mut password = use_signal(|| String::new());

    rsx! {
        div {
            padding: "24px",
            background: "#1e1e1e",
            border_radius: "8px",

            h2 {
                color: "white",
                margin_bottom: "24px",

                "New Connection"
            }

            div {
                margin_bottom: "16px",

                label {
                    display: "block",
                    color: "#888",
                    margin_bottom: "8px",

                    "Name"
                }

                input {
                    width: "100%",
                    padding: "8px",
                    background: "#2d2d2d",
                    border: "1px solid #444",
                    border_radius: "4px",
                    color: "white",
                    oninput: move |e| name.set(e.value()),
                    value: "{name}",
                }
            }

            div {
                margin_bottom: "16px",

                label {
                    display: "block",
                    color: "#888",
                    margin_bottom: "8px",

                    "Host"
                }

                input {
                    width: "100%",
                    padding: "8px",
                    background: "#2d2d2d",
                    border: "1px solid #444",
                    border_radius: "4px",
                    color: "white",
                    oninput: move |e| host.set(e.value()),
                    value: "{host}",
                }
            }

            div {
                margin_bottom: "16px",

                label {
                    display: "block",
                    color: "#888",
                    margin_bottom: "8px",

                    "Port"
                }

                input {
                    width: "100%",
                    padding: "8px",
                    background: "#2d2d2d",
                    border: "1px solid #444",
                    border_radius: "4px",
                    color: "white",
                    r#type: "number",
                    oninput: move |e| {
                        if let Ok(p) = e.value().parse() {
                            port.set(p);
                        }
                    },
                    value: "{port}",
                }
            }

            div {
                margin_bottom: "24px",

                label {
                    display: "block",
                    color: "#888",
                    margin_bottom: "8px",

                    "Password (optional)"
                }

                input {
                    width: "100%",
                    padding: "8px",
                    background: "#2d2d2d",
                    border: "1px solid #444",
                    border_radius: "4px",
                    color: "white",
                    r#type: "password",
                    oninput: move |e| password.set(e.value()),
                    value: "{password}",
                }
            }

            div {
                display: "flex",
                gap: "8px",

                button {
                    flex: "1",
                    padding: "10px",
                    background: "#007acc",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    onclick: move |_| {
                        let config = ConnectionConfig::new(name(), host(), port());
                        on_save.call(config);
                    },

                    "Save"
                }

                button {
                    flex: "1",
                    padding: "10px",
                    background: "#444",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    onclick: move |_| on_cancel.call(()),

                    "Cancel"
                }
            }
        }
    }
}
