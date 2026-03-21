use crate::redis::TreeNode;
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
pub struct TreeState {
    pub expanded_nodes: HashSet<String>,
    pub loaded_nodes: HashSet<String>,
    pub selected_keys: HashSet<String>,
    pub selection_mode: bool,
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            expanded_nodes: HashSet::new(),
            loaded_nodes: HashSet::new(),
            selected_keys: HashSet::new(),
            selection_mode: false,
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
    let is_expanded = tree_state.read().expanded_nodes.contains(&node.node_id);
    let is_selected = node.is_leaf && selected_key == node.path;
    let is_checked = tree_state.read().selected_keys.contains(&node.path);
    let selection_mode = tree_state.read().selection_mode;
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
            key: "{node.node_id}",

            div {
                padding: "6px 8px",
                padding_left: "{indent}px",
                display: "flex",
                align_items: "center",
                gap: "6px",
                background: if is_selected { "#094771" } else if is_checked { "#1a4a1a" } else { "transparent" },
                cursor: "pointer",

                onclick: {
                    let node_id = node.node_id.clone();
                    let path = node.path.clone();
                    move |_| {
                        if selection_mode {
                            let mut state = tree_state.write();
                            if state.selected_keys.contains(&path) {
                                state.selected_keys.remove(&path);
                            } else {
                                state.selected_keys.insert(path.clone());
                            }
                        } else if !node.is_leaf {
                            on_expand.call(node_id.clone());
                        } else {
                            on_select.call(path.clone());
                        }
                    }
                },

                oncontextmenu: {
                    let path = node.path.clone();
                    let is_leaf = node.is_leaf;
                    move |e| {
                        e.prevent_default();
                        let data = e.data();
                        let client_x = data.client_coordinates().x as i32;
                        let client_y = data.client_coordinates().y as i32;
                        context_menu.set(Some(ContextMenuState {
                            x: client_x,
                            y: client_y,
                            node_path: path.clone(),
                            is_leaf,
                        }));
                    }
                },

                if selection_mode {
                    input {
                        r#type: "checkbox",
                        checked: is_checked,
                        onclick: move |e| e.stop_propagation(),
                        onchange: {
                            let path = node.path.clone();
                            move |_| {
                                let mut state = tree_state.write();
                                if state.selected_keys.contains(&path) {
                                    state.selected_keys.remove(&path);
                                } else {
                                    state.selected_keys.insert(path.clone());
                                }
                            }
                        },
                    }
                }

                if !node.is_leaf && has_children {
                    span {
                        color: "#888",
                        font_size: "12px",
                        cursor: "pointer",
                        onclick: {
                            let node_id = node.node_id.clone();
                            move |e| {
                                e.stop_propagation();
                                on_expand.call(node_id.clone());
                            }
                        },

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
                    color: if is_selected { "white" } else if is_checked { "#68d391" } else if node.name.is_empty() { "#f59e0b" } else { "#cccccc" },
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
                        key: "{child.node_id}",
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
