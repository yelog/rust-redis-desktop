use crate::theme::{
    COLOR_BG, COLOR_BORDER, COLOR_BUTTON_SECONDARY, COLOR_BUTTON_SECONDARY_BORDER, COLOR_ERROR,
    COLOR_ERROR_BG, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_TEXT, COLOR_TEXT_CONTRAST,
    COLOR_TEXT_SECONDARY, SYNTAX_BOOLEAN, SYNTAX_BRACKET, SYNTAX_KEY, SYNTAX_NULL, SYNTAX_NUMBER,
    SYNTAX_STRING,
};
use crate::ui::{copy_text_to_clipboard, icons::IconCopy, ToastManager};
use dioxus::prelude::*;
use serde_json::Value;

pub fn is_json_content(content: &str) -> bool {
    let trimmed = content.trim();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return false;
    }
    serde_json::from_str::<Value>(trimmed).is_ok()
}

pub fn format_json(content: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|e| format!("JSON 解析错误: {}", e))?;
    serde_json::to_string_pretty(&value).map_err(|e| format!("JSON 格式化错误: {}", e))
}

pub fn minify_json(content: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|e| format!("JSON 解析错误: {}", e))?;
    serde_json::to_string(&value).map_err(|e| format!("JSON 压缩错误: {}", e))
}

fn highlight_json_value(value: &Value, indent: usize) -> Vec<HighlightSegment> {
    let mut segments = Vec::new();
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Object(map) => {
            segments.push(HighlightSegment {
                text: "{\n".to_string(),
                token_type: TokenType::Bracket,
            });

            let entries: Vec<_> = map.iter().collect();
            for (i, (key, val)) in entries.iter().enumerate() {
                segments.push(HighlightSegment {
                    text: format!("{}  \"{}\"", indent_str, key),
                    token_type: TokenType::Key,
                });
                segments.push(HighlightSegment {
                    text: ": ".to_string(),
                    token_type: TokenType::Bracket,
                });
                segments.extend(highlight_json_value(val, indent + 1));

                if i < entries.len() - 1 {
                    segments.push(HighlightSegment {
                        text: ",\n".to_string(),
                        token_type: TokenType::Bracket,
                    });
                } else {
                    segments.push(HighlightSegment {
                        text: "\n".to_string(),
                        token_type: TokenType::Bracket,
                    });
                }
            }

            segments.push(HighlightSegment {
                text: format!("{}}}", indent_str),
                token_type: TokenType::Bracket,
            });
        }
        Value::Array(arr) => {
            segments.push(HighlightSegment {
                text: "[\n".to_string(),
                token_type: TokenType::Bracket,
            });

            for (i, item) in arr.iter().enumerate() {
                segments.push(HighlightSegment {
                    text: format!("{}  ", indent_str),
                    token_type: TokenType::Bracket,
                });
                segments.extend(highlight_json_value(item, indent + 1));

                if i < arr.len() - 1 {
                    segments.push(HighlightSegment {
                        text: ",\n".to_string(),
                        token_type: TokenType::Bracket,
                    });
                } else {
                    segments.push(HighlightSegment {
                        text: "\n".to_string(),
                        token_type: TokenType::Bracket,
                    });
                }
            }

            segments.push(HighlightSegment {
                text: format!("{}]", indent_str),
                token_type: TokenType::Bracket,
            });
        }
        Value::String(s) => {
            segments.push(HighlightSegment {
                text: format!("\"{}\"", s),
                token_type: TokenType::String,
            });
        }
        Value::Number(n) => {
            segments.push(HighlightSegment {
                text: n.to_string(),
                token_type: TokenType::Number,
            });
        }
        Value::Bool(b) => {
            segments.push(HighlightSegment {
                text: b.to_string(),
                token_type: TokenType::Boolean,
            });
        }
        Value::Null => {
            segments.push(HighlightSegment {
                text: "null".to_string(),
                token_type: TokenType::Null,
            });
        }
    }

    segments
}

#[derive(Clone, Copy, PartialEq)]
pub enum TokenType {
    Key,
    String,
    Number,
    Boolean,
    Null,
    Bracket,
}

#[derive(Clone)]
pub struct HighlightSegment {
    pub text: String,
    pub token_type: TokenType,
}

fn token_color(token_type: TokenType) -> &'static str {
    match token_type {
        TokenType::Key => SYNTAX_KEY,
        TokenType::String => SYNTAX_STRING,
        TokenType::Number => SYNTAX_NUMBER,
        TokenType::Boolean => SYNTAX_BOOLEAN,
        TokenType::Null => SYNTAX_NULL,
        TokenType::Bracket => SYNTAX_BRACKET,
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Pretty,
    Raw,
}

#[component]
pub fn JsonViewer(value: String, on_change: EventHandler<String>, editable: bool) -> Element {
    let mut is_editing = use_signal(|| false);
    let mut temp_value = use_signal(String::new);
    let mut view_mode = use_signal(ViewMode::default);
    let mut parse_error = use_signal(|| None::<String>);
    let mut toast_manager = use_context::<Signal<ToastManager>>();

    let display_value = value.clone();
    let formatted = format_json(&display_value).unwrap_or_else(|e| {
        parse_error.set(Some(e.clone()));
        display_value.clone()
    });

    let json_value: Option<Value> = serde_json::from_str(&display_value).ok();
    let segments = json_value
        .as_ref()
        .map(|v| highlight_json_value(v, 0))
        .unwrap_or_default();
    let is_pretty = view_mode() == ViewMode::Pretty;
    let is_raw = view_mode() == ViewMode::Raw;
    let editing = is_editing();

    let copy_to_clipboard = {
        let val = formatted.clone();
        move |_| {
            if let Err(e) = copy_value_to_clipboard(&val) {
                toast_manager.write().error(&format!("复制失败：{}", e));
            } else {
                toast_manager.write().success("已复制到剪贴板");
            }
        }
    };

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100%",

            div {
                display: "flex",
                justify_content: "space_between",
                align_items: "center",
                gap: "8px",
                margin_bottom: "10px",
                flex_wrap: "wrap",

                div {
                    display: "flex",
                    gap: "8px",
                    align_items: "center",
                    flex_wrap: "wrap",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "11px",
                        font_weight: "700",
                        text_transform: "uppercase",
                        letter_spacing: "0.08em",

                        "JSON"
                    }

                    button {
                        height: "28px",
                        padding: "0 10px",
                        background: if is_pretty { COLOR_PRIMARY } else { COLOR_BUTTON_SECONDARY },
                        color: if is_pretty { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                        border: if is_pretty {
                            "1px solid transparent".to_string()
                        } else {
                            format!("1px solid {}", COLOR_BUTTON_SECONDARY_BORDER)
                        },
                        border_radius: "6px",
                        cursor: "pointer",
                        font_size: "12px",
                        font_weight: "500",
                        onclick: move |_| view_mode.set(ViewMode::Pretty),

                        "格式化"
                    }

                    button {
                        height: "28px",
                        padding: "0 10px",
                        background: if is_raw { COLOR_PRIMARY } else { COLOR_BUTTON_SECONDARY },
                        color: if is_raw { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                        border: if is_raw {
                            "1px solid transparent".to_string()
                        } else {
                            format!("1px solid {}", COLOR_BUTTON_SECONDARY_BORDER)
                        },
                        border_radius: "6px",
                        cursor: "pointer",
                        font_size: "12px",
                        font_weight: "500",
                        onclick: move |_| view_mode.set(ViewMode::Raw),

                        "压缩"
                    }

                    if editable && !editing {
                        button {
                            height: "28px",
                            padding: "0 10px",
                            background: COLOR_PRIMARY,
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            font_weight: "500",
                            onclick: move |_| {
                                temp_value.set(formatted.clone());
                                is_editing.set(true);
                                parse_error.set(None);
                            },

                            "编辑"
                        }
                    }
                }

                button {
                    margin_left: "auto",
                    flex_shrink: "0",
                    height: "28px",
                    padding: "0 10px",
                    background: COLOR_BUTTON_SECONDARY,
                    color: COLOR_TEXT,
                    border: "1px solid {COLOR_BUTTON_SECONDARY_BORDER}",
                    border_radius: "6px",
                    cursor: "pointer",
                    display: "flex",
                    align_items: "center",
                    gap: "4px",
                    font_size: "12px",
                    font_weight: "500",
                    onclick: copy_to_clipboard,

                    IconCopy { size: Some(14) }
                    "复制"
                }
            }

            if let Some(error) = parse_error() {
                div {
                    color: COLOR_ERROR,
                    font_size: "12px",
                    margin_bottom: "8px",
                    padding: "8px",
                    background: COLOR_ERROR_BG,
                    border_radius: "4px",

                    "{error}"
                }
            }

            if is_editing() {
                div {
                    display: "flex",
                    flex_direction: "column",
                    flex: "1",
                    min_height: "0",

                    textarea {
                        flex: "1",
                        min_height: "0",
                        padding: "12px",
                        background: COLOR_BG,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "8px",
                        color: COLOR_TEXT,
                        font_family: "Consolas, 'Courier New', monospace",
                        font_size: "14px",
                        resize: "none",
                        value: "{temp_value}",
                        oninput: move |e| temp_value.set(e.value()),
                    }

                    div {
                        display: "flex",
                        gap: "8px",
                        margin_top: "8px",

                        button {
                            padding: "6px 12px",
                            background: COLOR_SUCCESS,
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            onclick: move |_| {
                                let edited = temp_value();
                                match serde_json::from_str::<Value>(&edited) {
                                    Ok(_) => {
                                        match minify_json(&edited) {
                                            Ok(minified) => {
                                                on_change.call(minified);
                                                is_editing.set(false);
                                                parse_error.set(None);
                                            }
                                            Err(e) => parse_error.set(Some(e)),
                                        }
                                    }
                                    Err(e) => parse_error.set(Some(format!("JSON 无效: {}", e))),
                                }
                            },

                            "保存"
                        }

                        button {
                            padding: "6px 12px",
                            background: COLOR_BUTTON_SECONDARY,
                            color: COLOR_TEXT,
                            border: "1px solid {COLOR_BUTTON_SECONDARY_BORDER}",
                            border_radius: "6px",
                            cursor: "pointer",
                            onclick: move |_| {
                                is_editing.set(false);
                                parse_error.set(None);
                            },

                            "取消"
                        }
                    }
                }
            } else {
                div {
                    flex: "1",
                    min_height: "0",
                    overflow: "auto",
                    padding: "14px",
                    background: COLOR_BG,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "8px",

                    pre {
                        margin: "0",
                        font_family: "Consolas, 'Courier New', monospace",
                        font_size: "14px",
                        line_height: "1.5",
                        white_space: "pre-wrap",
                        word_break: "break-all",

                        if view_mode() == ViewMode::Pretty {
                            for segment in segments {
                                span {
                                    color: token_color(segment.token_type),

                                    "{segment.text}"
                                }
                            }
                        } else {
                            span {
                                color: COLOR_TEXT,

                                "{display_value}"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn copy_value_to_clipboard(value: &str) -> Result<(), String> {
    copy_text_to_clipboard(value)
}
