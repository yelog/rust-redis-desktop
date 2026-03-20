use crate::connection::{ConnectionConfig, ConnectionManager};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn Sidebar(
    connections: Vec<(Uuid, String)>,
    on_add_connection: EventHandler<()>,
    on_select_connection: EventHandler<Uuid>,
) -> Element {
    rsx! {
        div {
            width: "250px",
            height: "100vh",
            background: "#1e1e1e",
            padding: "16px",
            display: "flex",
            flex_direction: "column",

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
                        onclick: move |_| on_select_connection.call(id),
                        padding: "10px",
                        margin_bottom: "4px",
                        background: "#2d2d2d",
                        border_radius: "4px",
                        cursor: "pointer",
                        color: "white",

                        "{name}"
                    }
                }
            }
        }
    }
}
