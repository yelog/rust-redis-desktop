use crate::connection::{ConnectionConfig, ConnectionManager};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn Sidebar(
    connections: Vec<(Uuid, String)>,
    on_add_connection: EventHandler<()>,
    on_select_connection: EventHandler<Uuid>,
    on_edit_connection: EventHandler<Uuid>,
    on_delete_connection: EventHandler<Uuid>,
) -> Element {
    let mut hover_id = use_signal(|| None::<Uuid>);

    rsx! {
        div {
            width: "250px",
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
                    div {
                        key: "{id}",
                        padding: "10px",
                        margin_bottom: "4px",
                        background: "#2d2d2d",
                        border_radius: "4px",
                        color: "white",
                        position: "relative",

                        onmouseenter: {
                            let id = id;
                            move |_| hover_id.set(Some(id))
                        },
                        onmouseleave: move |_| hover_id.set(None),

                        div {
                            onclick: {
                                let id = id;
                                move |_| on_select_connection.call(id)
                            },
                            cursor: "pointer",

                            "{name}"
                        }

                        if hover_id() == Some(id) {
                            div {
                                display: "flex",
                                gap: "4px",
                                margin_top: "8px",

                                button {
                                    flex: "1",
                                    padding: "4px 8px",
                                    background: "#3182ce",
                                    color: "white",
                                    border: "none",
                                    border_radius: "3px",
                                    cursor: "pointer",
                                    font_size: "12px",

                                    onclick: {
                                        let id = id;
                                        move |_| on_edit_connection.call(id)
                                    },

                                    "✏️ Edit"
                                }

                                button {
                                    flex: "1",
                                    padding: "4px 8px",
                                    background: "#c53030",
                                    color: "white",
                                    border: "none",
                                    border_radius: "3px",
                                    cursor: "pointer",
                                    font_size: "12px",

                                    onclick: {
                                        let id = id;
                                        move |_| on_delete_connection.call(id)
                                    },

                                    "🗑️ Delete"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
