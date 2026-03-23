use crate::connection::{ConnectionConfig, ConnectionMode, SSHConfig};
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;

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

    let is_editing = editing_config.is_some();
    let title = if is_editing {
        "编辑连接"
    } else {
        "新建连接"
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "450px".to_string(),

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
                    font_size: "13px",
                    margin_bottom: "8px",

                    "连接模式"
                }

                div {
                    display: "flex",
                    gap: "16px",

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "6px",
                        color: "{colors.text}",
                        font_size: "13px",
                        cursor: "pointer",

                        input {
                            r#type: "radio",
                            name: "connection_mode",
                            checked: mode() == ConnectionMode::Direct,
                            onchange: move |_| mode.set(ConnectionMode::Direct),
                        }

                        "Direct"
                    }

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "6px",
                        color: "{colors.text}",
                        font_size: "13px",
                        cursor: "pointer",

                        input {
                            r#type: "radio",
                            name: "connection_mode",
                            checked: mode() == ConnectionMode::Cluster,
                            onchange: move |_| mode.set(ConnectionMode::Cluster),
                        }

                        "Cluster"
                    }

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "6px",
                        color: "{colors.text}",
                        font_size: "13px",
                        cursor: "pointer",

                        input {
                            r#type: "radio",
                            name: "connection_mode",
                            checked: mode() == ConnectionMode::Sentinel,
                            onchange: move |_| mode.set(ConnectionMode::Sentinel),
                        }

                        "Sentinel"
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

            div {
                    display: "flex",
                    gap: "8px",
                    margin_top: "20px",

                    button {
                        flex: "1",
                        padding: "8px",
                        background: "{colors.background_tertiary}",
                        color: "{colors.text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| on_cancel.call(()),

                        "取消"
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
                            let id = editing_config.as_ref().map(|c| c.id).unwrap_or_else(|| uuid::Uuid::new_v4());

                            let mut config = ConnectionConfig::new(name(), host(), port());
                            config.id = id;
                            config.username = if username.read().is_empty() { None } else { Some(username()) };
                            config.password = if password.read().is_empty() { None } else { Some(password()) };
                            config.db = 0;
                            config.mode = mode();

                            if enable_ssh() {
                                config.ssh = Some(SSHConfig::default());
                            }

                            on_save.call(config);
                        },

                        if is_editing { "更新" } else { "保存" }
                    }
                }
        }
    }
}
