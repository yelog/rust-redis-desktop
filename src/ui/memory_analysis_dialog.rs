use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::icons::{IconSearch, IconX};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct MemoryEntry {
    pub key: String,
    pub memory_bytes: u64,
    pub key_type: String,
    pub ttl: i64,
}

#[derive(Clone, PartialEq)]
pub struct PrefixStats {
    pub prefix: String,
    pub key_count: u64,
    pub memory_bytes: u64,
    pub types: Vec<String>,
    pub avg_ttl_secs: f64,
}

#[derive(Clone, PartialEq)]
enum ScanState {
    Idle,
    Scanning { checked: usize, found: usize },
    Completed { checked: usize, found: usize },
    Error(String),
}

#[derive(Clone, Copy, PartialEq)]
enum ViewTab {
    Keys,
    Prefixes,
}

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

fn format_ttl(ttl: i64) -> String {
    if ttl == -1 {
        "永久".to_string()
    } else if ttl == -2 {
        "不存在".to_string()
    } else if ttl < 0 {
        "未知".to_string()
    } else if ttl < 60 {
        format!("{}秒", ttl)
    } else if ttl < 3600 {
        format!("{}分", ttl / 60)
    } else if ttl < 86400 {
        format!("{}时", ttl / 3600)
    } else {
        format!("{}天", ttl / 86400)
    }
}

fn extract_prefix(key: &str, separator: &str) -> Option<String> {
    if key.contains(separator) {
        key.split(separator).next().map(|s| s.to_string())
    } else {
        None
    }
}

#[component]
pub fn MemoryAnalysisDialog(
    connection_pool: ConnectionPool,
    colors: ThemeColors,
    on_select_key: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let mut entries = use_signal(Vec::<MemoryEntry>::new);
    let mut prefix_stats = use_signal(Vec::<PrefixStats>::new);
    let mut state = use_signal(|| ScanState::Idle);
    let mut cancel_flag = use_signal(|| None::<Arc<AtomicBool>>);
    let mut min_size_filter = use_signal(|| 0u64);
    let mut pattern = use_signal(|| "*".to_string());
    let mut sample_ratio = use_signal(|| 1.0f32);
    let mut active_tab = use_signal(|| ViewTab::Keys);
    let mut separator = use_signal(|| ":".to_string());

    let start_scan = {
        let connection_pool = connection_pool.clone();
        move |_| {
            let pool = connection_pool.clone();
            let flag = Arc::new(AtomicBool::new(true));
            cancel_flag.set(Some(flag.clone()));
            entries.set(Vec::new());
            prefix_stats.set(Vec::new());

            let pattern_val = pattern();
            let min_size = min_size_filter();
            let ratio = sample_ratio();
            let sep = separator();
            state.set(ScanState::Scanning {
                checked: 0,
                found: 0,
            });

            spawn(async move {
                let flag = flag.clone();
                let mut cursor: u64 = 0;
                let mut all_entries = Vec::new();
                let mut checked_count = 0usize;
                let mut prefix_map: HashMap<String, (u64, u64, Vec<String>, i64, u64)> =
                    HashMap::new();

                loop {
                    if !flag.load(Ordering::Relaxed) {
                        state.set(ScanState::Idle);
                        return;
                    }

                    let result = pool.scan_keys_with_cursor(&pattern_val, cursor, 100).await;

                    match result {
                        Ok((next_cursor, keys)) => {
                            checked_count += keys.len();

                            for key in keys {
                                if !flag.load(Ordering::Relaxed) {
                                    state.set(ScanState::Idle);
                                    return;
                                }

                                if ratio < 1.0 && rand::random::<f32>() > ratio {
                                    continue;
                                }

                                if let Ok(Some(bytes)) = pool.memory_usage(&key).await {
                                    if bytes >= min_size {
                                        let key_type = pool
                                            .get_key_type(&key)
                                            .await
                                            .map(|t| format!("{:?}", t))
                                            .unwrap_or_else(|_| "Unknown".to_string());

                                        let ttl = pool
                                            .get_key_info(&key)
                                            .await
                                            .ok()
                                            .and_then(|info| info.ttl)
                                            .unwrap_or(-2);

                                        all_entries.push(MemoryEntry {
                                            key: key.clone(),
                                            memory_bytes: bytes,
                                            key_type: key_type.clone(),
                                            ttl,
                                        });

                                        if let Some(prefix) = extract_prefix(&key, &sep) {
                                            let entry = prefix_map.entry(prefix).or_insert((
                                                0,
                                                0,
                                                Vec::new(),
                                                0,
                                                0,
                                            ));
                                            entry.0 += 1;
                                            entry.1 += bytes;
                                            if !entry.2.contains(&key_type) {
                                                entry.2.push(key_type);
                                            }
                                            if ttl > 0 {
                                                entry.3 += ttl;
                                                entry.4 += 1;
                                            }
                                        }
                                    }
                                }
                            }

                            state.set(ScanState::Scanning {
                                checked: checked_count,
                                found: all_entries.len(),
                            });

                            cursor = next_cursor;
                            if cursor == 0 {
                                break;
                            }
                        }
                        Err(e) => {
                            state.set(ScanState::Error(e.to_string()));
                            return;
                        }
                    }
                }

                all_entries.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));

                let mut prefixes: Vec<PrefixStats> = prefix_map
                    .into_iter()
                    .map(
                        |(prefix, (count, mem, types, ttl_sum, ttl_count))| PrefixStats {
                            prefix: format!("{}{}*", prefix, sep),
                            key_count: count,
                            memory_bytes: mem,
                            types,
                            avg_ttl_secs: if ttl_count > 0 {
                                ttl_sum as f64 / ttl_count as f64
                            } else {
                                -1.0
                            },
                        },
                    )
                    .collect();
                prefixes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
                prefixes.truncate(50);

                let found = all_entries.len();
                entries.set(all_entries);
                prefix_stats.set(prefixes);
                state.set(ScanState::Completed {
                    checked: checked_count,
                    found,
                });
            });
        }
    };

    let stop_scan = move |_| {
        if let Some(flag) = cancel_flag() {
            flag.store(false, Ordering::Relaxed);
        }
        state.set(ScanState::Idle);
    };

    let total_memory: u64 = entries().iter().map(|e| e.memory_bytes).sum();
    let prefix_total_memory: u64 = prefix_stats().iter().map(|p| p.memory_bytes).sum();

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "900px".to_string(),
            max_height: "90vh".to_string(),
            title: i18n.read().t("Memory analysis"),

            div {
                display: "flex",
                flex_direction: "column",
                gap: "16px",

                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "12px",

                    div {
                        flex: "1 1 200px",
                        min_width: "0",

                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            margin_bottom: "6px",

                            "Pattern"
                        }

                        input {
                            width: "100%",
                            padding: "8px 12px",
                            background: "{colors.background}",
                            border: "1px solid {colors.border}",
                            border_radius: "6px",
                            color: "{colors.text}",
                            font_size: "13px",
                            font_family: "monospace",
                            box_sizing: "border_box",
                            value: "{pattern}",
                            oninput: move |e| pattern.set(e.value()),
                        }
                    }

                    div {
                        flex: "0 0 120px",

                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            margin_bottom: "6px",

                            "Min Size (B)"
                        }

                        input {
                            width: "100%",
                            padding: "8px 12px",
                            background: "{colors.background}",
                            border: "1px solid {colors.border}",
                            border_radius: "6px",
                            color: "{colors.text}",
                            font_size: "13px",
                            box_sizing: "border_box",
                            r#type: "number",
                            value: "{min_size_filter}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse() {
                                    min_size_filter.set(v);
                                }
                            },
                        }
                    }

                    div {
                        flex: "0 0 100px",

                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            margin_bottom: "6px",

                            "采样率"
                        }

                        input {
                            width: "100%",
                            padding: "8px 12px",
                            background: "{colors.background}",
                            border: "1px solid {colors.border}",
                            border_radius: "6px",
                            color: "{colors.text}",
                            font_size: "13px",
                            box_sizing: "border_box",
                            r#type: "number",
                            step: "0.1",
                            min: "0.1",
                            max: "1.0",
                            value: "{sample_ratio}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<f32>() {
                                    sample_ratio.set(v.clamp(0.1, 1.0));
                                }
                            },
                        }
                    }

                    div {
                        flex: "0 0 80px",

                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            margin_bottom: "6px",

                            "分隔符"
                        }

                        input {
                            width: "100%",
                            padding: "8px 12px",
                            background: "{colors.background}",
                            border: "1px solid {colors.border}",
                            border_radius: "6px",
                            color: "{colors.text}",
                            font_size: "13px",
                            box_sizing: "border_box",
                            value: "{separator}",
                            oninput: move |e| separator.set(e.value()),
                        }
                    }

                    div {
                        flex: "0 0 auto",
                        display: "flex",
                        align_items: "flex_end",

                        if matches!(state(), ScanState::Scanning { .. }) {
                            button {
                                padding: "8px 16px",
                                background: "{colors.error}",
                                color: "{colors.primary_text}",
                                border: "none",
                                border_radius: "6px",
                                cursor: "pointer",
                                font_size: "13px",
                                font_weight: "600",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                onclick: stop_scan,

                                IconX { size: Some(14), color: Some(colors.primary_text.to_string()) }
                                 {i18n.read().t("Stop")}
                            }
                        } else {
                            button {
                                padding: "8px 16px",
                                background: "{colors.primary}",
                                color: "{colors.primary_text}",
                                border: "none",
                                border_radius: "6px",
                                cursor: "pointer",
                                font_size: "13px",
                                font_weight: "600",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                onclick: start_scan.clone(),

                                IconSearch { size: Some(14), color: Some(colors.primary_text.to_string()) }
                                 {i18n.read().t("Scan")}
                            }
                        }
                    }
                }

                match state() {
                    ScanState::Scanning { checked, found } => rsx! {
                        div {
                            padding: "12px 16px",
                            background: "{colors.background_secondary}",
                            border: "1px solid {colors.border}",
                            border_radius: "8px",

                            span {
                                color: "{colors.text_secondary}",
                                font_size: "13px",

                                "已扫描 {checked} 个 key，发现 {found} 个大 key..."
                            }
                        }
                    },
                    ScanState::Error(ref e) => rsx! {
                        div {
                            padding: "12px 16px",
                            background: "{colors.error_bg}",
                            border: "1px solid {colors.error}",
                            border_radius: "8px",
                            color: "{colors.error}",
                            font_size: "13px",

                            {format!("{}{}", i18n.read().t("Scan failed: "), e)}
                        }
                    },
                    ScanState::Completed { checked, found } if found > 0 => rsx! {
                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: "12px",

                            div {
                                display: "flex",
                                justify_content: "space_between",
                                align_items: "center",
                                padding: "10px 14px",
                                background: "{colors.background_secondary}",
                                border: "1px solid {colors.border}",
                                border_radius: "8px",

                                span {
                                    color: "{colors.text}",
                                    font_size: "13px",

                                    "扫描 {checked} 个 key，发现 {found} 个大 key"
                                }

                                span {
                                    color: "{colors.accent}",
                                    font_size: "13px",
                                    font_weight: "600",

                                    "总内存: {format_bytes(total_memory)}"
                                }
                            }

                            div {
                                display: "flex",
                                gap: "8px",
                                margin_bottom: "8px",

                                button {
                                    padding: "6px 12px",
                                    background: if active_tab() == ViewTab::Keys { colors.primary } else { colors.surface_low },
                                    color: if active_tab() == ViewTab::Keys { colors.primary_text } else { colors.text },
                                    border: "1px solid {colors.border}",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "12px",
                                    onclick: move |_| active_tab.set(ViewTab::Keys),

                                    "Key 列表 ({entries.read().len()})"
                                }

                                button {
                                    padding: "6px 12px",
                                    background: if active_tab() == ViewTab::Prefixes { colors.primary } else { colors.surface_low },
                                    color: if active_tab() == ViewTab::Prefixes { colors.primary_text } else { colors.text },
                                    border: "1px solid {colors.border}",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "12px",
                                    onclick: move |_| active_tab.set(ViewTab::Prefixes),

                                    "前缀分组 ({prefix_stats.read().len()})"
                                }
                            }

                            if active_tab() == ViewTab::Keys {
                                div {
                                    border: "1px solid {colors.border}",
                                    border_radius: "8px",
                                    overflow: "hidden",
                                    max_height: "400px",
                                    overflow_y: "auto",
                                    background: "{colors.background_secondary}",

                                    table {
                                        width: "100%",
                                        border_collapse: "collapse",

                                        thead {
                                            tr {
                                                background: "{colors.surface_low}",

                                                th { padding: "10px 12px", text_align: "left", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", "Key" }
                                                th { padding: "10px 12px", text_align: "right", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", width: "120px", "Memory" }
                                                th { padding: "10px 12px", text_align: "center", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", width: "80px", "Type" }
                                                th { padding: "10px 12px", text_align: "center", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", width: "80px", "TTL" }
                                            }
                                        }

                                        tbody {
                                            for (idx, entry) in entries.read().iter().enumerate() {
                                                tr {
                                                    key: "{entry.key}",
                                                    background: if idx % 2 == 0 { colors.background } else { colors.background_secondary },
                                                    cursor: "pointer",
                                                    onclick: {
                                                        let key = entry.key.clone();
                                                        let on_select_key = on_select_key.clone();
                                                        let on_close = on_close.clone();
                                                        move |_| {
                                                            on_select_key.call(key.clone());
                                                            on_close.call(());
                                                        }
                                                    },

                                                    td { padding: "8px 12px", color: "{colors.text}", font_size: "12px", font_family: "monospace", border_bottom: "1px solid {colors.border}", word_break: "break_all", "{entry.key}" }
                                                    td { padding: "8px 12px", color: "{colors.accent}", font_size: "12px", font_weight: "600", text_align: "right", border_bottom: "1px solid {colors.border}", white_space: "nowrap", "{format_bytes(entry.memory_bytes)}" }
                                                    td { padding: "8px 12px", color: "{colors.text_secondary}", font_size: "11px", text_align: "center", border_bottom: "1px solid {colors.border}", white_space: "nowrap", "{entry.key_type}" }
                                                    td { padding: "8px 12px", color: "{colors.text_secondary}", font_size: "11px", text_align: "center", border_bottom: "1px solid {colors.border}", white_space: "nowrap", "{format_ttl(entry.ttl)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div {
                                    border: "1px solid {colors.border}",
                                    border_radius: "8px",
                                    overflow: "hidden",
                                    max_height: "400px",
                                    overflow_y: "auto",
                                    background: "{colors.background_secondary}",

                                    div {
                                        padding: "10px 12px",
                                        background: "{colors.surface_low}",
                                        border_bottom: "1px solid {colors.border}",
                                        display: "flex",
                                        justify_content: "space_between",

                                        span {
                                            color: "{colors.text}",
                                            font_size: "12px",

                                            "前缀分组总内存: {format_bytes(prefix_total_memory)}"
                                        }
                                    }

                                    table {
                                        width: "100%",
                                        border_collapse: "collapse",

                                        thead {
                                            tr {
                                                th { padding: "10px 12px", text_align: "left", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", "Prefix" }
                                                th { padding: "10px 12px", text_align: "right", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", width: "80px", "Keys" }
                                                th { padding: "10px 12px", text_align: "right", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", width: "120px", "Memory" }
                                                th { padding: "10px 12px", text_align: "center", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", width: "100px", "Types" }
                                                th { padding: "10px 12px", text_align: "center", color: "{colors.text_secondary}", font_size: "11px", font_weight: "600", border_bottom: "1px solid {colors.border}", width: "80px", "Avg TTL" }
                                            }
                                        }

                                        tbody {
                                            for (idx, stat) in prefix_stats.read().iter().enumerate() {
                                                tr {
                                                    key: "{stat.prefix}",
                                                    background: if idx % 2 == 0 { colors.background } else { colors.background_secondary },

                                                    td { padding: "8px 12px", color: "{colors.text}", font_size: "12px", font_family: "monospace", border_bottom: "1px solid {colors.border}", "{stat.prefix}" }
                                                    td { padding: "8px 12px", color: "{colors.text_secondary}", font_size: "12px", text_align: "right", border_bottom: "1px solid {colors.border}", "{stat.key_count}" }
                                                    td { padding: "8px 12px", color: "{colors.accent}", font_size: "12px", font_weight: "600", text_align: "right", border_bottom: "1px solid {colors.border}", "{format_bytes(stat.memory_bytes)}" }
                                                    td { padding: "8px 12px", color: "{colors.text_secondary}", font_size: "11px", text_align: "center", border_bottom: "1px solid {colors.border}", "{stat.types.join(\", \")}" }
                                                    td { padding: "8px 12px", color: "{colors.text_secondary}", font_size: "11px", text_align: "center", border_bottom: "1px solid {colors.border}", "{format_ttl(stat.avg_ttl_secs as i64)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    ScanState::Completed { checked, found: 0 } => rsx! {
                        div {
                            padding: "24px",
                            text_align: "center",
                            background: "{colors.background_secondary}",
                            border: "1px dashed {colors.border}",
                            border_radius: "8px",

                            div {
                                color: "{colors.text}",
                                font_size: "14px",
                                font_weight: "600",

                                {i18n.read().t("No matching keys found")}
                            }

                            div {
                                color: "{colors.text_secondary}",
                                font_size: "12px",
                                margin_top: "8px",

                                "扫描了 {checked} 个 key，但没有符合条件的大 key"
                            }
                        }
                    },
                    _ => rsx! {
                        div {
                            padding: "24px",
                            text_align: "center",
                            background: "{colors.background_secondary}",
                            border: "1px dashed {colors.border}",
                            border_radius: "8px",

                            div {
                                color: "{colors.text}",
                                font_size: "14px",
                                font_weight: "600",

                                {i18n.read().t("Ready to scan")}
                            }

                            div {
                                color: "{colors.text_secondary}",
                                font_size: "12px",
                                margin_top: "8px",

                                "设置扫描参数后点击扫描按钮开始分析。采样率小于 1.0 可以减少 Redis 负载。"
                            }
                        }
                    },
                }
            }
        }
    }
}
