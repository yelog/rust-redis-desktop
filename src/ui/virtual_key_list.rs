use crate::redis::TreeNode;
use dioxus::prelude::*;

#[component]
pub fn VirtualKeyList(
    nodes: Vec<TreeNode>,
    selected_key: String,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let mut scroll_offset = use_signal(|| 0i32);
    let item_height = 28i32;
    let visible_count = 30i32;
    let overscan = 5i32;

    let total_items = nodes.len() as i32;
    let start_index = ((scroll_offset() / item_height) - overscan).max(0) as usize;
    let end_index = (start_index + (visible_count + overscan * 2) as usize).min(nodes.len());

    rsx! {
        div {
            height: "100%",
            overflow_y: "auto",
            onscroll: move |e| {
                let data = e.data();
                scroll_offset.set(data.scroll_top() as i32);
            },

            div {
                height: "{total_items * item_height}px",
                position: "relative",

                for (idx, node) in nodes.iter().enumerate().skip(start_index).take(end_index - start_index) {
                    {
                        let top = idx as i32 * item_height;
                        let is_selected = selected_key == node.full_path;
                        let icon = if node.is_leaf { "📄" } else { "📁" };

                        rsx! {
                            div {
                                key: "{node.full_path}",
                                position: "absolute",
                                top: "{top}px",
                                left: "0",
                                right: "0",
                                height: "{item_height}px",
                                padding: "6px 8px",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                background: if is_selected { "#094771" } else { "transparent" },
                                cursor: "pointer",
                                overflow: "hidden",

                                onclick: {
                                    let full_path = node.full_path.clone();
                                    move |_| {
                                        on_select.call(full_path.clone());
                                    }
                                },

                                span {
                                    color: "#888",
                                    font_size: "12px",

                                    if !node.is_leaf { "▶" } else { "" }
                                }

                                span {
                                    color: "#888",

                                    "{icon}"
                                }

                                span {
                                    color: if is_selected { "white" } else { "#cccccc" },
                                    font_size: "13px",
                                    overflow: "hidden",
                                    text_overflow: "ellipsis",
                                    white_space: "nowrap",

                                    "{node.name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
