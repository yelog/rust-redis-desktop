use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType};
use crate::ui::{StringEditor, TTLEditor, KeyActions};

#[component]
pub fn ValueViewer(
    connection_pool: ConnectionPool,
    selected_key: String,
    on_key_deleted: EventHandler<()>,
    on_key_renamed: EventHandler<String>,
) -> Element {
    let mut key_info = use_signal(|| None::<KeyInfo>);
    let mut value = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut editing = use_signal(|| false);
    let mut show_ttl_editor = use_signal(|| false);
    
    let key = selected_key.clone();
    let pool = connection_pool.clone();
    
    use_effect(move || {
        if key.is_empty() {
            return;
        }
        
        let pool = pool.clone();
        let key = key.clone();
        
        spawn(async move {
            loading.set(true);
            
            match pool.get_key_info(&key).await {
                Ok(info) => {
                    key_info.set(Some(info.clone()));
                    
                    let val = match info.key_type {
                        KeyType::String => pool.get_string_value(&key).await.ok(),
                        KeyType::List => {
                            match pool.get_list_range(&key, 0, 100).await {
                                Ok(items) => Some(format!("{:#?}", items)),
                                Err(_) => None,
                            }
                        }
                        KeyType::Set => {
                            match pool.get_set_members(&key).await {
                                Ok(members) => Some(format!("{:#?}", members)),
                                Err(_) => None,
                            }
                        }
                        KeyType::Hash => {
                            match pool.get_hash_all(&key).await {
                                Ok(fields) => {
                                    let json = serde_json::to_string_pretty(&fields).unwrap_or_default();
                                    Some(json)
                                }
                                Err(_) => None,
                            }
                        }
                        KeyType::ZSet => {
                            match pool.get_zset_range(&key, 0, 100).await {
                                Ok(members) => Some(format!("{:#?}", members)),
                                Err(_) => None,
                            }
                        }
                        _ => None,
                    };
                    
                    if let Some(v) = val {
                        value.set(v);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to load key info: {}", e);
                }
            }
            
            loading.set(false);
        });
    });
    
    rsx! {
        div {
            flex: "1",
            height: "100%",
            background: "#1e1e1e",
            display: "flex",
            flex_direction: "column",
            
            // Header
            div {
                padding: "12px 16px",
                border_bottom: "1px solid #3c3c3c",
                background: "#252526",
                
                if let Some(info) = key_info() {
                    div {
                        display: "flex",
                        justify_content: "space_between",
                        align_items: "center",
                        margin_bottom: "8px",
                        
                        div {
                            span {
                                color: "#888",
                                font_size: "12px",
                                margin_right: "8px",
                                
                                "Key:"
                            }
                            
                            span {
                                color: "#4ec9b0",
                                font_size: "14px",
                                font_weight: "bold",
                                
                                "{selected_key}"
                            }
                        }
                        
                        div {
                            display: "flex",
                            gap: "8px",
                            font_size: "12px",
                            color: "#888",
                            
                            span {
                                "Type: {info.key_type}"
                            }
                            
                            if let Some(ttl) = info.ttl {
                                span {
                                    "TTL: {ttl}s"
                                }
                            }
                        }
                    }
                    
                    // Action buttons
                    div {
                        display: "flex",
                        gap: "8px",
                        
                        if info.key_type == KeyType::String && !editing() {
                            button {
                                padding: "4px 12px",
                                background: "#0e639c",
                                color: "white",
                                border: "none",
                                border_radius: "4px",
                                cursor: "pointer",
                                font_size: "12px",
                                onclick: move |_| editing.set(true),
                                
                                "✏️ Edit"
                            }
                        }
                        
                        button {
                            padding: "4px 12px",
                            background: "#3182ce",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| show_ttl_editor.set(true),
                            
                            "⏱️ TTL"
                        }
                        
                        KeyActions {
                            connection_pool: connection_pool.clone(),
                            key: selected_key.clone(),
                            on_delete: move |_| on_key_deleted.call(()),
                            on_rename: move |new_key| on_key_renamed.call(new_key),
                        }
                    }
                } else {
                    div {
                        color: "#888",
                        
                        "Select a key to view"
                    }
                }
            }
            
            // Content
            div {
                flex: "1",
                overflow: "auto",
                padding: "16px",
                
                if loading() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "Loading..."
                    }
                } else if selected_key.is_empty() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "No key selected"
                    }
                } else if editing() {
                    if let Some(info) = key_info() {
                        if info.key_type == KeyType::String {
                            StringEditor {
                                connection_pool: connection_pool.clone(),
                                key: selected_key.clone(),
                                initial_value: value(),
                                on_save: move |new_value| {
                                    value.set(new_value);
                                    editing.set(false);
                                },
                                on_cancel: move |_| editing.set(false),
                            }
                        }
                    }
                } else if value().is_empty() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "No value"
                    }
                } else {
                    pre {
                        color: "#d4d4d4",
                        font_family: "Consolas, 'Courier New', monospace",
                        font_size: "13px",
                        line_height: "1.6",
                        white_space: "pre_wrap",
                        word_wrap: "break_word",
                        
                        "{value}"
                    }
                }
            }
        }
        
        // TTL Editor Dialog
        if show_ttl_editor() {
            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                background: "rgba(0, 0, 0, 0.7)",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                z_index: "1000",
                
                TTLEditor {
                    connection_pool: connection_pool.clone(),
                    key: selected_key.clone(),
                    current_ttl: key_info().and_then(|i| i.ttl).unwrap_or(3600).to_string(),
                    has_ttl: key_info().and_then(|i| i.ttl).is_some(),
                    on_save: move |ttl_str| {
                        if let Some(mut info) = key_info.write().as_mut() {
                            info.ttl = Some(ttl_str.parse::<i64>().unwrap_or(3600));
                        }
                        show_ttl_editor.set(false);
                    },
                    on_persist: move |_| {
                        if let Some(mut info) = key_info.write().as_mut() {
                            info.ttl = None;
                        }
                        show_ttl_editor.set(false);
                    },
                    on_cancel: move |_| show_ttl_editor.set(false),
                }
            }
        }
    }
}