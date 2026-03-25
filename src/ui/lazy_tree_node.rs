use crate::redis::TreeNode;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG_TERTIARY, COLOR_OUTLINE, COLOR_SUCCESS, COLOR_TEXT,
    COLOR_TEXT_SECONDARY, COLOR_WARNING,
};
use crate::ui::icons::*;
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

#[component]
pub fn LazyTreeNode(
    node: TreeNode,
    depth: usize,
    selected_key: String,
    tree_state: Signal<TreeState>,
    on_select: EventHandler<String>,
    on_expand: EventHandler<String>,
    context_menu: Signal<Option<(String, bool, (i32, i32))>>,
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
                background: if is_selected { COLOR_BG_TERTIARY } else if is_checked || is_partial { "rgba(48, 209, 88, 0.08)" } else { "transparent" },
                cursor: "pointer",
                border_left: if is_selected { "2px solid {COLOR_ACCENT}" } else { "2px solid transparent" },
                margin_left: if is_selected { "-2px" } else { "0" },
                transition: "background 150ms ease-out, border_color 150ms ease-out",

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
                        let path = path.clone();
                        context_menu.set(None);
                        spawn(async move {
                            context_menu.set(Some((path, is_leaf, (client_x, client_y))));
                        });
                    }
                },

                if selection_mode {
                    div {
                        width: "14px",
                        height: "14px",
                        border: "1px solid",
                        border_color: if is_partial { COLOR_WARNING } else if is_checked { COLOR_SUCCESS } else { COLOR_OUTLINE },
                        border_radius: "3px",
                        background: if is_partial { "rgba(255, 159, 10, 0.15)" } else if is_checked { "rgba(48, 209, 88, 0.15)" } else { "transparent" },
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
                                color: COLOR_SUCCESS,
                                font_size: "11px",
                                font_weight: "bold",
                                line_height: "1",

                                "✓"
                            }
                        } else if is_partial {
                            span {
                                color: COLOR_WARNING,
                                font_size: "10px",
                                line_height: "1",

                                "─"
                            }
                        }
                    }
                }

                if !node.is_leaf && has_children {
                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",
                        cursor: "pointer",
                        display: "inline_block",
                        transition: "transform 200ms ease-out",
                        transform: if is_expanded { "rotate(90deg)" } else { "rotate(0deg)" },
                        onclick: {
                            let node_id = node.node_id.clone();
                            move |e| {
                                e.stop_propagation();
                                on_expand.call(node_id.clone());
                            }
                        },

                        "▶"
                    }
                } else {
                    span { width: "12px" }
                }

                span {
                    color: COLOR_TEXT_SECONDARY,
                    display: "flex",
                    align_items: "center",

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
                }

                span {
                    color: if is_selected { COLOR_ACCENT } else if is_checked || is_partial { COLOR_SUCCESS } else if node.name.is_empty() { COLOR_WARNING } else { COLOR_TEXT },
                    font_size: "13px",
                    overflow: "hidden",
                    text_overflow: "ellipsis",
                    white_space: "nowrap",

                    "{display_name}"
                }

                if !node.is_leaf && node.total_keys > 0 {
                    span {
                        color: COLOR_OUTLINE,
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
