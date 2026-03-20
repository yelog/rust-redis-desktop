use arboard::Clipboard;
use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use crate::redis::{TreeBuilder, TreeNode};
use crate::ui::lazy_tree_node::{LazyTreeNode, TreeState, ContextMenuState};
use crate::ui::context_menu::{ContextMenu, ContextMenuItem};
use crate::ui::delete_confirm_dialog::{DeleteConfirmDialog, DeleteTarget};
use crate::ui::add_key_dialog::AddKeyDialog;
use uuid::Uuid;

#[component]
pub fn KeyBrowser(
    connection_id: Uuid,
    connection_pool: ConnectionPool,
    selected_key: Signal<String>,
    on_key_select: EventHandler<String>,
) -> Element {
    let tree_nodes = use_signal(Vec::<TreeNode>::new);
    let mut search_pattern = use_signal(String::new);
    let loading = use_signal(|| false);
    let keys_count = use_signal(|| 0usize);
    let mut tree_state = use_signal(TreeState::default);
    let mut context_menu = use_signal(|| None::<ContextMenuState>);
    let mut show_delete_dialog = use_signal(|| None::<Vec<DeleteTarget>>);
    let mut refresh_trigger = use_signal(|| 0u32);
    let mut show_add_key_dialog = use_signal(|| false);

    let load_keys = {
        let pool = connection_pool.clone();
        let search_pattern = search_pattern.clone();
        let mut loading = loading.clone();
        let mut tree_nodes = tree_nodes.clone();
        let mut keys_count = keys_count.clone();
        move || {
            let pool = pool.clone();
            let pattern = if search_pattern.read().is_empty() {
                "*".to_string()
            } else {
                format!("*{}*", search_pattern.read())
            };

            spawn(async move {
                loading.set(true);
                tree_nodes.set(Vec::new());

                match pool.scan_keys(&pattern, 500).await {
                    Ok(keys) => {
                        keys_count.set(keys.len());

                        let builder = TreeBuilder::new(":");
                        let tree = builder.build(keys);
                        tree_nodes.set(tree);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load keys: {}", e);
                    }
                }

                loading.set(false);
            });
        }
    };

    use_effect({
        let load_keys = load_keys.clone();
        move || {
            let _ = refresh_trigger();
            load_keys();
        }
    });

    rsx! {
        div {
            width: "300px",
            height: "100%",
            background: "#252526",
            border_right: "1px solid #3c3c3c",
            display: "flex",
            flex_direction: "column",
            box_sizing: "border-box",

            div {
                padding: "8px",
                border_bottom: "1px solid #3c3c3c",

                input {
                    width: "100%",
                    padding: "6px 10px",
                    background: "#3c3c3c",
                    border: "1px solid #555",
                    border_radius: "4px",
                    color: "white",
                    font_size: "13px",
                    placeholder: "Search keys...",
                    value: "{search_pattern}",
                    oninput: move |e| search_pattern.set(e.value()),
                    onkeydown: {
                        let load_keys = load_keys.clone();
                        move |e| {
                            if e.data().key() == Key::Enter {
                                load_keys();
                            }
                        }
                    },
                }
            }

            div {
                padding: "8px",
                border_bottom: "1px solid #3c3c3c",
                display: "flex",
                gap: "8px",
                align_items: "center",

                button {
                    padding: "6px 12px",
                    background: "#0e639c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "12px",
                    onclick: move |_| show_add_key_dialog.set(true),

                    "➕ 新增"
                }

                button {
                    flex: "1",
                    padding: "6px",
                    background: "#3c3c3c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "12px",
                    onclick: {
                        let load_keys = load_keys.clone();
                        move |_| load_keys()
                    },

                    if loading() { "Loading..." } else { "🔄 Refresh" }
                }

                if keys_count() > 0 {
                    span {
                        color: "#888",
                        font_size: "11px",

                        "{keys_count} keys"
                    }
                }
            }

            div {
                flex: "1",
                overflow_y: "auto",
                padding: "4px 0",

                if tree_nodes.read().is_empty() {
                    if loading() {
                        div {
                            padding: "20px",
                            text_align: "center",
                            color: "#888",

                            "Loading keys..."
                        }
                    } else {
                        div {
                            padding: "20px",
                            text_align: "center",
                            color: "#888",

                            "No keys found"
                        }
                    }
                } else {
                    for node in tree_nodes.read().iter() {
                        LazyTreeNode {
                            key: "{node.full_path}",
                            node: node.clone(),
                            depth: 0,
                            selected_key: selected_key(),
                            tree_state: tree_state,
                            on_select: on_key_select.clone(),
                            on_expand: {
                                move |path: String| {
                                    let mut state = tree_state.write();
                                    if state.expanded_nodes.contains(&path) {
                                        state.expanded_nodes.remove(&path);
                                    } else {
                                        state.expanded_nodes.insert(path);
                                    }
                                }
                            },
                            context_menu: context_menu,
                        }
                    }
                }
            }
        }

        if let Some(menu_state) = context_menu() {
            ContextMenu {
                x: menu_state.x,
                y: menu_state.y,
                on_close: move |_| context_menu.set(None),

                ContextMenuItem {
                    icon: Some("📋".to_string()),
                    label: "复制Key".to_string(),
                    danger: false,
                    onclick: {
                        let menu_state = menu_state.clone();
                        move |_| {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(menu_state.node_path.clone());
                            }
                            context_menu.set(None);
                        }
                    },
                }

                ContextMenuItem {
                    icon: Some("🗑️".to_string()),
                    label: "删除".to_string(),
                    danger: true,
                    onclick: {
                        let menu_state = menu_state.clone();
                        move |_| {
                            context_menu.set(None);
                            show_delete_dialog.set(Some(vec![DeleteTarget {
                                key: menu_state.node_path.clone(),
                                is_folder: !menu_state.is_leaf,
                            }]));
                        }
                    },
                }
            }
        }

        if let Some(targets) = show_delete_dialog() {
            DeleteConfirmDialog {
                connection_pool: connection_pool.clone(),
                targets: targets.clone(),
                on_confirm: move |_| {
                    show_delete_dialog.set(None);
                    selected_key.set(String::new());
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                on_cancel: move |_| show_delete_dialog.set(None),
            }
        }

        if show_add_key_dialog() {
            AddKeyDialog {
                connection_pool: connection_pool.clone(),
                on_save: {
                    let mut refresh_trigger = refresh_trigger.clone();
                    let mut selected_key = selected_key.clone();
                    move |key: String| {
                        show_add_key_dialog.set(false);
                        selected_key.set(key);
                        refresh_trigger.set(refresh_trigger() + 1);
                    }
                },
                on_cancel: move |_| show_add_key_dialog.set(false),
            }
        }
    }
}