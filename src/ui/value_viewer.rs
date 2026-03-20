use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType};
use crate::ui::editable_field::EditableField;
use std::collections::HashMap;

#[component]
pub fn ValueViewer(
    connection_pool: ConnectionPool,
    selected_key: String,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut key_info = use_signal(|| None::<KeyInfo>);
    let mut string_value = use_signal(|| String::new());
    let mut hash_value = use_signal(|| HashMap::new());
    let mut loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    
    let key = selected_key.clone();
    let pool = connection_pool.clone();
    
    // Load data
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
                    
                    match info.key_type {
                        KeyType::String => {
                            if let Ok(val) = pool.get_string_value(&key).await {
                                string_value.set(val);
                            }
                        }
                        KeyType::Hash => {
                            if let Ok(fields) = pool.get_hash_all(&key).await {
                                hash_value.set(fields);
                            }
                        }
                        _ => {}
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
                } else if let Some(info) = key_info() {
                    match info.key_type {
                        KeyType::String => {
                            rsx! {
                                EditableField {
                                    label: "Value".to_string(),
                                    value: string_value(),
                                    on_change: {
                                        let pool = connection_pool.clone();
                                        let key = selected_key.clone();
                                        move |new_val: String| {
                                            let pool = pool.clone();
                                            let key = key.clone();
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
                                    editable: true,
                                    multiline: true,
                                }
                            }
                        }
                        KeyType::Hash => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",
                                    
                                    "Hash Fields:"
                                }
                                
                                for (field, value) in hash_value.read().iter() {
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
                                    
                                    "List Items:"
                                }
                                
                                div {
                                    color: "white",
                                    font_family: "Consolas, monospace",
                                    font_size: "13px",
                                    
                                    "Load with LRANGE command"
                                }
                            }
                        }
                        KeyType::Set => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",
                                    
                                    "Set Members:"
                                }
                                
                                div {
                                    color: "white",
                                    font_family: "Consolas, monospace",
                                    font_size: "13px",
                                    
                                    "Load with SMEMBERS command"
                                }
                            }
                        }
                        KeyType::ZSet => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",
                                    
                                    "Sorted Set Members:"
                                }
                                
                                div {
                                    color: "white",
                                    font_family: "Consolas, monospace",
                                    font_size: "13px",
                                    
                                    "Load with ZRANGE command"
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