use crate::connection::ConnectionPool;
use crate::redis::ServerInfo;
use dioxus::prelude::*;
use std::time::Duration;

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Clone, Default)]
pub struct MonitorData {
    pub timestamp: u64,
    pub used_memory: u64,
    pub ops_per_sec: u64,
    pub connected_clients: u64,
    pub keys_total: u64,
    pub hits: u64,
    pub misses: u64,
}

#[component]
pub fn MonitorPanel(
    connection_pool: ConnectionPool,
    auto_refresh_interval: u32,
) -> Element {
    let mut monitor_data = use_signal(Vec::<MonitorData>::new);
    let current_info = use_signal(|| None::<ServerInfo>);
    let loading = use_signal(|| false);
    let mut refresh_trigger = use_signal(|| 0u32);
    let mut is_monitoring = use_signal(|| false);

    let load_data = {
        let pool = connection_pool.clone();
        move || {
            let pool = pool.clone();
            let mut monitor_data = monitor_data.clone();
            let mut current_info = current_info.clone();
            let mut loading = loading.clone();
            spawn(async move {
                loading.set(true);
                match pool.get_server_info().await {
                    Ok(info) => {
                        current_info.set(Some(info.clone()));
                        let data = MonitorData {
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            used_memory: info.used_memory.unwrap_or(0),
                            ops_per_sec: info.instantaneous_ops_per_sec.unwrap_or(0),
                            connected_clients: info.connected_clients.unwrap_or(0),
                            keys_total: info.keys_total,
                            hits: 0,
                            misses: 0,
                        };
                        let mut data_vec = monitor_data();
                        data_vec.push(data);
                        if data_vec.len() > 60 {
                            data_vec.remove(0);
                        }
                        monitor_data.set(data_vec);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load monitor data: {}", e);
                    }
                }
                loading.set(false);
            });
        }
    };

    use_effect({
        let load_data = load_data.clone();
        move || {
            let _ = refresh_trigger();
            load_data();
        }
    });

    use_effect(move || {
        if !is_monitoring() || auto_refresh_interval == 0 {
            return;
        }

        let mut refresh_trigger = refresh_trigger.clone();
        spawn(async move {
            let mut timer = tokio::time::interval(Duration::from_secs(auto_refresh_interval as u64));
            loop {
                timer.tick().await;
                if !is_monitoring() {
                    break;
                }
                refresh_trigger.set(refresh_trigger() + 1);
            }
        });
    });

    let info = current_info();
    let data = monitor_data();
    let max_memory = data.iter().map(|d| d.used_memory).max().unwrap_or(1);
    let max_ops = data.iter().map(|d| d.ops_per_sec).max().unwrap_or(1);

    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background: "#1e1e1e",
            overflow_y: "auto",

            div {
                padding: "16px",
                border_bottom: "1px solid #3c3c3c",
                display: "flex",
                justify_content: "space_between",
                align_items: "center",

                h2 {
                    color: "white",
                    font_size: "18px",
                    margin: "0",

                    "📊 实时监控"
                }

                div {
                    display: "flex",
                    gap: "8px",

                    button {
                        padding: "6px 12px",
                        background: if is_monitoring() { "#c53030" } else { "#38a169" },
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: move |_| is_monitoring.set(!is_monitoring()),

                        if is_monitoring() { "⏹ 停止监控" } else { "▶ 开始监控" }
                    }

                    button {
                        padding: "6px 12px",
                        background: "#3c3c3c",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: move |_| {
                            monitor_data.set(Vec::new());
                            refresh_trigger.set(refresh_trigger() + 1);
                        },

                        "🔄 刷新"
                    }
                }
            }

            div {
                padding: "16px",

                if loading() && data.is_empty() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "40px",

                        "加载中..."
                    }
                } else {
                    div {
                        display: "grid",
                        grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                        gap: "16px",
                        margin_bottom: "24px",

                        div {
                            background: "#252526",
                            border: "1px solid #3c3c3c",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: "#888",
                                font_size: "12px",
                                margin_bottom: "4px",

                                "内存使用"
                            }

                            div {
                                color: "#4ec9b0",
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.used_memory_human.clone().unwrap_or_else(|| format_bytes(info.used_memory.unwrap_or(0)))}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: "#666",
                                font_size: "11px",
                                margin_top: "4px",

                                "峰值: {info.as_ref().and_then(|i| i.used_memory_peak_human.clone()).unwrap_or_default()}"
                            }
                        }

                        div {
                            background: "#252526",
                            border: "1px solid #3c3c3c",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: "#888",
                                font_size: "12px",
                                margin_bottom: "4px",

                                "每秒操作数"
                            }

                            div {
                                color: "#f59e0b",
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.instantaneous_ops_per_sec.unwrap_or(0)}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: "#666",
                                font_size: "11px",
                                margin_top: "4px",

                                "ops/sec"
                            }
                        }

                        div {
                            background: "#252526",
                            border: "1px solid #3c3c3c",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: "#888",
                                font_size: "12px",
                                margin_bottom: "4px",

                                "连接数"
                            }

                            div {
                                color: "#63b3ed",
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.connected_clients.unwrap_or(0)}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: "#666",
                                font_size: "11px",
                                margin_top: "4px",

                                "当前连接"
                            }
                        }

                        div {
                            background: "#252526",
                            border: "1px solid #3c3c3c",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: "#888",
                                font_size: "12px",
                                margin_bottom: "4px",

                                "Key 总数"
                            }

                            div {
                                color: "#a78bfa",
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.keys_total}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: "#666",
                                font_size: "11px",
                                margin_top: "4px",

                                "keys"
                            }
                        }
                    }

                    div {
                        margin_top: "24px",

                        h3 {
                            color: "#888",
                            font_size: "14px",
                            margin_bottom: "12px",

                            "内存使用趋势"
                        }

                        div {
                            background: "#252526",
                            border: "1px solid #3c3c3c",
                            border_radius: "8px",
                            padding: "16px",
                            height: "120px",
                            display: "flex",
                            align_items: "flex_end",
                            gap: "2px",

                            for d in data.iter() {
                                {
                                    let height = if max_memory > 0 {
                                        (d.used_memory as f64 / max_memory as f64 * 100.0) as u32
                                    } else {
                                        0
                                    };
                                    rsx! {
                                        div {
                                            flex: "1",
                                            background: "linear-gradient(to top, #4ec9b0, #38a169)",
                                            border_radius: "2px 2px 0 0",
                                            min_width: "4px",
                                            height: "{height}%",
                                            title: "{format_bytes(d.used_memory)}",
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        margin_top: "24px",

                        h3 {
                            color: "#888",
                            font_size: "14px",
                            margin_bottom: "12px",

                            "OPS 趋势"
                        }

                        div {
                            background: "#252526",
                            border: "1px solid #3c3c3c",
                            border_radius: "8px",
                            padding: "16px",
                            height: "120px",
                            display: "flex",
                            align_items: "flex_end",
                            gap: "2px",

                            for d in data.iter() {
                                {
                                    let height = if max_ops > 0 {
                                        (d.ops_per_sec as f64 / max_ops as f64 * 100.0) as u32
                                    } else {
                                        0
                                    };
                                    rsx! {
                                        div {
                                            flex: "1",
                                            background: "linear-gradient(to top, #f59e0b, #d97706)",
                                            border_radius: "2px 2px 0 0",
                                            min_width: "4px",
                                            height: "{height}%",
                                            title: "{d.ops_per_sec} ops/sec",
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if data.len() > 0 {
                        div {
                            margin_top: "16px",
                            color: "#666",
                            font_size: "11px",

                            "最近 {data.len()} 个数据点"
                        }
                    }
                }
            }
        }
    }
}