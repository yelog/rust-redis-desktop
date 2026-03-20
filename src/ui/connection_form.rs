use crate::connection::{ConnectionConfig, ConnectionMode, SSHConfig, SSLConfig, SentinelConfig};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum ConfigTab {
    Basic,
    SSH,
    SSL,
    Sentinel,
}

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
    let mut current_tab = use_signal(|| ConfigTab::Basic);

    // SSH
    let mut ssh_host = use_signal(|| "127.0.0.1".to_string());
    let mut ssh_port = use_signal(|| 22u16);
    let mut ssh_user = use_signal(|| "root".to_string());
    let mut ssh_password = use_signal(|| String::new());
    let mut ssh_key_path = use_signal(|| String::new());

    // SSL
    let mut ssl_enabled = use_signal(|| false);
    let mut ssl_ca_cert = use_signal(|| String::new());

    // Sentinel
    let mut sentinel_master = use_signal(|| "mymaster".to_string());
    let mut sentinel_nodes = use_signal(|| "127.0.0.1:26379".to_string());

    rsx! {
        div {
            width: "500px",
            max_height: "80vh",
            overflow_y: "auto",
            padding: "24px",
            background: "#1e1e1e",
            border_radius: "8px",

            h2 {
                color: "white",
                margin_bottom: "20px",

                "New Connection"
            }

            // Tab bar
            div {
                display: "flex",
                border_bottom: "1px solid #3c3c3c",
                margin_bottom: "16px",

                for tab in [ConfigTab::Basic, ConfigTab::SSH, ConfigTab::SSL, ConfigTab::Sentinel] {
                    button {
                        padding: "8px 16px",
                        background: if current_tab() == tab { "#2d2d2d" } else { "transparent" },
                        color: if current_tab() == tab { "white" } else { "#888" },
                        border: "none",
                        border_bottom: if current_tab() == tab { "2px solid #4ec9b0" } else { "none" },
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| current_tab.set(tab),

                        "{tab_name(tab)}"
                    }
                }
            }

            // Basic tab
            if current_tab() == ConfigTab::Basic {
                div {
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

                    // Username
                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "Username (Redis 6.0+ ACL)"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{username}",
                            oninput: move |e| username.set(e.value()),
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

                            "Database"
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

                        div {
                            display: "flex",
                            gap: "8px",

                            for m in [ConnectionMode::Direct, ConnectionMode::Cluster, ConnectionMode::Sentinel] {
                                button {
                                    flex: "1",
                                    padding: "8px",
                                    background: if mode() == m { "#0e639c" } else { "#3c3c3c" },
                                    color: "white",
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    onclick: move |_| mode.set(m),

                                    "{mode_name(m)}"
                                }
                            }
                        }
                    }
                }
            }

            // SSH tab
            if current_tab() == ConfigTab::SSH {
                div {
                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "SSH Host"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{ssh_host}",
                            oninput: move |e| ssh_host.set(e.value()),
                        }
                    }

                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "SSH Port"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            r#type: "number",
                            value: "{ssh_port}",
                            oninput: move |e| {
                                if let Ok(p) = e.value().parse() {
                                    ssh_port.set(p);
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

                            "SSH Username"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{ssh_user}",
                            oninput: move |e| ssh_user.set(e.value()),
                        }
                    }

                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "SSH Password"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            r#type: "password",
                            value: "{ssh_password}",
                            oninput: move |e| ssh_password.set(e.value()),
                        }
                    }

                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "Private Key Path (optional)"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            placeholder: "~/.ssh/id_rsa",
                            value: "{ssh_key_path}",
                            oninput: move |e| ssh_key_path.set(e.value()),
                        }
                    }
                }
            }

            // SSL tab
            if current_tab() == ConfigTab::SSL {
                div {
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
                                checked: ssl_enabled(),
                                onchange: move |e| ssl_enabled.set(e.checked()),
                            }

                            "Enable SSL/TLS"
                        }
                    }

                    if ssl_enabled() {
                        div {
                            margin_bottom: "16px",

                            label {
                                display: "block",
                                color: "#888",
                                margin_bottom: "8px",

                                "CA Certificate Path (optional)"
                            }

                            input {
                                width: "100%",
                                padding: "8px",
                                background: "#3c3c3c",
                                border: "1px solid #555",
                                border_radius: "4px",
                                color: "white",
                                value: "{ssl_ca_cert}",
                                oninput: move |e| ssl_ca_cert.set(e.value()),
                            }
                        }
                    }
                }
            }

            // Sentinel tab
            if current_tab() == ConfigTab::Sentinel {
                div {
                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "Master Name"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{sentinel_master}",
                            oninput: move |e| sentinel_master.set(e.value()),
                        }
                    }

                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "#888",
                            margin_bottom: "8px",

                            "Sentinel Nodes (comma-separated)"
                        }

                        input {
                            width: "100%",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            placeholder: "127.0.0.1:26379,127.0.0.1:26380",
                            value: "{sentinel_nodes}",
                            oninput: move |e| sentinel_nodes.set(e.value()),
                        }
                    }
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

                        config.username = if username().is_empty() { None } else { Some(username()) };
                        config.password = if password().is_empty() { None } else { Some(password()) };
                        config.db = db();
                        config.mode = mode();

                        // SSH
                        if !ssh_host().is_empty() {
                            config.ssh = Some(SSHConfig {
                                host: ssh_host(),
                                port: ssh_port(),
                                username: ssh_user(),
                                password: if ssh_password().is_empty() { None } else { Some(ssh_password()) },
                                private_key_path: if ssh_key_path().is_empty() { None } else { Some(ssh_key_path()) },
                                passphrase: None,
                            });
                        }

                        // SSL
                        config.ssl = SSLConfig {
                            enabled: ssl_enabled(),
                            ca_cert_path: if ssl_ca_cert().is_empty() { None } else { Some(ssl_ca_cert()) },
                            cert_path: None,
                            key_path: None,
                        };

                        // Sentinel
                        if mode() == ConnectionMode::Sentinel {
                            let nodes: Vec<String> = sentinel_nodes()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();

                            config.sentinel = Some(SentinelConfig {
                                master_name: sentinel_master(),
                                nodes,
                                password: None,
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

fn tab_name(tab: ConfigTab) -> &'static str {
    match tab {
        ConfigTab::Basic => "Basic",
        ConfigTab::SSH => "SSH",
        ConfigTab::SSL => "SSL/TLS",
        ConfigTab::Sentinel => "Sentinel",
    }
}

fn mode_name(mode: ConnectionMode) -> &'static str {
    match mode {
        ConnectionMode::Direct => "Direct",
        ConnectionMode::Cluster => "Cluster",
        ConnectionMode::Sentinel => "Sentinel",
    }
}
