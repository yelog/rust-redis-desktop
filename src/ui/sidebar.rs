use crate::connection::ConnectionState;
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[component]
pub fn Sidebar(
    width: f64,
    connections: Vec<(Uuid, String)>,
    connection_states: HashMap<Uuid, ConnectionState>,
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
            background: "#1e1e1e",
            padding: "16px",
            display: "flex",
            flex_direction: "column",
            box_sizing: "border-box",
            overflow: "hidden",

            div {
                display: "flex",
                justify_content: "space-between",
                align_items: "center",
                margin_bottom: "12px",

                span {
                    color: "#888",
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
                background: "#007acc",
                color: "white",
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
                            ConnectionState::Connected => ("#4ec9b0", false),
                            ConnectionState::Disconnected => ("#888888", false),
                            ConnectionState::Connecting => ("#f59e0b", true),
                            ConnectionState::Error => ("#ef4444", false),
                        };

                        rsx! {
                            div {
                                key: "{id}",
                                padding: "10px",
                                margin_bottom: "4px",
                                background: "#2d2d2d",
                                border_radius: "4px",
                                color: "white",
                                position: "relative",

                                oncontextmenu: {
                                    let id = id;
                                    move |evt: Event<MouseData>| {
                                        evt.prevent_default();
                                        let coords = evt.data().client_coordinates();
                                        context_menu.set(Some((id, (coords.x as i32, coords.y as i32))));
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
                                    ondoubleclick: {
                                        let id = id;
                                        move |_| {
                                            context_menu.set(None);
                                            on_reconnect_connection.call(id)
                                        }
                                    },
                                    cursor: "pointer",
                                    display: "flex",
                                    align_items: "center",
                                    gap: "8px",

                                    div {
                                        width: "8px",
                                        height: "8px",
                                        border_radius: "50%",
                                        background: "{dot_color}",
                                        flex_shrink: "0",
                                        animation: if is_pulsing { "pulse 1.2s ease-in-out infinite" } else { "none" },
                                    }

                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }

            div {
                margin_top: "12px",
                padding_top: "12px",
                border_top: "1px solid #3c3c3c",

                button {
                    width: "100%",
                    padding: "10px",
                    background: "#2d2d2d",
                    color: "#888",
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

                    "⚙️ 设置"
                }
            }
        }

        if let Some((menu_id, (x, y))) = context_menu() {
            div {
                position: "fixed",
                left: "{x}px",
                top: "{y}px",
                background: "#2d2d2d",
                border: "1px solid #3c3c3c",
                border_radius: "4px",
                box_shadow: "0 4px 12px rgba(0, 0, 0, 0.4)",
                z_index: "1000",
                min_width: "120px",
                padding: "4px 0",

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "white",
                    font_size: "13px",
                    background: if hover_reconnect() { "#2d7d46" } else { "transparent" },

                    onmouseenter: move |_| hover_reconnect.set(true),
                    onmouseleave: move |_| hover_reconnect.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_reconnect_connection.call(menu_id);
                        }
                    },

                    "🔄 Reconnect"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "white",
                    font_size: "13px",
                    background: if hover_close() { "#d97706" } else { "transparent" },

                    onmouseenter: move |_| hover_close.set(true),
                    onmouseleave: move |_| hover_close.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_close_connection.call(menu_id);
                        }
                    },

                    "✖️ Close"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "#f87171",
                    font_size: "13px",
                    background: if hover_flush() { "#c53030" } else { "transparent" },

                    onmouseenter: move |_| hover_flush.set(true),
                    onmouseleave: move |_| hover_flush.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_flush_connection.call(menu_id);
                        }
                    },

                    "⚠️ Flush Data"
                }

                div {
                    height: "1px",
                    background: "#3c3c3c",
                    margin: "4px 0",
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "white",
                    font_size: "13px",
                    background: if hover_edit() { "#3182ce" } else { "transparent" },

                    onmouseenter: move |_| hover_edit.set(true),
                    onmouseleave: move |_| hover_edit.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_edit_connection.call(menu_id);
                        }
                    },

                    "✏️ Edit"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: "white",
                    font_size: "13px",
                    background: if hover_delete() { "#c53030" } else { "transparent" },

                    onmouseenter: move |_| hover_delete.set(true),
                    onmouseleave: move |_| hover_delete.set(false),

                    onclick: {
                        let menu_id = menu_id;
                        move |_| {
                            context_menu.set(None);
                            on_delete_connection.call(menu_id);
                        }
                    },

                    "🗑️ Delete"
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
