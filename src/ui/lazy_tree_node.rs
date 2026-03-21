use crate::redis::TreeNode;
use dioxus::prelude::*;
use std::collections::HashSet;

fn collect_leaf_paths(node: &TreeNode) -> Vec<String> {
    let mut paths = Vec::new();
    if node.is_leaf {
        paths.push(node.path.clone());
    }
    for child in &node.children {
        paths.extend(collect_leaf_paths(child));
    }
    paths
}

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
    let selection_mode = tree_state.read().selection_mode;
    let has_children = !node.children.is_empty();
    let indent = depth * 16;

    let leaf_paths = if node.is_leaf {
        vec![node.path.clone()]
    } else {
        collect_leaf_paths(&node)
    };
    let total_leaves = leaf_paths.len();
    let selected_leaves = leaf_paths
        .iter()
        .filter(|p| tree_state.read().selected_keys.contains(*p))
        .count();
    let is_checked = if node.is_leaf {
        selected_leaves > 0
    } else {
        selected_leaves == total_leaves && total_leaves > 0
    };
    let is_partial = !node.is_leaf && selected_leaves > 0 && selected_leaves < total_leaves;

    let icon = if node.is_leaf {
        match node.key_info.as_ref().map(|k| &k.key_type) {
            Some(crate::redis::KeyType::String) => "📝",
            Some(crate::redis::KeyType::Hash) => "📦",
            Some(crate::redis::KeyType::List) => "📋",
            Some(crate::redis::KeyType::Set) => "🧩",
            Some(crate::redis::KeyType::ZSet) => "📊",
            Some(crate::redis::KeyType::Stream) => "🌊",
            _ => "📄",
        }
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
                background: if is_selected { "#094771" } else if is_checked || is_partial { "#1a4a1a" } else { "transparent" },
                cursor: "pointer",

                onclick: {
                    let node_id = node.node_id.clone();
                    let node_path = node.path.clone();
                    let leaf_paths = leaf_paths.clone();
                    let is_leaf = node.is_leaf;
                    move |_| {
                        if selection_mode {
                            let mut state = tree_state.write();
                            let all_selected = leaf_paths.iter().all(|p| state.selected_keys.contains(p));
                            if all_selected {
                                for p in &leaf_paths {
                                    state.selected_keys.remove(p);
                                }
                            } else {
                                for p in &leaf_paths {
                                    state.selected_keys.insert(p.clone());
                                }
                            }
                        } else if !is_leaf {
                            on_expand.call(node_id.clone());
                        } else {
                            on_select.call(node_path.clone());
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
                    div {
                        width: "14px",
                        height: "14px",
                        border: "1px solid",
                        border_color: if is_partial { "#f59e0b" } else if is_checked { "#68d391" } else { "#666" },
                        border_radius: "3px",
                        background: if is_partial { "rgba(245, 158, 11, 0.2)" } else if is_checked { "rgba(104, 211, 145, 0.2)" } else { "transparent" },
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        cursor: "pointer",
                        flex_shrink: 0,
                        onclick: {
                            let leaf_paths = leaf_paths.clone();
                            move |e| {
                                e.stop_propagation();
                                let mut state = tree_state.write();
                                let all_selected = leaf_paths.iter().all(|p| state.selected_keys.contains(p));
                                if all_selected {
                                    for p in &leaf_paths {
                                        state.selected_keys.remove(p);
                                    }
                                } else {
                                    for p in &leaf_paths {
                                        state.selected_keys.insert(p.clone());
                                    }
                                }
                            }
                        },

                        if is_checked {
                            span {
                                color: "#68d391",
                                font_size: "11px",
                                font_weight: "bold",
                                line_height: "1",

                                "✓"
                            }
                        } else if is_partial {
                            span {
                                color: "#f59e0b",
                                font_size: "10px",
                                line_height: "1",

                                "─"
                            }
                        }
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
                    color: if is_selected { "white" } else if is_checked || is_partial { "#68d391" } else if node.name.is_empty() { "#f59e0b" } else { "#cccccc" },
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
