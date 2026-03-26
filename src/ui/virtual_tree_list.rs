use crate::redis::{KeyType, TreeNode};
use crate::ui::{FlatNode, FlatTreeAdapter};
use dioxus::prelude::*;
use std::collections::HashSet;

const DEFAULT_ITEM_HEIGHT: f32 = 28.0;
const DEFAULT_OVERSCAN: usize = 5;

#[derive(Clone, Copy, PartialEq)]
pub enum KeyTypeIcon {
    String,
    Hash,
    List,
    Set,
    ZSet,
    Stream,
    JSON,
    None,
}

impl From<Option<KeyType>> for KeyTypeIcon {
    fn from(key_type: Option<KeyType>) -> Self {
        match key_type {
            Some(KeyType::String) => KeyTypeIcon::String,
            Some(KeyType::Hash) => KeyTypeIcon::Hash,
            Some(KeyType::List) => KeyTypeIcon::List,
            Some(KeyType::Set) => KeyTypeIcon::Set,
            Some(KeyType::ZSet) => KeyTypeIcon::ZSet,
            Some(KeyType::Stream) => KeyTypeIcon::Stream,
            Some(KeyType::JSON) => KeyTypeIcon::JSON,
            _ => KeyTypeIcon::None,
        }
    }
}

impl KeyTypeIcon {
    pub fn emoji(&self) -> &'static str {
        match self {
            KeyTypeIcon::String => "📄",
            KeyTypeIcon::Hash => "📑",
            KeyTypeIcon::List => "📋",
            KeyTypeIcon::Set => "📦",
            KeyTypeIcon::ZSet => "📊",
            KeyTypeIcon::Stream => "📜",
            KeyTypeIcon::JSON => "🔷",
            KeyTypeIcon::None => "📄",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            KeyTypeIcon::String => "#4ade80",
            KeyTypeIcon::Hash => "#60a5fa",
            KeyTypeIcon::List => "#fbbf24",
            KeyTypeIcon::Set => "#f472b6",
            KeyTypeIcon::ZSet => "#a78bfa",
            KeyTypeIcon::Stream => "#2dd4bf",
            KeyTypeIcon::JSON => "#f97316",
            KeyTypeIcon::None => "#9ca3af",
        }
    }
}

#[component]
pub fn VirtualTreeList(
    nodes: Vec<TreeNode>,
    selected_key: String,
    expanded_paths: Signal<HashSet<String>>,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let mut scroll_top = use_signal(|| 0.0f32);
    let viewport_height = use_signal(|| 600.0f32);
    let mut adapter = use_signal(|| FlatTreeAdapter::new(DEFAULT_ITEM_HEIGHT));
    let mut last_nodes_len = use_signal(|| 0usize);

    use_effect(move || {
        let expanded = expanded_paths.read().clone();
        let nodes_len = nodes.len();

        if nodes_len != last_nodes_len() || adapter.read().expanded_paths() != &expanded {
            adapter.write().set_expanded_paths(expanded);
            adapter.write().build_from_tree(&nodes);
            last_nodes_len.set(nodes_len);
        }
    });

    let adapter_read = adapter.read();
    let total_height = adapter_read.total_height();
    let (start, end) =
        adapter_read.get_visible_range(scroll_top(), viewport_height(), DEFAULT_OVERSCAN);
    let item_height = adapter_read.item_height();

    rsx! {
        div {
            height: "100%",
            overflow_y: "auto",
            onscroll: move |e| {
                let data = e.data();
                scroll_top.set(data.scroll_top() as f32);
            },

            div {
                height: "{total_height}px",
                position: "relative",

                for idx in start..end {
                    if let Some(node) = adapter_read.get_node_at_index(idx) {
                        {
                            let top = idx as f32 * item_height;
                            let is_selected = !node.is_folder && selected_key == node.path;
                            let indent = node.depth * 16 + 8;

                            rsx! {
                                VirtualTreeItem {
                                    key: "{node.id}",
                                    node: node.clone(),
                                    top: top,
                                    indent: indent,
                                    is_selected: is_selected,
                                    on_select: {
                                        let on_select = on_select.clone();
                                        let path = node.path.clone();
                                        move |_| {
                                            on_select.call(path.clone());
                                        }
                                    },
                                    on_toggle: {
                                        let on_toggle = on_toggle.clone();
                                        let path = node.path.clone();
                                        move |_| {
                                            on_toggle.call(path.clone());
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn VirtualTreeItem(
    node: FlatNode,
    top: f32,
    indent: usize,
    is_selected: bool,
    on_select: EventHandler<()>,
    on_toggle: EventHandler<()>,
) -> Element {
    let bg_color = if is_selected {
        "#094771"
    } else {
        "transparent"
    };
    let text_color = if is_selected { "white" } else { "#cccccc" };
    let key_type_icon: KeyTypeIcon = node.key_type.clone().into();
    let folder_icon = if node.is_folder {
        "📁"
    } else {
        key_type_icon.emoji()
    };

    rsx! {
        div {
            position: "absolute",
            top: "{top}px",
            left: "0",
            right: "0",
            height: "28px",
            padding: "6px 8px",
            padding_left: "{indent}px",
            display: "flex",
            align_items: "center",
            gap: "6px",
            background: bg_color,
            cursor: "pointer",
            overflow: "hidden",

            onclick: {
                let on_select = on_select.clone();
                let on_toggle = on_toggle.clone();
                let is_folder = node.is_folder;
                move |_| {
                    if is_folder {
                        on_toggle.call(());
                    } else {
                        on_select.call(());
                    }
                }
            },

            if node.is_folder {
                span {
                    color: "#888",
                    font_size: "12px",
                    if node.children_count > 0 {
                        "▶"
                    } else {
                        ""
                    }
                }
            }

            span {
                font_size: "14px",
                "{folder_icon}"
            }

            span {
                color: text_color,
                font_size: "13px",
                overflow: "hidden",
                text_overflow: "ellipsis",
                white_space: "nowrap",

                "{node.name}"
            }

            if node.is_folder && node.children_count > 0 {
                span {
                    color: "#888",
                    font_size: "11px",
                    "({node.children_count})"
                }
            }
        }
    }
}
