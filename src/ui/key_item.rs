use crate::redis::TreeNode;
use dioxus::prelude::*;

#[component]
pub fn KeyItem(
    node: TreeNode,
    depth: usize,
    selected_key: String,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let is_selected = selected_key == node.full_path;
    let has_children = !node.children.is_empty();
    let mut is_expanded = use_signal(|| false);

    let indent = depth * 16;
    let icon = if node.is_leaf {
        match node.key_info.as_ref().map(|k| &k.key_type) {
            Some(crate::redis::KeyType::String) => "📝",
            Some(crate::redis::KeyType::Hash) => "📦",
            Some(crate::redis::KeyType::List) => "📋",
            Some(crate::redis::KeyType::Set) => "📁",
            Some(crate::redis::KeyType::ZSet) => "📊",
            Some(crate::redis::KeyType::Stream) => "🌊",
            _ => "📄",
        }
    } else {
        if is_expanded() {
            "📂"
        } else {
            "📁"
        }
    };

    rsx! {
        div {
            key: "{node.full_path}",

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
                        on_select.call(node.full_path.clone());
                    } else {
                        is_expanded.toggle();
                        on_toggle.call(node.full_path.clone());
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

                span { "{icon}" }

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
                        key: "{child.full_path}",
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
