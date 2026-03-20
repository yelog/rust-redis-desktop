use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType};
use crate::ui::editable_field::EditableField;
use std::collections::HashMap;

#[component]
pub fn ValueViewer(
    connection_pool: ConnectionPool,
    selected_key: Signal<String>,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut key_info = use_signal(|| None::<KeyInfo>);
    let mut string_value = use_signal(|| String::new());
    let mut hash_value = use_signal(|| HashMap::new());
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    
    let pool = connection_pool.clone();
    let pool_for_edit = connection_pool.clone();
    
    // Load data when key changes
    use_effect(move || {
        let key = selected_key.read().clone();
        
        if key.is_empty() {
            key_info.set(None);
            string_value.set(String::new());
            hash_value.set(HashMap::new());
            return;
        }
        
        let pool = pool.clone();
        
        spawn(async move {
            loading.set(true);
            tracing::info!("Loading key: {}", key);
            
            match pool.get_key_info(&key).await {
                Ok(info) => {
                    tracing::info!("Key info loaded: {:?}", info.key_type);
                    key_info.set(Some(info.clone()));
                    
                    match info.key_type {
                        KeyType::String => {
                            match pool.get_string_value(&key).await {
                                Ok(val) => {
                                    tracing::info!("String value loaded: {} bytes", val.len());
                                    string_value.set(val);
                                }
                                Err(e) => tracing::error!("Failed to load string: {}", e),
                            }
                        }
                        KeyType::Hash => {
                            match pool.get_hash_all(&key).await {
                                Ok(fields) => {
                                    tracing::info!("Hash loaded: {} fields", fields.len());
                                    hash_value.set(fields);
                                }
                                Err(e) => tracing::error!("Failed to load hash: {}", e),
                            }
                        }
                        _ => {
                            tracing::info!("Type: {:?}", info.key_type);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to load key info: {}", e);
                }
            }
            
            loading.set(false);
        });
    });
    
    let key_for_edit = selected_key;
    
    // Prepare data for render
    let info = key_info();
    let is_loading = loading();
    let str_val = string_value();
    let hash_val = hash_value();
    let display_key = selected_key.read().clone();
    
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
                
if let Some(ref info) = info {
                    div {
                        display: "flex",
                        justify_content: "space_between",
                        align_items: "center",
                        
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
                                
                                "{display_key}"
                            }
                        }
                        
                        div {
                            display: "flex",
                            gap: "16px",
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
                overflow_y: "auto",
                padding: "16px",
                
                if is_loading {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "Loading..."
                    }
                } else if display_key.is_empty() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "No key selected"
                    }
                } else if let Some(info) = info {
                    match info.key_type {
                        KeyType::String => {
                            rsx! {
                                EditableField {
                                    label: "Value".to_string(),
                                    value: str_val.clone(),
                                    editable: true,
                                    multiline: true,
                                    on_change: {
                                        let pool = pool_for_edit.clone();
                                        let key_sig = key_for_edit.clone();
                                        move |new_val: String| {
                                            let pool = pool.clone();
                                            let key = key_sig.read().clone();
                                            let val = new_val.clone();
                                            spawn(async move {
                                                saving.set(true);
                                                if pool.set_string_value(&key, &val).await.is_ok() {
                                                    string_value.set(val);
                                                    on_refresh.call(());
                                                }
                                                saving.set(false);
                                            });
                                        }
                                    },
                                }
                            }
                        }
                        KeyType::Hash => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",
                                    
                                    "Hash Fields ({hash_val.len()}):"
                                }
                                
                                for (field, value) in hash_val.iter() {
                                    div {
                                        key: "{field}",
                                        margin_bottom: "8px",
                                        padding: "8px",
                                        background: "#2d2d2d",
                                        border_radius: "4px",
                                        
                                        div {
                                            color: "#4ec9b0",
                                            font_size: "12px",
                                            margin_bottom: "4px",
                                            
                                            "{field}"
                                        }
                                        
                                        div {
                                            color: "white",
                                            
                                            "{value}"
                                        }
                                    }
                                }
                            }
                        }
                        KeyType::List => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",
                                    
                                    "List Items (use CLI to view)"
                                }
                            }
                        }
                        KeyType::Set => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",
                                    
                                    "Set Members (use CLI to view)"
                                }
                            }
                        }
                        KeyType::ZSet => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",
                                    
                                    "Sorted Set Members (use CLI to view)"
                                }
                            }
                        }
                        _ => {
                            rsx! {
                                div {
                                    color: "#888",
                                    
                                    "Unsupported type"
                                }
                            }
                        }
                    }
                } else {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "No data"
                    }
                }
            }
        }
    }
}