use crate::connection::{
    ClusterConfig, ConnectionConfig, ConnectionMode, ConnectionPool, SSHConfig, SSLConfig,
};
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::icons::IconX;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub enum TestResult {
    #[default]
    None,
    Testing,
    Success(String),
    Failed(String),
}

#[component]
pub fn ConnectionForm(
    editing_config: Option<ConnectionConfig>,
    colors: ThemeColors,
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
    let mut cluster_nodes = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.cluster.as_ref())
            .map(|c| c.nodes.join("\n"))
            .unwrap_or_else(|| "127.0.0.1:6379".to_string())
    });
    let mut ssh_host = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssh.as_ref())
            .map(|s| s.host.clone())
            .unwrap_or_default()
    });
    let mut ssh_port = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssh.as_ref())
            .map(|s| s.port)
            .unwrap_or(22)
    });
    let mut ssh_username = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssh.as_ref())
            .map(|s| s.username.clone())
            .unwrap_or_default()
    });
    let mut ssh_password = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssh.as_ref())
            .and_then(|s| s.password.clone())
            .unwrap_or_default()
    });
    let mut ssh_key_path = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssh.as_ref())
            .and_then(|s| s.private_key_path.clone())
            .unwrap_or_default()
    });
    let mut enable_ssl = use_signal(|| {
        editing_config
            .as_ref()
            .map(|c| c.ssl.enabled)
            .unwrap_or(false)
    });
    let mut ssl_ca_cert_path = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssl.ca_cert_path.clone())
            .unwrap_or_default()
    });
    let mut ssl_client_cert_path = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssl.cert_path.clone())
            .unwrap_or_default()
    });
    let mut ssl_client_key_path = use_signal(|| {
        editing_config
            .as_ref()
            .and_then(|c| c.ssl.key_path.clone())
            .unwrap_or_default()
    });
    let mut test_result = use_signal(TestResult::default);
    let mut readonly = use_signal(|| editing_config.as_ref().map(|c| c.readonly).unwrap_or(false));

    let is_editing = editing_config.is_some();
    let title = if is_editing {
        "编辑连接"
    } else {
        "新建连接"
    };

    let editing_config_id = editing_config.as_ref().map(|c| c.id);

    let build_config = move |editing_config_id: Option<uuid::Uuid>,
                             name: String,
                             host: String,
                             port: u16,
                             username: String,
                             password: String,
                             mode: ConnectionMode,
                             enable_ssh: bool,
                             ssh_host: String,
                             ssh_port: u16,
                             ssh_username: String,
                             ssh_password: String,
                             ssh_key_path: String,
                             cluster_nodes: String,
                             enable_ssl: bool,
                             ssl_ca_cert_path: String,
                             ssl_client_cert_path: String,
                             ssl_client_key_path: String,
                             readonly: bool| {
        let id = editing_config_id.unwrap_or_else(|| uuid::Uuid::new_v4());

        let mut config = ConnectionConfig::new(name, host, port);
        config.id = id;
        config.username = if username.is_empty() {
            None
        } else {
            Some(username)
        };
        config.password = if password.is_empty() {
            None
        } else {
            Some(password)
        };
        config.db = 0;
        config.mode = mode.clone();
        config.connection_timeout = if mode == ConnectionMode::Cluster {
            15000
        } else {
            5000
        };
        config.readonly = readonly;

        if enable_ssh {
            config.ssh = Some(SSHConfig {
                host: ssh_host,
                port: ssh_port,
                username: ssh_username,
                password: if ssh_password.is_empty() {
                    None
                } else {
                    Some(ssh_password)
                },
                private_key_path: if ssh_key_path.is_empty() {
                    None
                } else {
                    Some(ssh_key_path)
                },
                passphrase: None,
                encrypted_password: None,
                encrypted_passphrase: None,
            });
        }

        config.ssl = SSLConfig {
            enabled: enable_ssl,
            ca_cert_path: if ssl_ca_cert_path.is_empty() {
                None
            } else {
                Some(ssl_ca_cert_path)
            },
            cert_path: if ssl_client_cert_path.is_empty() {
                None
            } else {
                Some(ssl_client_cert_path)
            },
            key_path: if ssl_client_key_path.is_empty() {
                None
            } else {
                Some(ssl_client_key_path)
            },
        };

        if mode == ConnectionMode::Cluster {
            let nodes: Vec<String> = cluster_nodes
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            config.cluster = Some(ClusterConfig {
                nodes,
                read_from_replicas: false,
            });
        }

        config
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "450px".to_string(),

            div {
                position: "relative",

                button {
                    position: "absolute",
                    top: "-8px",
                    right: "-8px",
                    z_index: "10",
                    padding: "4px",
                    background: "transparent",
                    border: "none",
                    cursor: "pointer",
                    color: "{colors.text_secondary}",
                    onclick: move |_| on_cancel.call(()),

                    IconX { size: Some(18) }
                }

                h2 {
                    color: "{colors.text}",
                    margin_bottom: "20px",
                    font_size: "20px",

                    "{title}"
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "13px",
                        margin_bottom: "8px",

                        "名称"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        box_sizing: "border_box",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                if mode() != ConnectionMode::Cluster {
                    div {
                        display: "flex",
                        gap: "8px",
                        margin_bottom: "16px",

                        div {
                            flex: "2",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "13px",
                                margin_bottom: "8px",

                                "Host"
                            }

                            input {
                                width: "100%",
                                padding: "8px 12px",
                                background: "{colors.background_tertiary}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "13px",
                                box_sizing: "border_box",
                                value: "{host}",
                                oninput: move |e| host.set(e.value()),
                            }
                        }

                        div {
                            flex: "1",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "13px",
                                margin_bottom: "8px",

                                "Port"
                            }

                            input {
                                width: "100%",
                                padding: "8px 12px",
                                background: "{colors.background_tertiary}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "13px",
                                box_sizing: "border_box",
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
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "13px",
                        margin_bottom: "8px",

                        "密码"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        box_sizing: "border_box",
                        r#type: "password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "8px",

                        "连接模式"
                    }

                    div {
                        display: "flex",
                        flex_wrap: "wrap",
                        gap: "8px",

                        for (mode_name, mode_val) in [("Direct", ConnectionMode::Direct), ("Cluster", ConnectionMode::Cluster), ("Sentinel", ConnectionMode::Sentinel)] {
                            {
                                let is_selected = mode() == mode_val;
                                let bg = if is_selected { colors.primary.clone() } else { colors.background_tertiary.clone() };
                                let border_color = if is_selected { colors.primary.clone() } else { colors.border.clone() };
                                let text_color = if is_selected { colors.primary_text.clone() } else { colors.text.clone() };
                                rsx! {
                                    div {
                                        key: "{mode_name}",
                                        padding: "6px 14px",
                                        background: "{bg}",
                                        border: "1px solid {border_color}",
                                        border_radius: "16px",
                                        color: "{text_color}",
                                        font_size: "13px",
                                        cursor: "pointer",
                                        user_select: "none",
                                        onclick: {
                                            let mv = mode_val.clone();
                                            move |_| mode.set(mv.clone())
                                        },

                                        "{mode_name}"
                                    }
                                }
                            }
                        }
                    }
                }

                if mode() == ConnectionMode::Cluster {
                    div {
                        margin_bottom: "16px",

                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "13px",
                            margin_bottom: "8px",

                            "集群节点 (每行一个 host:port)"
                        }

                        textarea {
                            width: "100%",
                            height: "80px",
                            padding: "8px 12px",
                            background: "{colors.background_tertiary}",
                            border: "1px solid {colors.border}",
                            border_radius: "4px",
                            color: "{colors.text}",
                            font_size: "13px",
                            font_family: "monospace",
                            box_sizing: "border_box",
                            resize: "vertical",
                            value: "{cluster_nodes}",
                            oninput: move |e| cluster_nodes.set(e.value()),
                        }
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        color: "{colors.text}",
                        font_size: "13px",
                        cursor: "pointer",

                        input {
                            r#type: "checkbox",
                            checked: readonly(),
                            onchange: move |e| readonly.set(e.checked()),
                        }

                        "只读模式 (禁止写操作)"
                    }

                    if readonly() {
                        div {
                            margin_top: "4px",
                            margin_left: "24px",
                            color: "{colors.text_secondary}",
                            font_size: "11px",

                            "启用后将阻止所有写命令执行 (SET, DEL, HSET, LPUSH 等)"
                        }
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        color: "{colors.text}",
                        font_size: "13px",
                        cursor: "pointer",

                        input {
                            r#type: "checkbox",
                            checked: enable_ssh(),
                            onchange: move |e| enable_ssh.set(e.checked()),
                        }

                        "启用 SSH 隧道"
                    }
                }

                if enable_ssh() {
                    div {
                        margin_bottom: "16px",
                        padding: "12px",
                        background: "{colors.background_tertiary}",
                        border_radius: "4px",

                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            margin_bottom: "12px",
                            font_weight: "500",
                            "SSH 配置"
                        }

                        div {
                            display: "flex",
                            gap: "8px",
                            margin_bottom: "8px",

                            div {
                                flex: "2",

                                label {
                                    display: "block",
                                    color: "{colors.text_secondary}",
                                    font_size: "11px",
                                    margin_bottom: "4px",
                                    "SSH Host"
                                }

                                input {
                                    width: "100%",
                                    padding: "6px 10px",
                                    background: "{colors.background}",
                                    border: "1px solid {colors.border}",
                                    border_radius: "4px",
                                    color: "{colors.text}",
                                    font_size: "12px",
                                    box_sizing: "border_box",
                                    value: "{ssh_host}",
                                    oninput: move |e| ssh_host.set(e.value()),
                                }
                            }

                            div {
                                flex: "1",

                                label {
                                    display: "block",
                                    color: "{colors.text_secondary}",
                                    font_size: "11px",
                                    margin_bottom: "4px",
                                    "SSH Port"
                                }

                                input {
                                    width: "100%",
                                    padding: "6px 10px",
                                    background: "{colors.background}",
                                    border: "1px solid {colors.border}",
                                    border_radius: "4px",
                                    color: "{colors.text}",
                                    font_size: "12px",
                                    box_sizing: "border_box",
                                    r#type: "number",
                                    value: "{ssh_port}",
                                    oninput: move |e| {
                                        if let Ok(p) = e.value().parse() {
                                            ssh_port.set(p);
                                        }
                                    },
                                }
                            }
                        }

                        div {
                            margin_bottom: "8px",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "11px",
                                margin_bottom: "4px",
                                "SSH 用户名"
                            }

                            input {
                                width: "100%",
                                padding: "6px 10px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "12px",
                                box_sizing: "border_box",
                                value: "{ssh_username}",
                                oninput: move |e| ssh_username.set(e.value()),
                            }
                        }

                        div {
                            margin_bottom: "8px",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "11px",
                                margin_bottom: "4px",
                                "SSH 密码"
                            }

                            input {
                                width: "100%",
                                padding: "6px 10px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "12px",
                                box_sizing: "border_box",
                                r#type: "password",
                                value: "{ssh_password}",
                                oninput: move |e| ssh_password.set(e.value()),
                            }
                        }

                        div {

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "11px",
                                margin_bottom: "4px",
                                "私钥路径 (可选)"
                            }

                            input {
                                width: "100%",
                                padding: "6px 10px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "12px",
                                box_sizing: "border_box",
                                placeholder: "~/.ssh/id_rsa",
                                value: "{ssh_key_path}",
                                oninput: move |e| ssh_key_path.set(e.value()),
                            }
                        }
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        color: "{colors.text}",
                        font_size: "13px",
                        cursor: "pointer",

                        input {
                            r#type: "checkbox",
                            checked: enable_ssl(),
                            onchange: move |e| enable_ssl.set(e.checked()),
                        }

                        "启用 SSL/TLS"
                    }
                }

                if enable_ssl() {
                    div {
                        margin_bottom: "16px",
                        padding: "12px",
                        background: "{colors.background_tertiary}",
                        border_radius: "4px",

                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            margin_bottom: "12px",
                            font_weight: "500",
                            "SSL/TLS 配置"
                        }

                        div {
                            margin_bottom: "8px",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "11px",
                                margin_bottom: "4px",
                                "CA 证书路径 (可选)"
                            }

                            input {
                                width: "100%",
                                padding: "6px 10px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "12px",
                                box_sizing: "border_box",
                                placeholder: "/path/to/ca.crt",
                                value: "{ssl_ca_cert_path}",
                                oninput: move |e| ssl_ca_cert_path.set(e.value()),
                            }
                        }

                        div {
                            margin_bottom: "8px",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "11px",
                                margin_bottom: "4px",
                                "客户端证书路径 (可选)"
                            }

                            input {
                                width: "100%",
                                padding: "6px 10px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "12px",
                                box_sizing: "border_box",
                                placeholder: "/path/to/client.crt",
                                value: "{ssl_client_cert_path}",
                                oninput: move |e| ssl_client_cert_path.set(e.value()),
                            }
                        }

                        div {

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "11px",
                                margin_bottom: "4px",
                                "客户端密钥路径 (可选)"
                            }

                            input {
                                width: "100%",
                                padding: "6px 10px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "4px",
                                color: "{colors.text}",
                                font_size: "12px",
                                box_sizing: "border_box",
                                placeholder: "/path/to/client.key",
                                value: "{ssl_client_key_path}",
                                oninput: move |e| ssl_client_key_path.set(e.value()),
                            }
                        }
                    }
                }

                if let TestResult::Success(msg) = test_result() {
                    div {
                        margin_top: "12px",
                        padding: "8px 12px",
                        background: "rgba(34, 197, 94, 0.1)",
                        border: "1px solid rgba(34, 197, 94, 0.3)",
                        border_radius: "4px",
                        color: "#22c55e",
                        font_size: "12px",

                        "✓ 连接成功: {msg}"
                    }
                }

                if let TestResult::Failed(msg) = test_result() {
                    div {
                        margin_top: "12px",
                        padding: "8px 12px",
                        background: "rgba(239, 68, 68, 0.1)",
                        border: "1px solid rgba(239, 68, 68, 0.3)",
                        border_radius: "4px",
                        color: "#ef4444",
                        font_size: "12px",

                        "✗ 连接失败: {msg}"
                    }
                }

                div {
                    display: "flex",
                    gap: "8px",
                    margin_top: "20px",

                    button {
                        flex: "1",
                        padding: "8px",
                        background: "{colors.accent}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: if matches!(test_result(), TestResult::Testing) { "wait" } else { "pointer" },
                        font_size: "13px",
                        disabled: matches!(test_result(), TestResult::Testing),
                        onclick: move |_| {
                            let config = build_config(
                                editing_config_id,
                                name(),
                                host(),
                                port(),
                                username(),
                                password(),
                                mode(),
                                enable_ssh(),
                                ssh_host(),
                                ssh_port(),
                                ssh_username(),
                                ssh_password(),
                                ssh_key_path(),
                                cluster_nodes(),
                                enable_ssl(),
                                ssl_ca_cert_path(),
                                ssl_client_cert_path(),
                                ssl_client_key_path(),
                                readonly(),
                            );
                            test_result.set(TestResult::Testing);

                            spawn(async move {
                                tracing::info!("Starting connection test...");

                                let result = tokio::time::timeout(
                                    std::time::Duration::from_secs(15),
                                    async {
                                        let pool = ConnectionPool::new(config).await
                                            .map_err(|e| format!("连接失败: {}", e))?;
                                        pool.ping().await
                                            .map_err(|e| format!("PING 失败: {}", e))
                                    }
                                ).await;

                                tracing::info!("Connection test completed: {:?}", result);

                                match result {
                                    Ok(Ok(response)) => {
                                        tracing::info!("Test success: {}", response);
                                        test_result.set(TestResult::Success(response));
                                    }
                                    Ok(Err(e)) => {
                                        tracing::error!("Test error: {}", e);
                                        test_result.set(TestResult::Failed(e));
                                    }
                                    Err(_) => {
                                        tracing::error!("Test timeout");
                                        test_result.set(TestResult::Failed("连接超时 (15秒)".to_string()));
                                    }
                                }
                            });
                        },

                        if matches!(test_result(), TestResult::Testing) { "测试中..." } else { "测试连接" }
                    }

                    button {
                        flex: "1",
                        padding: "8px",
                        background: "{colors.primary}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| {
                            let config = build_config(
                                editing_config_id,
                                name(),
                                host(),
                                port(),
                                username(),
                                password(),
                                mode(),
                                enable_ssh(),
                                ssh_host(),
                                ssh_port(),
                                ssh_username(),
                                ssh_password(),
                                ssh_key_path(),
                                cluster_nodes(),
                                enable_ssl(),
                                ssl_ca_cert_path(),
                                ssl_client_cert_path(),
                                ssl_client_key_path(),
                                readonly(),
                            );
                            on_save.call(config);
                        },

                        if is_editing { "更新" } else { "保存" }
                    }
                }
            }
        }
    }
}
