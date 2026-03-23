use crate::connection::ConnectionPool;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;

#[component]
pub fn FlushConfirmDialog(
    connection_pool: ConnectionPool,
    current_db: u8,
    colors: ThemeColors,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut processing = use_signal(|| false);
    let mut db_size = use_signal(|| None::<u64>);
    let mut loading = use_signal(|| true);

    use_effect({
        let pool = connection_pool.clone();
        move || {
            if db_size().is_none() {
                let pool = pool.clone();
                spawn(async move {
                    match pool.db_size().await {
                        Ok(size) => db_size.set(Some(size)),
                        Err(e) => tracing::error!("Failed to get db size: {}", e),
                    }
                    loading.set(false);
                });
            }
        }
    });

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "450px".to_string(),

            h3 {
                color: "{colors.error}",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "8px",
                font_size: "18px",

                "⚠️ 确认清空数据库"
            }

            div {
                background: "{colors.background_tertiary}",
                border: "1px solid {colors.border}",
                border_radius: "4px",
                padding: "16px",
                margin_bottom: "16px",

                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    margin_bottom: "12px",

                    "将要清空的数据库："
                }

                div {
                    display: "flex",
                    justify_content: "space_between",
                    align_items: "center",
                    padding: "8px 0",

                    span {
                        color: "{colors.text}",
                        font_size: "14px",

                        "数据库"
                    }

                    span {
                        color: "{colors.accent}",
                        font_size: "14px",
                        font_weight: "bold",

                        "DB {current_db}"
                    }
                }

                div {
                    display: "flex",
                    justify_content: "space_between",
                    align_items: "center",
                    padding: "8px 0",

                    span {
                        color: "{colors.text}",
                        font_size: "14px",

                        "Key 数量"
                    }

                    span {
                        color: if loading() {
                            colors.text_secondary
                        } else if db_size().unwrap_or(0) > 0 {
                            colors.error
                        } else {
                            colors.success
                        },
                        font_size: "14px",
                        font_weight: "bold",

                        if loading() {
                            "加载中..."
                        } else {
                            "{db_size().unwrap_or(0)}"
                        }
                    }
                }
            }

            div {
                color: "{colors.error}",
                font_size: "13px",
                margin_bottom: "16px",
                padding: "8px 12px",
                background: "{colors.error_bg}",
                border_radius: "4px",

                "⚠️ 此操作将清空 DB{current_db} 的所有 key，且不可恢复！"
            }

            div {
                display: "flex",
                gap: "8px",

                button {
                    flex: "1",
                    padding: "10px",
                    background: "{colors.error}",
                    color: "{colors.primary_text}",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    disabled: processing() || loading(),
                    onclick: {
                        let pool = connection_pool.clone();
                        let on_confirm = on_confirm.clone();
                        move |_| {
                            let pool = pool.clone();
                            let on_confirm = on_confirm.clone();
                            spawn(async move {
                                processing.set(true);

                                match pool.flush_db().await {
                                    Ok(_) => tracing::info!("Database flushed successfully"),
                                    Err(e) => tracing::error!("Failed to flush database: {}", e),
                                }

                                processing.set(false);
                                on_confirm.call(());
                            });
                        }
                    },

                    if processing() {
                        "清空中..."
                    } else {
                        "确认清空"
                    }
                }

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

                    "取消"
                }
            }
        }
    }
}