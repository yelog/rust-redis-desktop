use crate::connection::ConnectionPool;
use crate::redis::ExportFormat;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use arboard::Clipboard;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ExportTarget {
    pub key: String,
    pub is_folder: bool,
}

#[component]
pub fn ExportDialog(
    connection_pool: ConnectionPool,
    targets: Vec<ExportTarget>,
    colors: ThemeColors,
    on_close: EventHandler<()>,
) -> Element {
    let mut export_format = use_signal(|| ExportFormat::Json);
    let mut processing = use_signal(|| false);
    let mut exported_content = use_signal(String::new);
    let mut keys_to_export = use_signal(Vec::<String>::new);
    let mut loaded = use_signal(|| false);
    let mut export_done = use_signal(|| false);
    let mut error_msg = use_signal(String::new);

    use_effect({
        let pool = connection_pool.clone();
        let targets = targets.clone();
        move || {
            if !loaded() {
                let pool = pool.clone();
                let targets = targets.clone();
                spawn(async move {
                    let mut all_keys = Vec::new();

                    for target in targets.iter() {
                        if target.is_folder {
                            match pool.scan_keys(&format!("{}*", target.key), 1000).await {
                                Ok(keys) => all_keys.extend(keys),
                                Err(e) => tracing::error!("Failed to scan keys: {}", e),
                            }
                        } else {
                            all_keys.push(target.key.clone());
                        }
                    }

                    keys_to_export.set(all_keys);
                    loaded.set(true);
                });
            }
        }
    });

    let do_export = {
        let pool = connection_pool.clone();
        move |_| {
            let pool = pool.clone();
            processing.set(true);
            error_msg.set(String::new());
            
            spawn(async move {
                let keys = keys_to_export.read().clone();
                let format = *export_format.read();
                
                match pool.export_keys(&keys, format).await {
                    Ok(content) => {
                        exported_content.set(content);
                        export_done.set(true);
                    }
                    Err(e) => {
                        error_msg.set(format!("导出失败: {}", e));
                    }
                }
                
                processing.set(false);
            });
        }
    };

    let copy_to_clipboard = move |_| {
        if let Ok(mut clipboard) = Clipboard::new() {
            let _ = clipboard.set_text(exported_content());
        }
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "500px".to_string(),

            h2 {
                color: "{colors.text}",
                margin_bottom: "20px",
                font_size: "18px",
                "导出数据"
            }

            if !loaded() {
                div {
                    padding: "40px",
                    text_align: "center",
                    color: "{colors.text_secondary}",
                    "正在扫描键..."
                }
            } else if export_done() {
                div {
                    div {
                        margin_bottom: "12px",
                        
                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "13px",
                            margin_bottom: "8px",
                            "导出格式"
                        }

                        div {
                            display: "flex",
                            gap: "16px",

                            label {
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                color: "{colors.text}",
                                font_size: "13px",
                                cursor: "pointer",

                                input {
                                    r#type: "radio",
                                    name: "export_format",
                                    checked: matches!(*export_format.read(), ExportFormat::Json),
                                    onchange: move |_| export_format.set(ExportFormat::Json),
                                }
                                "JSON"
                            }

                            label {
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                color: "{colors.text}",
                                font_size: "13px",
                                cursor: "pointer",

                                input {
                                    r#type: "radio",
                                    name: "export_format",
                                    checked: matches!(*export_format.read(), ExportFormat::Commands),
                                    onchange: move |_| export_format.set(ExportFormat::Commands),
                                }
                                "Redis Commands"
                            }
                        }
                    }

                    div {
                        margin_bottom: "12px",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        "共 {keys_to_export.read().len()} 个键"
                    }

                    textarea {
                        width: "100%",
                        height: "200px",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "12px",
                        font_family: "monospace",
                        box_sizing: "border_box",
                        resize: "vertical",
                        readonly: true,
                        value: "{exported_content}",
                    }

                    if !error_msg.read().is_empty() {
                        div {
                            margin_top: "12px",
                            padding: "8px 12px",
                            background: "rgba(239, 68, 68, 0.1)",
                            border: "1px solid rgba(239, 68, 68, 0.3)",
                            border_radius: "4px",
                            color: "#ef4444",
                            font_size: "12px",
                            "{error_msg}"
                        }
                    }

                    div {
                        display: "flex",
                        gap: "8px",
                        margin_top: "16px",

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "{colors.accent}",
                            color: "{colors.primary_text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: copy_to_clipboard,
                            "复制到剪贴板"
                        }

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: move |_| on_close.call(()),
                            "关闭"
                        }
                    }
                }
            } else {
                div {
                    div {
                        margin_bottom: "16px",
                        
                        label {
                            display: "block",
                            color: "{colors.text_secondary}",
                            font_size: "13px",
                            margin_bottom: "8px",
                            "导出格式"
                        }

                        div {
                            display: "flex",
                            gap: "16px",

                            label {
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                color: "{colors.text}",
                                font_size: "13px",
                                cursor: "pointer",

                                input {
                                    r#type: "radio",
                                    name: "export_format",
                                    checked: matches!(*export_format.read(), ExportFormat::Json),
                                    onchange: move |_| export_format.set(ExportFormat::Json),
                                }
                                "JSON"
                            }

                            label {
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                color: "{colors.text}",
                                font_size: "13px",
                                cursor: "pointer",

                                input {
                                    r#type: "radio",
                                    name: "export_format",
                                    checked: matches!(*export_format.read(), ExportFormat::Commands),
                                    onchange: move |_| export_format.set(ExportFormat::Commands),
                                }
                                "Redis Commands"
                            }
                        }
                    }

                    div {
                        margin_bottom: "16px",
                        padding: "12px",
                        background: "{colors.background_tertiary}",
                        border_radius: "4px",
                        
                        div {
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            margin_bottom: "8px",
                            "将要导出的键:"
                        }

                        div {
                            color: "{colors.text}",
                            font_size: "13px",
                            max_height: "150px",
                            overflow_y: "auto",

                            for key in keys_to_export.read().iter().take(20) {
                                div {
                                    padding: "2px 0",
                                    "{key}"
                                }
                            }

                            if keys_to_export.read().len() > 20 {
                                div {
                                    color: "{colors.text_secondary}",
                                    font_size: "12px",
                                    padding_top: "8px",
                                    "... 还有 {keys_to_export.read().len() - 20} 个键"
                                }
                            }
                        }
                    }

                    div {
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "16px",
                        "共 {keys_to_export.read().len()} 个键将被导出"
                    }

                    div {
                        display: "flex",
                        gap: "8px",

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: move |_| on_close.call(()),
                            "取消"
                        }

                        button {
                            flex: "1",
                            padding: "8px",
                            background: "{colors.primary}",
                            color: "{colors.primary_text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: if processing() { "wait" } else { "pointer" },
                            font_size: "13px",
                            disabled: processing(),
                            onclick: do_export,

                            if processing() { "导出中..." } else { "导出" }
                        }
                    }
                }
            }
        }
    }
}