use crate::connection::{ConnectionError, ConnectionPool};
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[component]
pub fn PatternDeleteDialog(
    connection_pool: ConnectionPool,
    initial_pattern: String,
    colors: ThemeColors,
    on_confirm: EventHandler<usize>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut pattern = use_signal(|| initial_pattern);
    let mut preview_keys = use_signal(Vec::<String>::new);
    let mut scanning = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let mut scan_complete = use_signal(|| false);
    let mut total_found = use_signal(|| 0usize);
    let mut scan_progress = use_signal(|| Arc::new(AtomicUsize::new(0)));

    let do_scan = {
        let pool = connection_pool.clone();
        let pattern = pattern.clone();
        let mut scanning = scanning.clone();
        let mut scan_complete = scan_complete.clone();
        let mut preview_keys = preview_keys.clone();
        let mut total_found = total_found.clone();
        let mut scan_progress = scan_progress.clone();
        move || {
            let pool = pool.clone();
            let pattern_val = pattern();
            scanning.set(true);
            scan_complete.set(false);
            preview_keys.set(Vec::new());
            total_found.set(0);

            let progress = Arc::new(AtomicUsize::new(0));
            scan_progress.set(progress.clone());

            spawn(async move {
                let mut all_keys = Vec::new();
                let pattern_for_scan = if pattern_val.contains('*') {
                    pattern_val.clone()
                } else {
                    format!("{}*", pattern_val)
                };

                let result = pool
                    .scan_keys_with_progress(&pattern_for_scan, 500, {
                        let progress = progress.clone();
                        move |count| {
                            progress.store(count, Ordering::Relaxed);
                        }
                    })
                    .await;

                match result {
                    Ok(count) => {
                        let keys = pool
                            .scan_keys(&pattern_for_scan, 1000)
                            .await
                            .unwrap_or_default();
                        all_keys = keys;
                        total_found.set(count);
                        scan_complete.set(true);
                    }
                    Err(e) => {
                        tracing::error!("Failed to scan keys: {}", e);
                    }
                }

                preview_keys.set(all_keys);
                scanning.set(false);
            });
        }
    };

    let display_keys: Vec<String> = preview_keys.read().iter().take(50).cloned().collect();
    let remaining = preview_keys.read().len().saturating_sub(50);
    let current_progress = scan_progress.read().load(Ordering::Relaxed);

    let scan_handler = {
        let mut do_scan = do_scan.clone();
        move |_| do_scan()
    };

    let keypress_handler = {
        let mut do_scan = do_scan.clone();
        move |e: Event<KeyboardData>| {
            if e.data().key() == Key::Enter {
                do_scan();
            }
        }
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "550px".to_string(),
            max_height: "85vh".to_string(),
            title: "按模式批量删除".to_string(),

            div {
                margin_bottom: "16px",

                label {
                    display: "block",
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    margin_bottom: "8px",

                    "Key 模式 (支持通配符 * 和 ?)"
                }

                div {
                    display: "flex",
                    gap: "8px",

                    input {
                        flex: "1",
                        padding: "10px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        font_family: "monospace",
                        box_sizing: "border_box",
                        value: "{pattern}",
                        oninput: move |e| {
                            pattern.set(e.value());
                            scan_complete.set(false);
                        },
                        onkeypress: keypress_handler.clone(),
                    }

                    button {
                        padding: "10px 16px",
                        background: "{colors.primary}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: if scanning() { "wait" } else { "pointer" },
                        font_size: "13px",
                        disabled: scanning() || pattern().is_empty(),
                        onclick: scan_handler.clone(),

                        if scanning() { "扫描中..." } else { "预览" }
                    }
                }
            }

            if scanning() {
                div {
                    padding: "20px",
                    text_align: "center",
                    color: "{colors.text_secondary}",

                    "正在扫描... 已找到 {current_progress} 个 key"
                }
            } else if scan_complete() {
                if preview_keys.read().is_empty() {
                    div {
                        padding: "20px",
                        text_align: "center",
                        color: "{colors.text_secondary}",

                        "没有找到匹配的 key"
                    }
                } else {
                    div {
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        padding: "12px",
                        margin_bottom: "16px",
                        max_height: "300px",
                        overflow_y: "auto",

                        div {
                            display: "flex",
                            justify_content: "space_between",
                            align_items: "center",
                            margin_bottom: "8px",

                            span {
                                color: "{colors.text_secondary}",
                                font_size: "12px",

                                "找到 {total_found()} 个匹配的 key："
                            }
                        }

                        for key in display_keys.iter() {
                            div {
                                color: "{colors.text}",
                                font_size: "12px",
                                padding: "3px 0",
                                font_family: "monospace",

                                "• {key}"
                            }
                        }

                        if remaining > 0 {
                            div {
                                color: "{colors.text_subtle}",
                                font_size: "11px",
                                margin_top: "8px",

                                "... 还有 {remaining} 个 key"
                            }
                        }
                    }

                    div {
                        padding: "12px",
                        background: "rgba(239, 68, 68, 0.1)",
                        border: "1px solid rgba(239, 68, 68, 0.3)",
                        border_radius: "4px",
                        margin_bottom: "16px",

                        span {
                            color: "{colors.error}",
                            font_size: "12px",

                            "⚠️ 警告：此操作将删除所有匹配的 key，且不可恢复！"
                        }
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
                            cursor: if deleting() { "wait" } else { "pointer" },
                            font_size: "13px",
                            disabled: deleting(),
                            onclick: {
                                let pool = connection_pool.clone();
                                let on_confirm = on_confirm.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let on_confirm = on_confirm.clone();
                                    let keys = preview_keys.read().clone();
                                    deleting.set(true);

                                    spawn(async move {
                                        let mut deleted_count = 0;

                                        for key in keys.iter() {
                                            match pool.delete_key(key).await {
                                                Ok(true) => deleted_count += 1,
                                                Ok(false) => {}
                                                Err(e) => {
                                                    if !matches!(e, ConnectionError::ReadonlyMode) {
                                                        tracing::error!("Failed to delete {}: {}", key, e);
                                                    }
                                                }
                                            }
                                        }

                                        deleting.set(false);
                                        on_confirm.call(deleted_count);
                                    });
                                }
                            },

                            if deleting() {
                                "删除中..."
                            } else {
                                "确认删除 ({total_found()})"
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
                            disabled: deleting(),
                            onclick: move |_| on_cancel.call(()),

                            "取消"
                        }
                    }
                }
            } else {
                div {
                    padding: "20px",
                    text_align: "center",
                    color: "{colors.text_subtle}",
                    font_size: "13px",

                    "输入 key 模式并点击\"预览\"查看匹配的 key"
                }
            }
        }
    }
}
