use crate::redis::TreeNode;
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
pub struct TreeState {
    pub expanded_nodes: HashSet<String>,
    pub loaded_nodes: HashSet<String>,
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            expanded_nodes: HashSet::new(),
            loaded_nodes: HashSet::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ContextMenuState {
    pub x: i32,
    pub y: i32,
    pub node_path: String,
    pub is_leaf: bool,
}

#[component]
pub fn LazyTreeNode(
    node: TreeNode,
    depth: usize,
    selected_key: String,
    tree_state: Signal<TreeState>,
    on_select: EventHandler<String>,
    on_expand: EventHandler<String>,
    context_menu: Signal<Option<ContextMenuState>>,
) -> Element {
    let is_expanded = tree_state.read().expanded_nodes.contains(&node.full_path);
    let is_selected = selected_key == node.full_path;
    let has_children = !node.children.is_empty();
    let indent = depth * 16;

    let icon = if node.is_leaf {
        "📄"
    } else {
        if is_expanded {
            "📂"
        } else {
            "📁"
        }
    };

    let display_name = if node.name.is_empty() {
        "[空]"
    } else {
        &node.name
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

                onclick: {
                    let full_path = node.full_path.clone();
                    move |_| {
                        if !node.is_leaf {
                            on_expand.call(full_path.clone());
                        } else {
                            on_select.call(full_path.clone());
                        }
                    }
                },

                oncontextmenu: {
                    let full_path = node.full_path.clone();
                    let is_leaf = node.is_leaf;
                    move |e| {
                        e.prevent_default();
                        let data = e.data();
                        let client_x = data.client_coordinates().x as i32;
                        let client_y = data.client_coordinates().y as i32;
                        context_menu.set(Some(ContextMenuState {
                            x: client_x,
                            y: client_y,
                            node_path: full_path.clone(),
                            is_leaf,
                        }));
                    }
                },

                if !node.is_leaf && has_children {
                    span {
                        color: "#888",
                        font_size: "12px",

                        if is_expanded { "▼" } else { "▶" }
                    }
                } else {
                    span { width: "12px" }
                }

                span {
                    color: "#888",

                    "{icon}"
                }

                span {
                    color: if is_selected { "white" } else if node.name.is_empty() { "#f59e0b" } else { "#cccccc" },
                    font_size: "13px",
                    overflow: "hidden",
                    text_overflow: "ellipsis",
                    white_space: "nowrap",

                    "{display_name}"
                }

                if !node.is_leaf && node.total_keys > 0 {
                    span {
                        color: "#666",
                        font_size: "11px",

                        "({node.total_keys})"
                    }
                }
            }

            if is_expanded && has_children {
                for child in node.children.iter() {
                    LazyTreeNode {
                        key: "{child.full_path}",
                        node: child.clone(),
                        depth: depth + 1,
                        selected_key: selected_key.clone(),
                        tree_state: tree_state,
                        on_select: on_select.clone(),
                        on_expand: on_expand.clone(),
                        context_menu: context_menu,
                    }
                }
            }
        }
    }
}
