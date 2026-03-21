use crate::connection::ConnectionPool;
use dioxus::prelude::*;

#[component]
pub fn FlushConfirmDialog(
    connection_pool: ConnectionPool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut processing = use_signal(|| false);

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

                    "⚠️ 确认清空数据"
                }

                div {
                    color: "#888",
                    margin_bottom: "16px",
                    line_height: "1.6",

                    "此操作将清空当前数据库的所有 key，且不可恢复！"
                }

                div {
                    background: "#1e1e1e",
                    border: "1px solid #3c3c3c",
                    border_radius: "4px",
                    padding: "12px",
                    margin_bottom: "16px",

                    div {
                        color: "#f87171",
                        font_size: "13px",

                        "⚠️ 危险操作：所有数据将被永久删除"
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
                            "🗑️ 确认清空"
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
