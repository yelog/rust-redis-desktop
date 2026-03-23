use crate::connection::ConnectionPool;
use crate::theme::{
    COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_TEXT,
    COLOR_TEXT_SECONDARY, COLOR_TEXT_SOFT, COLOR_TEXT_SUBTLE, COLOR_WARNING,
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SlowLogEntry {
    pub id: u64,
    pub timestamp: u64,
    pub duration: u64,
    pub command: Vec<String>,
}

async fn get_slowlog(pool: &ConnectionPool) -> Result<Vec<SlowLogEntry>, String> {
    let mut connection = pool.connection.lock().await;

    if let Some(ref mut conn) = *connection {
        let result: Vec<(u64, u64, u64, Vec<String>)> = conn
            .execute_cmd(redis::cmd("SLOWLOG").arg("GET").arg(100))
            .await
            .map_err(|e| format!("Failed to get slowlog: {}", e))?;

        Ok(result
            .into_iter()
            .map(|(id, ts, duration, cmd)| SlowLogEntry {
                id,
                timestamp: ts,
                duration,
                command: cmd,
            })
            .collect())
    } else {
        Err("Connection closed".to_string())
    }
}

fn format_duration(micros: u64) -> String {
    if micros >= 1_000_000 {
        format!("{:.2} s", micros as f64 / 1_000_000.0)
    } else if micros >= 1_000 {
        format!("{:.2} ms", micros as f64 / 1_000.0)
    } else {
        format!("{} μs", micros)
    }
}

fn format_timestamp(ts: u64) -> String {
    let datetime = chrono::DateTime::from_timestamp(ts as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH);
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[component]
pub fn SlowLogPanel(connection_pool: ConnectionPool) -> Element {
    let slowlog_entries = use_signal(Vec::<SlowLogEntry>::new);
    let loading = use_signal(|| false);
    let mut refresh_trigger = use_signal(|| 0u32);

    use_effect({
        let pool = connection_pool.clone();
        move || {
            let _ = refresh_trigger();
            let pool = pool.clone();
            let mut slowlog_entries = slowlog_entries.clone();
            let mut loading = loading.clone();
            spawn(async move {
                loading.set(true);
                match get_slowlog(&pool).await {
                    Ok(entries) => {
                        slowlog_entries.set(entries);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load slowlog: {}", e);
                    }
                }
                loading.set(false);
            });
        }
    });

    let entries = slowlog_entries();

    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background: COLOR_BG,
            overflow_y: "auto",

            div {
                padding: "16px",
                border_bottom: "1px solid {COLOR_BORDER}",
                display: "flex",
                justify_content: "space_between",
                align_items: "center",

                h2 {
                    color: COLOR_TEXT,
                    font_size: "18px",
                    margin: "0",

                    "🐌 慢查询日志"
                }

                button {
                    padding: "6px 12px",
                    background: COLOR_BG_TERTIARY,
                    color: COLOR_TEXT,
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "12px",
                    onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),

                    "🔄 刷新"
                }
            }

            div {
                padding: "16px",

                if loading() {
                    div {
                        color: COLOR_TEXT_SECONDARY,
                        text_align: "center",
                        padding: "40px",

                        "加载中..."
                    }
                } else if entries.is_empty() {
                    div {
                        color: COLOR_TEXT_SECONDARY,
                        text_align: "center",
                        padding: "40px",

                        "暂无慢查询记录"
                    }
                } else {
                    div {
                        overflow_x: "auto",
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "8px",
                        background: COLOR_BG_SECONDARY,

                        table {
                            width: "100%",
                            border_collapse: "collapse",

                            thead {
                                tr {
                                    background: COLOR_BG_TERTIARY,
                                    border_bottom: "1px solid {COLOR_BORDER}",

                                    th {
                                        width: "60px",
                                        padding: "12px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "12px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "ID"
                                    }

                                    th {
                                        width: "150px",
                                        padding: "12px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "12px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "时间"
                                    }

                                    th {
                                        width: "100px",
                                        padding: "12px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "12px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "耗时"
                                    }

                                    th {
                                        padding: "12px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "12px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "命令"
                                    }
                                }
                            }

                            tbody {
                                for (idx, entry) in entries.iter().enumerate() {
                                    tr {
                                        key: "{entry.id}",
                                        background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },
                                        border_bottom: "1px solid {COLOR_BORDER}",

                                        td {
                                            padding: "10px 12px",
                                            color: COLOR_TEXT_SECONDARY,
                                            font_size: "12px",

                                            "{entry.id}"
                                        }

                                        td {
                                            padding: "10px 12px",
                                            color: COLOR_TEXT_SOFT,
                                            font_size: "12px",

                                            "{format_timestamp(entry.timestamp)}"
                                        }

                                        td {
                                            padding: "10px 12px",
                                            color: if entry.duration > 10_000 { "var(--theme-error, #d13438)" } else { COLOR_WARNING },
                                            font_size: "12px",
                                            font_weight: "bold",

                                            "{format_duration(entry.duration)}"
                                        }

                                        td {
                                            padding: "10px 12px",
                                            color: COLOR_TEXT,
                                            font_size: "12px",
                                            font_family: "Consolas, monospace",

                                            "{entry.command.join(\" \")}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        margin_top: "12px",
                        color: COLOR_TEXT_SUBTLE,
                        font_size: "12px",

                        "共 {entries.len()} 条记录"
                    }
                }
            }
        }
    }
}
