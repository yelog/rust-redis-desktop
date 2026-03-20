use crate::connection::ConnectionPool;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct DeleteTarget {
    pub key: String,
    pub is_folder: bool,
}

#[component]
pub fn DeleteConfirmDialog(
    connection_pool: ConnectionPool,
    targets: Vec<DeleteTarget>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut processing = use_signal(|| false);
    let mut keys_to_delete = use_signal(Vec::<String>::new);
    let mut loaded = use_signal(|| false);

    let total_count = targets.len();

    use_effect({
        let pool = connection_pool.clone();
        let targets = targets.clone();
        move || {
            if !loaded() {
                let pool = pool.clone();
                let targets = targets.clone();
                spawn(async move {
                    let mut all_keys = Vec::new();

                    for target in targets.iter() {
                        if target.is_folder {
                            match pool.scan_keys(&format!("{}*", target.key), 1000).await {
                                Ok(keys) => all_keys.extend(keys),
                                Err(e) => tracing::error!("Failed to scan keys: {}", e),
                            }
                        } else {
                            all_keys.push(target.key.clone());
                        }
                    }

                    keys_to_delete.set(all_keys);
                    loaded.set(true);
                });
            }
        }
    });

    let display_keys: Vec<String> = keys_to_delete.read().iter().take(20).cloned().collect();
    let remaining = keys_to_delete.read().len().saturating_sub(20);

    rsx! {
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
            z_index: "1001",

            div {
                background: "#252526",
                padding: "24px",
                border_radius: "8px",
                max_width: "500px",
                max_height: "80vh",
                display: "flex",
                flex_direction: "column",

                h3 {
                    color: "#f87171",
                    margin_bottom: "16px",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

                    "⚠️ 确认删除"
                }

                if !loaded() {
                    div {
                        color: "#888",
                        padding: "20px",
                        text_align: "center",

                        "正在加载 key 列表..."
                    }
                } else {
                    div {
                        color: "#888",
                        margin_bottom: "16px",

                        if total_count == 1 && !targets[0].is_folder {
                            "确定要删除这个 key 吗？"
                        } else {
                            "确定要删除这些 key 吗？"
                        }
                    }

                    div {
                        background: "#1e1e1e",
                        border: "1px solid #3c3c3c",
                        border_radius: "4px",
                        padding: "12px",
                        margin_bottom: "16px",
                        max_height: "300px",
                        overflow_y: "auto",
                        flex: "1",

                        div {
                            color: "#888",
                            font_size: "12px",
                            margin_bottom: "8px",

                            "即将删除 {keys_to_delete.read().len()} 个 key："
                        }

                        for key in display_keys.iter() {
                            div {
                                color: "#cccccc",
                                font_size: "13px",
                                padding: "4px 0",
                                font_family: "monospace",

                                "• {key}"
                            }
                        }

                        if remaining > 0 {
                            div {
                                color: "#666",
                                font_size: "12px",
                                margin_top: "8px",

                                "... 还有 {remaining} 个 key"
                            }
                        }
                    }

                    div {
                        display: "flex",
                        gap: "8px",

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#c53030",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            disabled: processing(),
                            onclick: {
                                let pool = connection_pool.clone();
                                let on_confirm = on_confirm.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let on_confirm = on_confirm.clone();
                                    let keys = keys_to_delete.read().clone();
                                    spawn(async move {
                                        processing.set(true);

                                        let mut success_count = 0;
                                        for key in keys.iter() {
                                            match pool.delete_key(key).await {
                                                Ok(_) => success_count += 1,
                                                Err(e) => tracing::error!("Failed to delete {}: {}", key, e),
                                            }
                                        }

                                        tracing::info!("Deleted {} keys", success_count);
                                        processing.set(false);
                                        on_confirm.call(());
                                    });
                                }
                            },

                            if processing() {
                                "删除中..."
                            } else {
                                "🗑️ 确认删除 ({keys_to_delete.read().len()})"
                            }
                        }

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#5a5a5a",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            disabled: processing(),
                            onclick: move |_| on_cancel.call(()),

                            "取消"
                        }
                    }
                }
            }
        }
    }
}