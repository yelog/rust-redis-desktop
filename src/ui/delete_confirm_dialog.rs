use crate::connection::ConnectionPool;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
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
    colors: ThemeColors,
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
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "500px".to_string(),
            max_height: "80vh".to_string(),

            h3 {
                color: "{colors.error}",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "8px",
                font_size: "18px",

                "⚠️ 确认删除"
            }

            if !loaded() {
                div {
                    color: "{colors.text_secondary}",
                    padding: "20px",
                    text_align: "center",

                    "正在加载 key 列表..."
                }
            } else {
                div {
                    color: "{colors.text_secondary}",
                    margin_bottom: "16px",
                    font_size: "13px",

                    if total_count == 1 && !targets[0].is_folder {
                        "确定要删除这个 key 吗？"
                    } else {
                        "确定要删除这些 key 吗？"
                    }
                }

                div {
                    background: "{colors.background_tertiary}",
                    border: "1px solid {colors.border}",
                    border_radius: "4px",
                    padding: "12px",
                    margin_bottom: "16px",
                    max_height: "300px",
                    overflow_y: "auto",
                    flex: "1",

                    div {
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "8px",

                        "即将删除 {keys_to_delete.read().len()} 个 key："
                    }

                    for key in display_keys.iter() {
                        div {
                            color: "{colors.text}",
                            font_size: "13px",
                            padding: "4px 0",
                            font_family: "monospace",

                            "• {key}"
                        }
                    }

                    if remaining > 0 {
                        div {
                            color: "{colors.text_subtle}",
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
                        background: "{colors.error}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
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
                            "确认删除 ({keys_to_delete.read().len()})"
                        }
                    }

                    button {
                        flex: "1",
                        padding: "8px",
                        background: "{colors.background_tertiary}",
                        color: "{colors.text}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        disabled: processing(),
                        onclick: move |_| on_cancel.call(()),

                        "取消"
                    }
                }
            }
        }
    }
}