use crate::connection::{ConnectionConfig, ConnectionMode, SSHConfig};
use dioxus::prelude::*;

#[component]
pub fn ConnectionForm(
    editing_config: Option<ConnectionConfig>,
    on_save: EventHandler<ConnectionConfig>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| {
        editing_config
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_default()
    });
    let mut host = use_signal(|| {
        editing_config
            .as_ref()
            .map(|c| c.host.clone())
            .unwrap_or("127.0.0.1".to_string())
    });
    let mut port = use_signal(|| editing_config.as_ref().map(|c| c.port).unwrap_or(6379));
    let mut password = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.password.clone())
            .unwrap_or_default()
    });
    let username = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.username.clone())
            .unwrap_or_default()
    });
    let mut db = use_signal(|| editing_config.as_ref().map(|c| c.db).unwrap_or(0));
    let mut mode = use_signal(|| {
        editing_config
            .as_ref()
            .map(|c| c.mode.clone())
            .unwrap_or(ConnectionMode::Direct)
    });
    let mut enable_ssh = use_signal(|| {
        editing_config
            .as_ref()
            .map(|c| c.ssh.is_some())
            .unwrap_or(false)
    });

    let is_editing = editing_config.is_some();
    let title = if is_editing {
        "Edit Connection"
    } else {
        "New Connection"
    };

    rsx! {
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
                width: "450px",
                max_height: "90vh",
                overflow_y: "auto",
                padding: "24px",
                background: "#1e1e1e",
                border_radius: "8px",
                box_shadow: "0 4px 24px rgba(0, 0, 0, 0.5)",

                onclick: move |evt| {
                    evt.stop_propagation();
                },

                h2 {
                    color: "white",
                    margin_bottom: "20px",

                    "{title}"
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
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div {
                    display: "flex",
                    gap: "8px",
                    margin_bottom: "16px",

                    div {
                        flex: "2",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "Host"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{host}",
                            oninput: move |e| host.set(e.value()),
                        }
                    }

                    div {
                        flex: "1",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "Port"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            r#type: "number",
                            value: "{port}",
                            oninput: move |e| {
                                if let Ok(p) = e.value().parse() {
                                    port.set(p);
                                }
                            },
                        }
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        margin_bottom: "8px",

                        "Password"
                    }

                    input {
                        width: "100%",
                        padding: "8px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        r#type: "password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        margin_bottom: "8px",

                        "Database (0-15)"
                    }

                    input {
                        width: "100%",
                        padding: "8px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        r#type: "number",
                        min: "0",
                        max: "15",
                        value: "{db}",
                        oninput: move |e| {
                            if let Ok(d) = e.value().parse::<u8>() {
                                db.set(d.min(15));
                            }
                        },
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        margin_bottom: "8px",

                        "Connection Mode"
                    }

                    select {
                        width: "100%",
                        padding: "8px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        value: "{mode_display(mode())}",
                        onchange: move |e| {
                            let new_mode = match e.value().as_str() {
                                "Direct" => ConnectionMode::Direct,
                                "Cluster" => ConnectionMode::Cluster,
                                "Sentinel" => ConnectionMode::Sentinel,
                                _ => ConnectionMode::Direct,
                            };
                            mode.set(new_mode);
                        },

                        option { value: "Direct", "Direct" }
                        option { value: "Cluster", "Cluster" }
                        option { value: "Sentinel", "Sentinel" }
                    }
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
                            checked: enable_ssh(),
                            onchange: move |e| enable_ssh.set(e.checked()),
                        }

                        "Enable SSH Tunnel"
                    }
                }

                div {
                    display: "flex",
                    gap: "8px",
                    margin_top: "20px",

                    button {
                        flex: "1",
                        padding: "10px",
                        background: "#0e639c",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        onclick: move |_| {
                            let id = editing_config.as_ref().map(|c| c.id).unwrap_or_else(|| uuid::Uuid::new_v4());

                            let mut config = ConnectionConfig::new(name(), host(), port());
                            config.id = id;
                            config.username = if username.read().is_empty() { None } else { Some(username()) };
                            config.password = if password.read().is_empty() { None } else { Some(password()) };
                            config.db = db();
                            config.mode = mode();

                            if enable_ssh() {
                                config.ssh = Some(SSHConfig::default());
                            }

                            on_save.call(config);
                        },

                        if is_editing { "💾 Update" } else { "💾 Save" }
                    }

                    button {
                        flex: "1",
                        padding: "10px",
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
}

fn mode_display(mode: ConnectionMode) -> String {
    match mode {
        ConnectionMode::Direct => "Direct".to_string(),
        ConnectionMode::Cluster => "Cluster".to_string(),
        ConnectionMode::Sentinel => "Sentinel".to_string(),
    }
}
