use crate::connection::{ConnectionConfig, ConnectionMode, SSHConfig, SSLConfig, SentinelConfig};
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
    let mut username = use_signal(|| String::new());
    let mut db = use_signal(|| 0u8);
    let mut mode = use_signal(|| ConnectionMode::Direct);
    let mut enable_ssh = use_signal(|| false);
    let mut ssh_host = use_signal(|| String::new());

    rsx! {
        div {
            width: "450px",
            padding: "24px",
            background: "#1e1e1e",
            border_radius: "8px",

            h2 {
                color: "white",
                margin_bottom: "20px",

                "New Connection"
            }

            // Name
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

            // Host & Port
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

            // Password
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

            // Database
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

            // Connection Mode
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

            // SSH Tunnel
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

            // Action buttons
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
                        let mut config = ConnectionConfig::new(name(), host(), port());

                        config.username = if username.read().is_empty() { None } else { Some(username()) };
                        config.password = if password.read().is_empty() { None } else { Some(password()) };
                        config.db = db();
                        config.mode = mode();

                        if enable_ssh() && !ssh_host.read().is_empty() {
                            config.ssh = Some(SSHConfig {
                                host: ssh_host(),
                                ..Default::default()
                            });
                        }

                        on_save.call(config);
                    },

                    "💾 Save"
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

fn mode_display(mode: ConnectionMode) -> String {
    match mode {
        ConnectionMode::Direct => "Direct".to_string(),
        ConnectionMode::Cluster => "Cluster".to_string(),
        ConnectionMode::Sentinel => "Sentinel".to_string(),
    }
}
