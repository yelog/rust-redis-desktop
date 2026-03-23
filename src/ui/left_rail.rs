use crate::connection::ConnectionState;
use crate::theme::ThemeColors;
use crate::ui::icons::{IconEdit, IconPlus, IconRefresh, IconSettings, IconTrash, IconX};
use crate::ui::status_indicator::StatusIndicatorWithLabel;
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

fn state_label(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connected => "已连接",
        ConnectionState::Connecting => "连接中",
        ConnectionState::Disconnected => "未连接",
        ConnectionState::Error => "连接异常",
    }
}

#[component]
pub fn LeftRail(
    width: Signal<f64>,
    connections: Vec<(Uuid, String)>,
    connection_states: HashMap<Uuid, ConnectionState>,
    selected_connection: Option<Uuid>,
    colors: ThemeColors,
    on_add_connection: EventHandler<()>,
    on_select_connection: EventHandler<Uuid>,
    on_edit_connection: EventHandler<Uuid>,
    on_delete_connection: EventHandler<Uuid>,
    on_reconnect_connection: EventHandler<Uuid>,
    on_close_connection: EventHandler<Uuid>,
    on_flush_connection: EventHandler<Uuid>,
    on_open_settings: EventHandler<()>,
) -> Element {
    let has_connections = !connections.is_empty();
    let selected_name = selected_connection.and_then(|id| {
        connections
            .iter()
            .find(|(conn_id, _)| *conn_id == id)
            .map(|(_, name)| name.clone())
    });
    let selected_state = selected_connection
        .and_then(|id| connection_states.get(&id).copied())
        .unwrap_or(ConnectionState::Disconnected);

    rsx! {
        div {
            width: "{width()}px",
            height: "100%",
            background: "{colors.surface_lowest}",
            border_right: "1px solid {colors.border}",
            display: "flex",
            flex_direction: "column",
            overflow: "hidden",

            div {
                padding: "16px",
                display: "flex",
                flex_direction: "column",
                gap: "12px",
                border_bottom: "1px solid {colors.border}",

                div {
                    padding: "14px",
                    background: "{colors.background}",
                    border: "1px solid {colors.border}",
                    border_radius: "8px",
                    display: "flex",
                    flex_direction: "column",
                    gap: "8px",

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "10px",

                        StatusIndicatorWithLabel {
                            state: selected_state,
                            colors,
                            show_label: Some(false),
                            size: Some(10.0),
                        }

                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: "3px",

                            span {
                                color: "{colors.text}",
                                font_size: "14px",
                                font_weight: "700",

                                if let Some(name) = selected_name.as_ref() {
                                    "{name}"
                                } else {
                                    "未选择连接"
                                }
                            }

                            span {
                                color: "{colors.text_subtle}",
                                font_size: "11px",
                                text_transform: "uppercase",
                                letter_spacing: "0.12em",

                                "{state_label(selected_state)}"
                            }
                        }
                    }

                    if let Some(id) = selected_connection {
                        div {
                            display: "flex",
                            flex_wrap: "wrap",
                            gap: "8px",

                            button {
                                padding: "7px 10px",
                                background: "{colors.surface_high}",
                                color: "{colors.text}",
                                border: "1px solid {colors.border}",
                                border_radius: "6px",
                                cursor: "pointer",
                                font_size: "12px",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                onclick: move |_| on_edit_connection.call(id),

                                IconEdit { size: Some(13) }
                                "编辑"
                            }

                            button {
                                padding: "7px 10px",
                                background: "{colors.surface_high}",
                                color: "{colors.text}",
                                border: "1px solid {colors.border}",
                                border_radius: "6px",
                                cursor: "pointer",
                                font_size: "12px",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                onclick: move |_| on_reconnect_connection.call(id),

                                IconRefresh { size: Some(13) }
                                "重连"
                            }

                            button {
                                padding: "7px 10px",
                                background: "{colors.surface_high}",
                                color: "{colors.text}",
                                border: "1px solid {colors.border}",
                                border_radius: "6px",
                                cursor: "pointer",
                                font_size: "12px",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                onclick: move |_| on_close_connection.call(id),

                                IconX { size: Some(13) }
                                "断开"
                            }

                            button {
                                padding: "7px 10px",
                                background: "rgba(255, 180, 171, 0.08)",
                                color: "{colors.error}",
                                border: "1px solid {colors.error}",
                                border_radius: "6px",
                                cursor: "pointer",
                                font_size: "12px",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                onclick: move |_| on_delete_connection.call(id),

                                IconTrash { size: Some(13) }
                                "删除"
                            }
                        }

                        button {
                            padding: "7px 10px",
                            background: "transparent",
                            color: "{colors.text_secondary}",
                            border: "1px dashed {colors.outline_variant}",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| on_flush_connection.call(id),

                            "清空当前数据库"
                        }
                    }
                }

                button {
                    width: "100%",
                    padding: "10px 12px",
                    background: "{colors.primary}",
                    color: "{colors.primary_text}",
                    border: "none",
                    border_radius: "8px",
                    cursor: "pointer",
                    font_size: "13px",
                    font_weight: "700",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    gap: "8px",
                    onclick: move |_| on_add_connection.call(()),

                    IconPlus { size: Some(14) }
                    "新建连接"
                }
            }

            div {
                padding: "12px 16px 8px",
                color: "{colors.text_subtle}",
                font_size: "11px",
                text_transform: "uppercase",
                letter_spacing: "0.16em",

                "Connections"
            }

            div {
                flex: "1",
                overflow_y: "auto",
                padding: "0 12px 12px",
                display: "flex",
                flex_direction: "column",
                gap: "6px",

                for (id, name) in connections {
                    {
                        let state = connection_states
                            .get(&id)
                            .copied()
                            .unwrap_or(ConnectionState::Disconnected);
                        let is_selected = selected_connection == Some(id);
                        let dot_color = match state {
                            ConnectionState::Connected => colors.accent,
                            ConnectionState::Connecting => colors.warning,
                            ConnectionState::Disconnected => colors.text_subtle,
                            ConnectionState::Error => colors.error,
                        };

                        rsx! {
                            button {
                                padding: "12px",
                                background: if is_selected { colors.background } else { colors.surface_low },
                                color: if is_selected { colors.text } else { colors.text_secondary },
                                border: if is_selected {
                                    format!("1px solid {}", colors.border)
                                } else {
                                    "1px solid transparent".to_string()
                                },
                                border_radius: "8px",
                                cursor: "pointer",
                                text_align: "left",
                                display: "flex",
                                flex_direction: "column",
                                gap: "6px",
                                onclick: move |_| on_select_connection.call(id),

                                div {
                                    display: "flex",
                                    align_items: "center",
                                    gap: "8px",

                                    div {
                                        width: "8px",
                                        height: "8px",
                                        border_radius: "50%",
                                        background: "{dot_color}",
                                        flex_shrink: "0",
                                    }

                                    span {
                                        font_size: "13px",
                                        font_weight: if is_selected { "700" } else { "500" },
                                        color: if is_selected { colors.accent } else { colors.text },

                                        "{name}"
                                    }
                                }

                                span {
                                    color: "{colors.text_subtle}",
                                    font_size: "11px",

                                    "{state_label(state)}"
                                }
                            }
                        }
                    }
                }

                if selected_connection.is_none() && !has_connections {
                    div {
                        padding: "12px",
                        color: "{colors.text_subtle}",
                        font_size: "12px",
                        background: "{colors.surface_low}",
                        border_radius: "8px",

                        "还没有连接，先创建一个 Redis 连接。"
                    }
                }
            }

            div {
                padding: "12px",
                border_top: "1px solid {colors.border}",
                display: "flex",
                flex_direction: "column",
                gap: "8px",

                button {
                    padding: "10px 12px",
                    background: "{colors.surface_low}",
                    color: "{colors.text_secondary}",
                    border: "1px solid {colors.border}",
                    border_radius: "8px",
                    cursor: "pointer",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    font_size: "13px",
                    onclick: move |_| on_open_settings.call(()),

                    IconSettings { size: Some(14) }
                    "设置"
                }
            }
        }
    }
}
