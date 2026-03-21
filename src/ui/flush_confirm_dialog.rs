use crate::connection::ConnectionPool;
use dioxus::prelude::*;

#[component]
pub fn FlushConfirmDialog(
    connection_pool: ConnectionPool,
    current_db: u8,
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
                    max_width: "450px",

                    h3 {
                        color: "#f87171",
                        margin_bottom: "16px",
                        display: "flex",
                        align_items: "center",
                        gap: "8px",

                        "⚠️ 确认清空数据库"
                    }

                    div {
                        background: "#1e1e1e",
                        border: "1px solid #3c3c3c",
                        border_radius: "4px",
                        padding: "16px",
                        margin_bottom: "16px",

                        div {
                            color: "#888",
                            font_size: "13px",
                            margin_bottom: "12px",

                                "将要清空的数据库："
                        }

    div {
                            display: "flex",
                            justify_content: "space-between",
                            align_items: "center",
                            padding: "8px 0",

                            span {
                                color: "#cccccc",
                                font_size: "14px",

                                "数据库"
                            }

                            span {
                                color: "#4ec9b0",
                                font_size: "14px",
                                font_weight: "bold",

                                "DB {current_db}"
                            }
                        }

                        div {
                            display: "flex",
                            justify_content: "space-between",
                            align_items: "center",
                            padding: "8px 0",

                            span {
                                color: "#cccccc",
                                font_size: "14px",

                                "Key 数量"
                            }

                            span {
                                color: if loading() {
                                    "#888"
                                } else if db_size().unwrap_or(0) > 0 {
                                    "#f87171"
                                } else {
                                    "#68d391"
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
                        color: "#f87171",
                        font_size: "13px",
                        margin_bottom: "16px",
                        padding: "8px 12px",
                        background: "rgba(248, 113, 113, 0.1)",
                        border_radius: "4px",

                        "⚠️ 此操作将清空 DB{current_db} 的所有 key，且不可恢复！"
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
