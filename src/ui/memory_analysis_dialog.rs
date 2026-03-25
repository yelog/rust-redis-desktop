use crate::connection::ConnectionPool;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::icons::{IconSearch, IconX};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct MemoryEntry {
    pub key: String,
    pub memory_bytes: u64,
    pub key_type: String,
}

#[derive(Clone, PartialEq)]
enum ScanState {
    Idle,
    Scanning(usize),
    Completed(usize),
    Error(String),
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

#[component]
pub fn MemoryAnalysisDialog(
    connection_pool: ConnectionPool,
    colors: ThemeColors,
    on_select_key: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut entries = use_signal(Vec::<MemoryEntry>::new);
    let mut state = use_signal(|| ScanState::Idle);
    let mut cancel_flag = use_signal(|| None::<Arc<AtomicBool>>);
    let mut min_size_filter = use_signal(|| 0u64);
    let mut pattern = use_signal(|| "*".to_string());
    let sort_desc = use_signal(|| true);

    let start_scan = {
        let connection_pool = connection_pool.clone();
        move |_| {
            let pool = connection_pool.clone();
            let flag = Arc::new(AtomicBool::new(true));
            cancel_flag.set(Some(flag.clone()));
            entries.set(Vec::new());

            let pattern_val = pattern();
            let min_size = min_size_filter();
            state.set(ScanState::Scanning(0));

            spawn(async move {
                let flag = flag.clone();
                let mut cursor: u64 = 0;
                let mut all_entries = Vec::new();
                let mut scanned = 0usize;

                loop {
                    if !flag.load(Ordering::Relaxed) {
                        state.set(ScanState::Idle);
                        return;
                    }

                    let result = pool.scan_keys_with_cursor(&pattern_val, cursor, 100).await;

                    match result {
                        Ok((next_cursor, keys)) => {
                            scanned += keys.len();
                            state.set(ScanState::Scanning(scanned));

                            for key in keys {
                                if !flag.load(Ordering::Relaxed) {
                                    state.set(ScanState::Idle);
                                    return;
                                }

                                if let Ok(Some(bytes)) = pool.memory_usage(&key).await {
                                    if bytes >= min_size {
                                        let key_type = pool
                                            .get_key_type(&key)
                                            .await
                                            .map(|t| format!("{:?}", t))
                                            .unwrap_or_else(|_| "Unknown".to_string());

                                        all_entries.push(MemoryEntry {
                                            key,
                                            memory_bytes: bytes,
                                            key_type,
                                        });
                                    }
                                }
                            }

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

                all_entries.sort_by(|a, b| {
                    if sort_desc() {
                        b.memory_bytes.cmp(&a.memory_bytes)
                    } else {
                        a.memory_bytes.cmp(&b.memory_bytes)
                    }
                });

                let total = all_entries.len();
                entries.set(all_entries);
                state.set(ScanState::Completed(total));
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

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "760px".to_string(),
            max_height: "85vh".to_string(),
            title: "内存分析".to_string(),

            div {
                display: "flex",
                flex_direction: "column",
                gap: "16px",

                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    line_height: "1.5",

                    "Scan keys by pattern, filter out small values, and jump directly to the selected result."
                }

                div {
                    padding: "16px",
                    background: "{colors.background_secondary}",
                    border: "1px solid {colors.border}",
                    border_radius: "12px",
                    display: "flex",
                    flex_direction: "column",
                    gap: "14px",

                    div {
                        display: "flex",
                        flex_wrap: "wrap",
                        gap: "12px",

                        div {
                            flex: "1 1 320px",
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
                                padding: "10px 12px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "8px",
                                color: "{colors.text}",
                                font_size: "13px",
                                font_family: "monospace",
                                box_sizing: "border_box",
                                value: "{pattern}",
                                oninput: move |e| pattern.set(e.value()),
                            }
                        },

                        div {
                            flex: "1 1 180px",
                            min_width: "0",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "12px",
                                margin_bottom: "6px",

                                "Min Size (bytes)"
                            }

                            input {
                                width: "100%",
                                padding: "10px 12px",
                                background: "{colors.background}",
                                border: "1px solid {colors.border}",
                                border_radius: "8px",
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
                    }

                    div {
                        display: "flex",
                        flex_wrap: "wrap",
                        align_items: "center",
                        justify_content: "space_between",
                        gap: "12px",

                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: "4px",

                            span {
                                color: "{colors.text_secondary}",
                                font_size: "12px",

                                "Pattern supports glob syntax such as `user:*` or `cache:*`."
                            }

                            span {
                                color: "{colors.text_subtle}",
                                font_size: "12px",

                                "Results stay sorted by memory usage so the largest keys remain at the top."
                            }
                        }

                        div {
                            display: "flex",
                            align_items: "center",
                            gap: "10px",
                            flex_wrap: "wrap",

                            if matches!(state(), ScanState::Scanning(_)) {
                                button {
                                    padding: "10px 16px",
                                    background: "{colors.error}",
                                    color: "{colors.primary_text}",
                                    border: "none",
                                    border_radius: "8px",
                                    cursor: "pointer",
                                    font_size: "13px",
                                    font_weight: "600",
                                    display: "flex",
                                    align_items: "center",
                                    gap: "6px",
                                    onclick: stop_scan,

                                    IconX { size: Some(14), color: Some(colors.primary_text.to_string()) }
                                    "Stop"
                                }
                            } else {
                                button {
                                    padding: "10px 16px",
                                    background: "{colors.primary}",
                                    color: "{colors.primary_text}",
                                    border: "none",
                                    border_radius: "8px",
                                    cursor: "pointer",
                                    font_size: "13px",
                                    font_weight: "600",
                                    display: "flex",
                                    align_items: "center",
                                    gap: "6px",
                                    onclick: start_scan.clone(),

                                    IconSearch { size: Some(14), color: Some(colors.primary_text.to_string()) }
                                    "Scan"
                                }
                            }
                        }
                    }
                }

                match state() {
                    ScanState::Scanning(count) => rsx! {
                        div {
                            padding: "18px 20px",
                            background: "{colors.background_secondary}",
                            border: "1px solid {colors.border}",
                            border_radius: "12px",
                            display: "flex",
                            flex_direction: "column",
                            gap: "6px",

                            div {
                                color: "{colors.text}",
                                font_size: "15px",
                                font_weight: "600",

                                "Scanning in progress"
                            }

                            div {
                                color: "{colors.text_secondary}",
                                font_size: "13px",
                                line_height: "1.5",

                                "{count} keys checked so far. You can keep waiting or stop the scan at any time."
                            }
                        }
                    },
                    ScanState::Error(ref e) => rsx! {
                        div {
                            padding: "18px 20px",
                            background: "{colors.error_bg}",
                            border: "1px solid {colors.error}",
                            border_radius: "12px",
                            display: "flex",
                            flex_direction: "column",
                            gap: "6px",

                            div {
                                color: "{colors.error}",
                                font_size: "15px",
                                font_weight: "600",

                                "Scan failed"
                            }

                            div {
                                color: "{colors.error}",
                                font_size: "13px",
                                line_height: "1.5",

                                "{e}"
                            }
                        }
                    },
                    ScanState::Completed(total) if entries.read().is_empty() => rsx! {
                        div {
                            padding: "28px 20px",
                            text_align: "center",
                            background: "{colors.background_secondary}",
                            border: "1px dashed {colors.border}",
                            border_radius: "12px",
                            display: "flex",
                            flex_direction: "column",
                            gap: "8px",

                            div {
                                color: "{colors.text}",
                                font_size: "15px",
                                font_weight: "600",

                                "No matching keys"
                            }

                            div {
                                color: "{colors.text_secondary}",
                                font_size: "13px",
                                line_height: "1.5",

                                "The scan completed successfully, but no keys matched the current pattern and minimum size filter. Checked result count: {total}."
                            }
                        }
                    },
                    _ => rsx! {
                        if !entries.read().is_empty() {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "12px",

                                div {
                                    padding: "12px 14px",
                                    background: "{colors.background_secondary}",
                                    border: "1px solid {colors.border}",
                                    border_radius: "12px",
                                    display: "flex",
                                    flex_wrap: "wrap",
                                    justify_content: "space_between",
                                    align_items: "center",
                                    gap: "10px",

                                    div {
                                        display: "flex",
                                        flex_direction: "column",
                                        gap: "4px",

                                        span {
                                            color: "{colors.text}",
                                            font_size: "14px",
                                            font_weight: "600",

                                            "{entries.read().len()} keys found"
                                        }

                                        span {
                                            color: "{colors.text_secondary}",
                                            font_size: "12px",

                                            "Click any row to open the corresponding key."
                                        }
                                    }

                                    span {
                                        color: "{colors.accent}",
                                        font_size: "13px",
                                        font_weight: "600",

                                        "Total memory: {format_bytes(total_memory)}"
                                    }
                                }

                                div {
                                    border: "1px solid {colors.border}",
                                    border_radius: "12px",
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

                                                th {
                                                    padding: "11px 14px",
                                                    text_align: "left",
                                                    color: "{colors.text_secondary}",
                                                    font_size: "12px",
                                                    font_weight: "600",
                                                    border_bottom: "1px solid {colors.border}",

                                                    "Key"
                                                }

                                                th {
                                                    padding: "11px 14px",
                                                    text_align: "right",
                                                    color: "{colors.text_secondary}",
                                                    font_size: "12px",
                                                    font_weight: "600",
                                                    border_bottom: "1px solid {colors.border}",
                                                    width: "140px",

                                                    "Memory"
                                                }

                                                th {
                                                    padding: "11px 14px",
                                                    text_align: "center",
                                                    color: "{colors.text_secondary}",
                                                    font_size: "12px",
                                                    font_weight: "600",
                                                    border_bottom: "1px solid {colors.border}",
                                                    width: "110px",

                                                    "Type"
                                                }
                                            }
                                        }

                                        tbody {
                                            for (idx, entry) in entries.read().iter().enumerate() {
                                                tr {
                                                    key: "{entry.key}",
                                                    background: if idx % 2 == 0 {
                                                        colors.background
                                                    } else {
                                                        colors.background_secondary
                                                    },
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

                                                    td {
                                                        padding: "10px 14px",
                                                        color: "{colors.text}",
                                                        font_size: "13px",
                                                        font_family: "monospace",
                                                        border_bottom: "1px solid {colors.border}",
                                                        word_break: "break_all",
                                                        line_height: "1.5",

                                                        "{entry.key}"
                                                    }

                                                    td {
                                                        padding: "10px 14px",
                                                        color: "{colors.accent}",
                                                        font_size: "13px",
                                                        font_weight: "600",
                                                        text_align: "right",
                                                        border_bottom: "1px solid {colors.border}",
                                                        white_space: "nowrap",

                                                        "{format_bytes(entry.memory_bytes)}"
                                                    }

                                                    td {
                                                        padding: "10px 14px",
                                                        color: "{colors.text_secondary}",
                                                        font_size: "12px",
                                                        text_align: "center",
                                                        border_bottom: "1px solid {colors.border}",
                                                        white_space: "nowrap",

                                                        "{entry.key_type}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if entries.read().is_empty() && matches!(state(), ScanState::Idle) {
                            div {
                                padding: "28px 20px",
                                text_align: "center",
                                background: "{colors.background_secondary}",
                                border: "1px dashed {colors.border}",
                                border_radius: "12px",
                                display: "flex",
                                flex_direction: "column",
                                gap: "8px",

                                div {
                                    color: "{colors.text}",
                                    font_size: "15px",
                                    font_weight: "600",

                                    "Ready to scan"
                                }

                                div {
                                    color: "{colors.text_secondary}",
                                    font_size: "13px",
                                    line_height: "1.5",

                                    "Set a pattern and minimum size, then click Scan. Matching keys will appear here in descending memory order."
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
