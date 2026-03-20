use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use crate::redis::{TreeBuilder, TreeNode};
use uuid::Uuid;

#[component]
pub fn KeyBrowser(
    connection_id: Uuid,
    connection_pool: ConnectionPool,
    selected_key: Signal<String>,
    on_key_select: EventHandler<String>,
) -> Element {
    let mut tree_nodes = use_signal(Vec::<TreeNode>::new);
    let mut search_pattern = use_signal(String::new);
    let mut loading = use_signal(|| false);
    
    let load_keys = {
        let pool = connection_pool.clone();
        move || {
            let pool = pool.clone();
            let pattern = if search_pattern.read().is_empty() {
                "*".to_string()
            } else {
                format!("*{}*", search_pattern.read())
            };
            
            spawn(async move {
                loading.set(true);
                
                match pool.scan_keys(&pattern, 1000).await {
                    Ok(keys) => {
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
    
    // Load keys on mount
    {
        let load_keys = load_keys.clone();
        use_effect(move || {
            load_keys();
        });
    }
    
    rsx! {
        div {
            width: "300px",
            height: "100%",
            background: "#252526",
            border_right: "1px solid #3c3c3c",
            display: "flex",
            flex_direction: "column",
            
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
                
                button {
                    flex: "1",
                    padding: "6px",
                    background: "#0e639c",
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
                        crate::ui::KeyItem {
                            key: "{node.full_path}",
                            node: node.clone(),
                            depth: 0,
                            selected_key: selected_key(),
                            on_select: on_key_select.clone(),
                            on_toggle: move |_path: String| {},
                        }
                    }
                }
            }
        }
    }
}