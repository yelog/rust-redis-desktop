use crate::connection::ConnectionPool;
use crate::ui::icons::*;
use dioxus::prelude::*;

#[component]
pub fn BatchTtlDialog(
    connection_pool: ConnectionPool,
    keys: Vec<String>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut ttl_value = use_signal(|| String::from("-1"));
    let mut processing = use_signal(|| false);
    let mut status_message = use_signal(String::new);
    let mut status_error = use_signal(|| false);

    let keys_count = keys.len();
    let display_keys: Vec<String> = keys.iter().take(10).cloned().collect();
    let remaining = keys.len().saturating_sub(10);

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
                width: "100%",

                h3 {
                    color: "#4ec9b0",
                    margin_bottom: "16px",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

                    IconRefresh { size: Some(16) }
                    " 批量设置 TTL"
                }

                div {
                    color: "#888",
                    margin_bottom: "16px",

                    "将为 {keys_count} 个 key 设置过期时间"
                }

                div {
                    background: "#1e1e1e",
                    border: "1px solid #3c3c3c",
                    border_radius: "4px",
                    padding: "12px",
                    margin_bottom: "16px",
                    max_height: "150px",
                    overflow_y: "auto",

                    for key in display_keys.iter() {
                        div {
                            color: "#cccccc",
                            font_size: "12px",
                            padding: "2px 0",
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
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        font_size: "13px",
                        margin_bottom: "8px",

                        "TTL (秒, -1 表示永不过期, -2 表示立即删除)"
                    }

                    input {
                        width: "100%",
                        padding: "10px 12px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        font_size: "14px",
                        box_sizing: "border-box",
                        value: "{ttl_value}",
                        oninput: move |e| ttl_value.set(e.value()),
                    }
                }

                if !status_message.read().is_empty() {
                    div {
                        margin_bottom: "16px",
                        padding: "8px 12px",
                        background: if status_error() { "rgba(248, 113, 113, 0.1)" } else { "rgba(78, 201, 176, 0.1)" },
                        border_radius: "4px",
                        color: if status_error() { "#f87171" } else { "#4ec9b0" },
                        font_size: "13px",

                        "{status_message}"
                    }
                }

                div {
                    display: "flex",
                    gap: "8px",

                    button {
                        flex: "1",
                        padding: "10px",
                        background: "#5a5a5a",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        disabled: processing(),
                        onclick: move |_| on_cancel.call(()),

                        "取消"
                    }

                    button {
                        flex: "1",
                        padding: "10px",
                        background: "#0e639c",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        disabled: processing(),
                        onclick: {
                            let pool = connection_pool.clone();
                            move |_| {
                                let pool = pool.clone();
                                let ttl = ttl_value().parse::<i64>().unwrap_or(-1);
                                let keys = keys.clone();
                                spawn(async move {
                                    if ttl < -2 {
                                        status_message.set("TTL 必须大于等于 -2".to_string());
                                        status_error.set(true);
                                        return;
                                    }

                                    processing.set(true);
                                    status_message.set(String::new());
                                    status_error.set(false);

                                    let mut success_count = 0;
                                    let mut fail_count = 0;

                                    for key in keys.iter() {
                                        if ttl == -1 {
                                            match pool.remove_ttl(key).await {
                                                Ok(_) => success_count += 1,
                                                Err(_) => fail_count += 1,
                                            }
                                        } else if ttl >= 0 {
                                            match pool.set_ttl(key, ttl).await {
                                                Ok(_) => success_count += 1,
                                                Err(_) => fail_count += 1,
                                            }
                                        }
                                    }

                                    processing.set(false);

                                    if fail_count == 0 {
                                        status_message.set(format!("成功设置 {} 个 key 的 TTL", success_count));
                                        status_error.set(false);
                                        spawn(async move {
                                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                            on_confirm.call(());
                                        });
                                    } else {
                                        status_message.set(format!("成功: {}, 失败: {}", success_count, fail_count));
                                        status_error.set(true);
                                    }
                                });
                            }
                        },

                        if processing() {
                            "设置中..."
                        } else {
                            "✓ 确认设置"
                        }
                    }
                }
            }
        }
    }
}
