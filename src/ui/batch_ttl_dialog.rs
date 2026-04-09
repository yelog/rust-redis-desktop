use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;

#[component]
pub fn BatchTtlDialog(
    connection_pool: ConnectionPool,
    keys: Vec<String>,
    current_ttl: Option<i64>,
    colors: ThemeColors,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let default_ttl = current_ttl.unwrap_or(-1);
    let mut ttl_value = use_signal(|| default_ttl.to_string());
    let mut processing = use_signal(|| false);
    let mut status_message = use_signal(String::new);
    let mut status_error = use_signal(|| false);

    let keys_count = keys.len();
    let is_single = keys_count == 1;
    let display_keys: Vec<String> = keys.iter().take(10).cloned().collect();
    let remaining = keys.len().saturating_sub(10);

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "500px".to_string(),
            title: if is_single {
                i18n.read().t("Set TTL")
            } else {
                i18n.read().t("Batch set TTL")
            },

            div {
                color: "{colors.text_secondary}",
                margin_bottom: "16px",
                font_size: "13px",

                if is_single {
                    "为以下 key 设置过期时间"
                } else {
                    "将为 {keys_count} 个 key 设置过期时间"
                }
            }

            div {
                background: "{colors.background_tertiary}",
                border: "1px solid {colors.border}",
                border_radius: "4px",
                padding: "12px",
                margin_bottom: "16px",
                max_height: "150px",
                overflow_y: "auto",

                for key in display_keys.iter() {
                    div {
                        color: "{colors.text}",
                        font_size: "12px",
                        padding: "2px 0",
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
                margin_bottom: "16px",

                label {
                    display: "block",
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    margin_bottom: "8px",

                    "TTL (秒, -1 表示永不过期, -2 表示立即删除)"
                }

                input {
                    width: "100%",
                    padding: "10px 12px",
                    background: "{colors.background_tertiary}",
                    border: "1px solid {colors.border}",
                    border_radius: "4px",
                    color: "{colors.text}",
                    font_size: "14px",
                    box_sizing: "border_box",
                    value: "{ttl_value}",
                    oninput: move |e| ttl_value.set(e.value()),
                }
            }

            if !status_message.read().is_empty() {
                div {
                    margin_bottom: "16px",
                    padding: "8px 12px",
                    background: if status_error() { colors.error_bg } else { colors.success_bg },
                    border_radius: "4px",
                    color: if status_error() { colors.error } else { colors.success },
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
                    background: "{colors.background_tertiary}",
                    color: "{colors.text}",
                    border: "1px solid {colors.border}",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    disabled: processing(),
                    onclick: move |_| on_cancel.call(()),

                    {i18n.read().t("Cancel")}
                }

                button {
                    flex: "1",
                    padding: "10px",
                    background: "{colors.primary}",
                    color: "{colors.primary_text}",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    disabled: processing(),
                    onclick: {
                        let pool = connection_pool.clone();
                        move |_| {
                            let pool = pool.clone();
                            let ttl = ttl_value().parse::<i64>().unwrap_or(-1);
                            let keys = keys.clone();
                            spawn(async move {
                                if ttl < -2 {
                                     status_message.set(i18n.read().t("TTL must be greater than or equal to -2"));
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
                                     status_message.set(format!("{} {}", i18n.read().t("TTL updated for keys:"), success_count));
                                    status_error.set(false);
                                    spawn(async move {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                        on_confirm.call(());
                                    });
                                } else {
                                     status_message.set(format!("{} {}, {} {}", i18n.read().t("Success:"), success_count, i18n.read().t("Failed:"), fail_count));
                                    status_error.set(true);
                                }
                            });
                        }
                    },

                    if processing() {
                        {i18n.read().t("Applying TTL...")}
                    } else {
                        {format!("✓ {}", i18n.read().t("Apply TTL"))}
                    }
                }
            }
        }
    }
}
