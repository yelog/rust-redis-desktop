use crate::redis::TreeNode;
use crate::ui::icons::*;
use dioxus::prelude::*;

#[component]
pub fn KeyItem(
    node: TreeNode,
    depth: usize,
    selected_key: String,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let is_selected = node.is_leaf && selected_key == node.path;
    let has_children = !node.children.is_empty();
    let mut is_expanded = use_signal(|| false);

    let indent = depth * 16;

    rsx! {
        div {
            key: "{node.node_id}",

            div {
                padding: "6px 8px",
                padding_left: "{indent}px",
                display: "flex",
                align_items: "center",
                gap: "6px",
                background: if is_selected { "#094771" } else { "transparent" },
                cursor: "pointer",
                onmouseenter: |_| {},
                onclick: move |_| {
                    if node.is_leaf {
                        on_select.call(node.path.clone());
                    } else {
                        is_expanded.toggle();
                        on_toggle.call(node.node_id.clone());
                    }
                },

                if !node.is_leaf && has_children {
                    span {
                        font_size: "12px",
                        color: "#888",
                        if is_expanded() { "▼" } else { "▶" }
                    }
                } else {
                    span { width: "12px" }
                }

                if node.is_leaf {
                    match node.key_info.as_ref().map(|k| &k.key_type) {
                        Some(crate::redis::KeyType::String) => rsx! { IconFile { size: Some(14) } },
                        Some(crate::redis::KeyType::Hash) => rsx! { IconHash { size: Some(14) } },
                        Some(crate::redis::KeyType::List) => rsx! { IconList { size: Some(14) } },
                        Some(crate::redis::KeyType::Set) => rsx! { IconSet { size: Some(14) } },
                        Some(crate::redis::KeyType::ZSet) => rsx! { IconZSet { size: Some(14) } },
                        Some(crate::redis::KeyType::Stream) => rsx! { IconStream { size: Some(14) } },
                        _ => rsx! { IconFile { size: Some(14) } },
                    }
                } else {
                    IconFolder { size: Some(14) }
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

            if is_expanded() && has_children {
                for child in node.children.iter() {
                    KeyItem {
                        key: "{child.node_id}",
                        node: child.clone(),
                        depth: depth + 1,
                        selected_key: selected_key.clone(),
                        on_select: on_select.clone(),
                        on_toggle: on_toggle.clone(),
                    }
                }
            }
        }
    }
}
