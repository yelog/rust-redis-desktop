use crate::connection::ConnectionState;
use crate::theme::ThemeColors;
use crate::ui::icons::*;
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[component]
pub fn Sidebar(
    width: f64,
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
    let mut context_menu = use_signal(|| None::<(Uuid, (i32, i32))>);
    let mut hover_edit = use_signal(|| false);
    let mut hover_delete = use_signal(|| false);
    let mut hover_reconnect = use_signal(|| false);
    let mut hover_close = use_signal(|| false);
    let mut hover_flush = use_signal(|| false);
    let mut hover_connection = use_signal(|| None::<Uuid>);

    rsx! {
        style { {r#"
            @keyframes pulse {
                0%, 100% { transform: scale(1); opacity: 1; }
                50% { transform: scale(1.3); opacity: 0.7; }
            }
        "#} }
        div {
            width: "{width}px",
            height: "100%",
            background: "{colors.background}",
            padding: "16px",
            display: "flex",
            flex_direction: "column",
            box_sizing: "border-box",
            overflow: "hidden",

            div {
                display: "flex",
                justify_content: "space_between",
                align_items: "center",
                margin_bottom: "12px",

                span {
                    color: "{colors.text_secondary}",
                    font_size: "12px",

                    if connections.is_empty() {
                        "No connections"
                    } else {
                        "{connections.len()} connection(s)"
                    }
                }
            }

            button {
                onclick: move |_| on_add_connection.call(()),
                background: "{colors.primary}",
                color: "{colors.primary_text}",
                border: "none",
                padding: "10px",
                border_radius: "4px",
                cursor: "pointer",
                margin_bottom: "16px",

                "+ New Connection"
            }

            div {
                flex: "1",
                overflow_y: "auto",

                for (id, name) in connections {
                    {
                        let state = connection_states.get(&id).copied().unwrap_or(ConnectionState::Disconnected);
                        let (dot_color, is_pulsing) = match state {
                            ConnectionState::Connected => (colors.state_connected, false),
                            ConnectionState::Disconnected => (colors.state_disconnected, false),
                            ConnectionState::Connecting => (colors.state_connecting, true),
                            ConnectionState::Error => (colors.state_error, false),
                        };

                        rsx! {
                            div {
                                key: "{id}",
                                padding: "10px 14px",
                                margin_bottom: "2px",
                                margin_left: if selected_connection == Some(id) { "4px" } else { "8px" },
                                background: if selected_connection == Some(id) {
                                    colors.background_tertiary
                                } else if hover_connection() == Some(id) {
                                    colors.background_tertiary
                                } else {
                                    colors.background_secondary
                                },
                                border_radius: "4px",
                                color: "{colors.text}",
                                position: "relative",
                                border_left: if selected_connection == Some(id) {
                                    "2px solid {colors.accent}"
                                } else {
                                    "2px solid transparent"
                                },
                                transition: "all 0.15s ease",

                                onmouseenter: {
                                    let id = id;
                                    move |_| hover_connection.set(Some(id))
                                },
                                onmouseleave: move |_| hover_connection.set(None),

                                oncontextmenu: {
                                    let id = id;
                                    move |evt: Event<MouseData>| {
                                        evt.prevent_default();
                                        let coords = evt.data().client_coordinates();
                                        let x = coords.x as i32;
                                        let y = coords.y as i32;
                                        context_menu.set(Some((id, (x, y))));
                                    }
                                },

                                div {
                                    onclick: {
                                        let id = id;
                                        move |_| {
                                            context_menu.set(None);
                                            on_select_connection.call(id)
                                        }
                                    },
                                    cursor: "pointer",
                                    display: "flex",
                                    align_items: "center",
                                    gap: "10px",

                                    div {
                                        width: "8px",
                                        height: "8px",
                                        border_radius: "50%",
                                        background: "{dot_color}",
                                        flex_shrink: "0",
                                        box_shadow: "0 0 4px {dot_color}",
                                        animation: if is_pulsing { "pulse 1.2s ease-in-out infinite" } else { "none" },
                                    }

                                    span {
                                        font_size: "13px",
                                        font_weight: if selected_connection == Some(id) { "500" } else { "400" },
                                        color: if selected_connection == Some(id) {
                                            colors.accent
                                        } else {
                                            colors.text
                                        },

                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                margin_top: "12px",
                padding_top: "12px",
                border_top: "1px solid {colors.border}",

                button {
                    width: "100%",
                    padding: "10px",
                    background: "{colors.background_secondary}",
                    color: "{colors.text_secondary}",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    gap: "8px",
                    onclick: move |_| on_open_settings.call(()),

                    onmouseenter: move |evt: Event<MouseData>| {
                        let target = evt.data().client_coordinates();
                        let _ = target;
                    },

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "4px",

                        IconSettings { size: Some(14) }
                        " 设置"
                    }
                }
            }
        }

        if let Some((menu_id, (x, y))) = context_menu() {
            div {
                position: "fixed",
                left: "{x}px",
                top: "{y}px",
                background: "{colors.background_secondary}",
                border: "1px solid {colors.border}",
                border_radius: "4px",
                box_shadow: "0 4px 12px rgba(0, 0, 0, 0.4)",
                z_index: "1000",
                min_width: "120px",
                padding: "4px 0",

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "{colors.text}",
                    font_size: "13px",
                    background: if hover_reconnect() { colors.success } else { "transparent" },
                    display: "flex",
                    align_items: "center",
                    gap: "6px",

                    onmouseenter: move |_| hover_reconnect.set(true),
                    onmouseleave: move |_| hover_reconnect.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_reconnect_connection.call(menu_id);
                        }
                    },

                    IconRefresh { size: Some(14) }
                    "Reconnect"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "{colors.text}",
                    font_size: "13px",
                    background: if hover_close() { colors.warning } else { "transparent" },
                    display: "flex",
                    align_items: "center",
                    gap: "6px",

                    onmouseenter: move |_| hover_close.set(true),
                    onmouseleave: move |_| hover_close.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_close_connection.call(menu_id);
                        }
                    },

                    IconX { size: Some(14) }
                    "Close"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "{colors.error}",
                    font_size: "13px",
                    background: if hover_flush() { colors.error } else { "transparent" },
                    display: "flex",
                    align_items: "center",
                    gap: "6px",

                    onmouseenter: move |_| hover_flush.set(true),
                    onmouseleave: move |_| hover_flush.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_flush_connection.call(menu_id);
                        }
                    },

                    IconAlert { size: Some(14) }
                    "FlushDB"
                }

                div {
                    height: "1px",
                    background: "{colors.border}",
                    margin: "4px 0",
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "{colors.text}",
                    font_size: "13px",
                    background: if hover_edit() { colors.primary } else { "transparent" },
                    display: "flex",
                    align_items: "center",
                    gap: "6px",

                    onmouseenter: move |_| hover_edit.set(true),
                    onmouseleave: move |_| hover_edit.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_edit_connection.call(menu_id);
                        }
                    },

                    IconEdit { size: Some(14) }
                    "Edit"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "{colors.text}",
                    font_size: "13px",
                    background: if hover_delete() { colors.error } else { "transparent" },
                    display: "flex",
                    align_items: "center",
                    gap: "6px",

                    onmouseenter: move |_| hover_delete.set(true),
                    onmouseleave: move |_| hover_delete.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_delete_connection.call(menu_id);
                        }
                    },

                    IconTrash { size: Some(14) }
                    "Delete"
                }
            }

            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                z_index: "999",

                onclick: move |_| context_menu.set(None),
            }
        }
    }
}
