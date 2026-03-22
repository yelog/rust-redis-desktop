use crate::connection::ConnectionPool;
use crate::redis::ServerInfo;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_INFO,
    COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
    COLOR_WARNING,
};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

fn format_time(timestamp: u64) -> String {
    use chrono::{TimeZone, Utc};
    if let Some(dt) = Utc.timestamp_opt(timestamp as i64, 0).single() {
        dt.format("%H:%M:%S").to_string()
    } else {
        "--:--:--".to_string()
    }
}

fn format_memory_axis(max_value: u64) -> Vec<String> {
    if max_value == 0 {
        return vec!["0 B".to_string()];
    }

    let mut ticks = Vec::new();
    let steps = 4;
    for i in 0..=steps {
        let value = max_value * i / steps;
        ticks.push(format_bytes(value));
    }
    ticks
}

fn format_ops_axis(max_value: u64) -> Vec<String> {
    if max_value == 0 {
        return vec!["0".to_string()];
    }

    let mut ticks = Vec::new();
    let steps = 4;
    for i in 0..=steps {
        let value = max_value * i / steps;
        if value >= 10000 {
            ticks.push(format!("{:.1}K", value as f64 / 1000.0));
        } else {
            ticks.push(value.to_string());
        }
    }
    ticks
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
pub fn MonitorPanel(connection_pool: ConnectionPool, auto_refresh_interval: u32) -> Element {
    let mut monitor_data = use_signal(Vec::<MonitorData>::new);
    let mut current_info = use_signal(|| None::<ServerInfo>);
    let loading = use_signal(|| false);
    let mut is_monitoring = use_signal(|| false);
    let mut monitoring_handle: Signal<Option<Arc<AtomicBool>>> = use_signal(|| None);

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

    let start_monitoring = {
        let pool = connection_pool.clone();
        let load_data = load_data.clone();
        move |_| {
            if is_monitoring() {
                if let Some(handle) = monitoring_handle() {
                    handle.store(false, Ordering::SeqCst);
                }
                is_monitoring.set(false);
                monitoring_handle.set(None);
            } else {
                let running = Arc::new(AtomicBool::new(true));
                monitoring_handle.set(Some(running.clone()));
                is_monitoring.set(true);

                load_data();

                let pool = pool.clone();
                let mut monitor_data = monitor_data.clone();
                let mut current_info = current_info.clone();
                let running = running.clone();
                let interval_secs = auto_refresh_interval.max(1) as u64;

                spawn(async move {
                    let mut timer = tokio::time::interval(Duration::from_secs(interval_secs));
                    timer.tick().await;

                    while running.load(Ordering::SeqCst) {
                        timer.tick().await;
                        if !running.load(Ordering::SeqCst) {
                            break;
                        }

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
                    }
                });
            }
        }
    };

    let refresh_data = {
        let load_data = load_data.clone();
        move |_| {
            load_data();
        }
    };

    let info = current_info();
    let data = monitor_data();
    let max_memory = data.iter().map(|d| d.used_memory).max().unwrap_or(1);
    let max_ops = data.iter().map(|d| d.ops_per_sec).max().unwrap_or(1);

    rsx! {
        style { {include_str!("monitor_panel.css")} }

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

                    "📊 实时监控"
                }

                div {
                    display: "flex",
                    gap: "8px",

                    button {
                        padding: "6px 12px",
                        background: if is_monitoring() { "#c53030" } else { "#38a169" },
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: start_monitoring,

                        if is_monitoring() { "⏹ 停止监控" } else { "▶ 开始监控" }
                    }

                    button {
                        padding: "6px 12px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: refresh_data,

                        "🔄 刷新"
                    }
                }
            }

            div {
                padding: "16px",

                if loading() && data.is_empty() {
                    div {
                        color: COLOR_TEXT_SECONDARY,
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
                            background: COLOR_BG_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: COLOR_TEXT_SECONDARY,
                                font_size: "12px",
                                margin_bottom: "4px",

                                "内存使用"
                            }

                            div {
                                color: COLOR_ACCENT,
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.used_memory_human.clone().unwrap_or_else(|| format_bytes(info.used_memory.unwrap_or(0)))}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "11px",
                                margin_top: "4px",

                                "峰值: {info.as_ref().and_then(|i| i.used_memory_peak_human.clone()).unwrap_or_default()}"
                            }
                        }

                        div {
                            background: COLOR_BG_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: COLOR_TEXT_SECONDARY,
                                font_size: "12px",
                                margin_bottom: "4px",

                                "每秒操作数"
                            }

                            div {
                                color: COLOR_WARNING,
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.instantaneous_ops_per_sec.unwrap_or(0)}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "11px",
                                margin_top: "4px",

                                "ops/sec"
                            }
                        }

                        div {
                            background: COLOR_BG_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: COLOR_TEXT_SECONDARY,
                                font_size: "12px",
                                margin_bottom: "4px",

                                "连接数"
                            }

                            div {
                                color: COLOR_INFO,
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.connected_clients.unwrap_or(0)}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "11px",
                                margin_top: "4px",

                                "当前连接"
                            }
                        }

                        div {
                            background: COLOR_BG_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                color: COLOR_TEXT_SECONDARY,
                                font_size: "12px",
                                margin_bottom: "4px",

                                "Key 总数"
                            }

                            div {
                                color: COLOR_ACCENT,
                                font_size: "24px",
                                font_weight: "bold",

                                if let Some(ref info) = info {
                                    "{info.keys_total}"
                                } else {
                                    "-"
                                }
                            }

                            div {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "11px",
                                margin_top: "4px",

                                "keys"
                            }
                        }
                    }

                    div {
                        margin_top: "24px",

                        h3 {
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "14px",
                            margin_bottom: "12px",

                            "内存使用趋势"
                        }

                        div {
                            class: "chart-container",
                            background: COLOR_BG_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                display: "flex",

                                div {
                                    class: "y-axis",
                                    width: "60px",

                                    for label in format_memory_axis(max_memory).iter().rev() {
                                        div {
                                            class: "y-axis-label",
                                            "{label}"
                                        }
                                    }
                                }

                                div {
                                    class: "chart-content",

                                    div {
                                        class: "chart-bars",

                                        for (idx, d) in data.iter().enumerate() {
                                            {
                                                let height = if max_memory > 0 {
                                                    (d.used_memory as f64 / max_memory as f64 * 100.0) as u32
                                                } else {
                                                    0
                                                };
                                                let time_str = format_time(d.timestamp);
                                                let memory_str = format_bytes(d.used_memory);
                                                let show_time = data.len() < 10 || idx % (data.len() / 5).max(1) == 0 || idx == data.len() - 1;
                                                rsx! {
                                                    div {
                                                        class: "bar-wrapper",

                                                        div {
                                                            class: "bar bar-memory",
                                                            style: "height: {height}%",

                                                            div {
                                                                class: "tooltip",

                                                                div {
                                                                    class: "tooltip-value tooltip-value-memory",
                                                                    "{memory_str}"
                                                                }

                                                                div {
                                                                    class: "tooltip-time",
                                                                    "{time_str}"
                                                                }
                                                            }
                                                        }

                                                        if show_time {
                                                            div {
                                                                class: "x-axis-label",
                                                                "{time_str}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        margin_top: "24px",

                        h3 {
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "14px",
                            margin_bottom: "12px",

                            "OPS 趋势"
                        }

                        div {
                            class: "chart-container",
                            background: COLOR_BG_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "8px",
                            padding: "16px",

                            div {
                                display: "flex",

                                div {
                                    class: "y-axis",
                                    width: "60px",

                                    for label in format_ops_axis(max_ops).iter().rev() {
                                        div {
                                            class: "y-axis-label",
                                            "{label}"
                                        }
                                    }
                                }

                                div {
                                    class: "chart-content",

                                    div {
                                        class: "chart-bars",

                                        for (idx, d) in data.iter().enumerate() {
                                            {
                                                let height = if max_ops > 0 {
                                                    (d.ops_per_sec as f64 / max_ops as f64 * 100.0) as u32
                                                } else {
                                                    0
                                                };
                                                let time_str = format_time(d.timestamp);
                                                let ops_str = d.ops_per_sec.to_string();
                                                let show_time = data.len() < 10 || idx % (data.len() / 5).max(1) == 0 || idx == data.len() - 1;
                                                rsx! {
                                                    div {
                                                        class: "bar-wrapper",

                                                        div {
                                                            class: "bar bar-ops",
                                                            style: "height: {height}%",

                                                            div {
                                                                class: "tooltip",

                                                                div {
                                                                    class: "tooltip-value tooltip-value-ops",
                                                                    "{ops_str} ops/sec"
                                                                }

                                                                div {
                                                                    class: "tooltip-time",
                                                                    "{time_str}"
                                                                }
                                                            }
                                                        }

                                                        if show_time {
                                                            div {
                                                                class: "x-axis-label",
                                                                "{time_str}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if data.len() > 0 {
                        div {
                            margin_top: "16px",
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",

                            "最近 {data.len()} 个数据点"
                        }
                    }
                }
            }
        }
    }
}
