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
    let mut sort_desc = use_signal(|| true);

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

                    let result = pool
                        .scan_keys_with_cursor(&pattern_val, cursor, 100)
                        .await;

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
            width: "700px".to_string(),
            max_height: "85vh".to_string(),

            h3 {
                color: "{colors.text}",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "8px",
                font_size: "18px",

                IconSearch { size: Some(20) }
                "Memory Analysis"
            }

            div {
                margin_bottom: "16px",
                display: "flex",
                gap: "12px",
                align_items: "flex_end",

                div {
                    flex: "1",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "4px",

                        "Pattern"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        font_family: "monospace",
                        box_sizing: "border_box",
                        value: "{pattern}",
                        oninput: move |e| pattern.set(e.value()),
                    }
                }

                div {
                    width: "150px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "4px",

                        "Min Size (bytes)"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
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

                if matches!(state(), ScanState::Scanning(_)) {
                    button {
                        padding: "8px 16px",
                        background: "{colors.error}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        display: "flex",
                        align_items: "center",
                        gap: "6px",
                        onclick: stop_scan,

                        IconX { size: Some(14) }
                        "Stop"
                    }
                } else {
                    button {
                        padding: "8px 16px",
                        background: "{colors.primary}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        display: "flex",
                        align_items: "center",
                        gap: "6px",
                        onclick: start_scan.clone(),

                        IconSearch { size: Some(14) }
                        "Scan"
                    }
                }
            }

            match state() {
                ScanState::Scanning(count) => rsx! {
                    div {
                        padding: "20px",
                        text_align: "center",
                        color: "{colors.text_secondary}",

                        "Scanning... {count} keys checked"
                    }
                },
                ScanState::Error(ref e) => rsx! {
                    div {
                        padding: "20px",
                        background: "rgba(239, 68, 68, 0.1)",
                        border: "1px solid rgba(239, 68, 68, 0.3)",
                        border_radius: "4px",
                        color: "{colors.error}",

                        "{e}"
                    }
                },
                ScanState::Completed(total) if entries.read().is_empty() => rsx! {
                    div {
                        padding: "20px",
                        text_align: "center",
                        color: "{colors.text_secondary}",

                        "No keys found matching the criteria"
                    }
                },
                _ => rsx! {
                    if !entries.read().is_empty() {
                        div {
                            margin_bottom: "12px",
                            padding: "10px 12px",
                            background: "{colors.background_tertiary}",
                            border_radius: "4px",
                            display: "flex",
                            justify_content: "space_between",

                            span {
                                color: "{colors.text_secondary}",
                                font_size: "12px",

                                "{entries.read().len()} keys found"
                            }

                            span {
                                color: "{colors.text_secondary}",
                                font_size: "12px",

                                "Total: {format_bytes(total_memory)}"
                            }
                        }

                        div {
                            border: "1px solid {colors.border}",
                            border_radius: "4px",
                            overflow: "hidden",
                            max_height: "400px",
                            overflow_y: "auto",

                            table {
                                width: "100%",
                                border_collapse: "collapse",

                                thead {
                                    tr {
                                        background: "{colors.background_tertiary}",

                                        th {
                                            padding: "10px 12px",
                                            text_align: "left",
                                            color: "{colors.text_secondary}",
                                            font_size: "12px",
                                            font_weight: "600",
                                            border_bottom: "1px solid {colors.border}",

                                            "Key"
                                        }

                                        th {
                                            padding: "10px 12px",
                                            text_align: "right",
                                            color: "{colors.text_secondary}",
                                            font_size: "12px",
                                            font_weight: "600",
                                            border_bottom: "1px solid {colors.border}",
                                            width: "120px",

                                            "Memory"
                                        }

                                        th {
                                            padding: "10px 12px",
                                            text_align: "center",
                                            color: "{colors.text_secondary}",
                                            font_size: "12px",
                                            font_weight: "600",
                                            border_bottom: "1px solid {colors.border}",
                                            width: "80px",

                                            "Type"
                                        }
                                    }
                                }

                                tbody {
                                    for entry in entries.read().iter() {
                                        tr {
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
                                                padding: "8px 12px",
                                                color: "{colors.text}",
                                                font_size: "13px",
                                                font_family: "monospace",
                                                border_bottom: "1px solid {colors.border}",

                                                "{entry.key}"
                                            }

                                            td {
                                                padding: "8px 12px",
                                                color: "{colors.accent}",
                                                font_size: "13px",
                                                font_weight: "600",
                                                text_align: "right",
                                                border_bottom: "1px solid {colors.border}",

                                                "{format_bytes(entry.memory_bytes)}"
                                            }

                                            td {
                                                padding: "8px 12px",
                                                color: "{colors.text_secondary}",
                                                font_size: "12px",
                                                text_align: "center",
                                                border_bottom: "1px solid {colors.border}",

                                                "{entry.key_type}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if entries.read().is_empty() && matches!(state(), ScanState::Idle) {
                        div {
                            padding: "40px",
                            text_align: "center",
                            color: "{colors.text_secondary}",

                            "Set pattern and minimum size, then click Scan"
                        }
                    }
                },
            }
        }
    }
}