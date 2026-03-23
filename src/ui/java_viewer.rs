use crate::serialization::java_converters::{
    get_collection_display_name, is_collection_type, is_map_type, is_set_type,
};
use crate::serialization::{extract_inner_value, simplify_class_name, Content, Parser};
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_ERROR,
    COLOR_ERROR_BG, COLOR_OUTLINE_VARIANT, COLOR_SUCCESS, COLOR_SUCCESS_BG_ALT, COLOR_TEXT,
    COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE, COLOR_WARNING, SYNTAX_BOOLEAN, SYNTAX_COMMENT,
    SYNTAX_FUNCTION, SYNTAX_KEY, SYNTAX_NULL, SYNTAX_NUMBER, SYNTAX_STRING, SYNTAX_TYPE,
};
use crate::ui::json_viewer::JsonViewer;
use arboard::Clipboard;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;
use std::io::Cursor;

#[component]
pub fn JavaSerializedViewer(data: Vec<u8>) -> Element {
    let parse_result = use_resource(move || {
        let data = data.clone();
        async move {
            let cursor = Cursor::new(data);
            match Parser::new(cursor) {
                Ok(mut parser) => match parser.read() {
                    Ok(content) => Ok(content),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
    });

    let mut expand_all = use_signal(|| true);
    let mut search_query = use_signal(String::new);
    let mut show_raw_json = use_signal(|| false);

    rsx! {
        div {
            padding: "16px",
            background: COLOR_BG_SECONDARY,
            border: "1px solid {COLOR_BORDER}",
            border_radius: "8px",
            max_height: "600px",
            overflow_y: "auto",

            div {
                display: "flex",
                align_items: "center",
                justify_content: "space_between",
                margin_bottom: "12px",
                padding_bottom: "12px",
                border_bottom: "1px solid {COLOR_BORDER}",

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

                    svg {
                        width: "20",
                        height: "20",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: COLOR_SUCCESS,
                        stroke_width: "2",

                        rect {
                            x: "3",
                            y: "3",
                            width: "18",
                            height: "18",
                            rx: "2",
                        }
                        path {
                            d: "M9 9h6v6H9z",
                        }
                    }

                    span {
                        color: COLOR_ACCENT,
                        font_size: "14px",
                        font_weight: "600",

                        "Java 序列化对象"
                    }
                }
            }

            div {
                display: "flex",
                gap: "8px",
                margin_bottom: "12px",
                flex_wrap: "wrap",

                input {
                    r#type: "text",
                    placeholder: "搜索字段名...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value()),
                    font_size: "12px",
                    padding: "4px 8px",
                    background: COLOR_BG,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "4px",
                    color: COLOR_TEXT,
                    flex: "1",
                    min_width: "150px",
                }

                button {
                    font_size: "11px",
                    padding: "4px 8px",
                    background: COLOR_BG_TERTIARY,
                    border: "1px solid {COLOR_BORDER}",
                    color: COLOR_TEXT,
                    border_radius: "4px",
                    cursor: "pointer",

                    onclick: move |_| expand_all.set(true),

                    "全部展开"
                }

                button {
                    font_size: "11px",
                    padding: "4px 8px",
                    background: COLOR_BG_TERTIARY,
                    border: "1px solid {COLOR_BORDER}",
                    color: COLOR_TEXT,
                    border_radius: "4px",
                    cursor: "pointer",

                    onclick: move |_| expand_all.set(false),

                    "全部折叠"
                }

                button {
                    font_size: "11px",
                    padding: "4px 8px",
                    background: if *show_raw_json.read() { COLOR_SUCCESS_BG_ALT } else { COLOR_BG_TERTIARY },
                    border: "1px solid {COLOR_BORDER}",
                    color: if *show_raw_json.read() { COLOR_SUCCESS } else { COLOR_TEXT },
                    border_radius: "4px",
                    cursor: "pointer",

                    onclick: move |_| show_raw_json.toggle(),

                    "JSON"
                }
            }

            {
                let result = parse_result.read();
                match result.as_ref() {
                    Some(Ok(content)) => {
                        match content {
                            Content::Object(value) => {
                                match serde_json::to_value(value) {
                                    Ok(json) => {
                                        let json = extract_inner_value(json);
                                        if *show_raw_json.read() {
                                            let json_str = serde_json::to_string_pretty(&json).unwrap_or_else(|_| "序列化失败".to_string());
                                            rsx! {
                                                JsonViewer {
                                                    value: json_str,
                                                    on_change: move |_| {},
                                                    editable: false,
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                JsonTreeNode {
                                                    value: json,
                                                    depth: 0,
                                                    expand_all,
                                                    search_query: search_query.read().to_lowercase(),
                                                }
                                            }
                                        }
                                    },
                                    Err(_) => rsx! {
                                        div {
                                            color: COLOR_TEXT_SECONDARY,
                                            "序列化失败"
                                        }
                                    },
                                }
                            },
                            Content::Block(bytes) => rsx! {
                                div {
                                    color: COLOR_TEXT_SECONDARY,
                                    "原始数据块 ({bytes.len()} 字节)"
                                }
                            },
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div {
                            color: COLOR_ERROR,
                            padding: "12px",
                            background: COLOR_ERROR_BG,
                            border_radius: "4px",

                            "解析错误: {e}"
                        }
                    },
                    None => rsx! {
                        div {
                            color: COLOR_TEXT_SECONDARY,
                            text_align: "center",
                            padding: "20px",

                            "解析中..."
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn JsonTreeNode(
    value: JsonValue,
    depth: usize,
    expand_all: Signal<bool>,
    search_query: String,
) -> Element {
    match &value {
        JsonValue::Object(obj) => rsx! {
            JsonObjectNode {
                obj: obj.clone(),
                depth,
                expand_all,
                search_query,
            }
        },
        JsonValue::Array(arr) => rsx! {
            JsonArrayNode {
                arr: arr.clone(),
                depth,
                expand_all,
                search_query,
            }
        },
        _ => rsx! {
            JsonPrimitiveNode { value: value.clone(), depth }
        },
    }
}

fn matches_search(key: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    key.to_lowercase().contains(query)
}

#[component]
fn JsonObjectNode(
    obj: serde_json::Map<String, JsonValue>,
    depth: usize,
    expand_all: Signal<bool>,
    search_query: String,
) -> Element {
    let mut expanded = use_signal(|| depth == 0 || *expand_all.read());

    let has_fields = !obj.is_empty();
    let indent = depth * 16;

    let is_java_object = obj.contains_key("class") && obj.contains_key("fields");
    let class_name = obj
        .get("class")
        .and_then(|v| v.as_str())
        .unwrap_or("Object");
    let simple_name = simplify_class_name(class_name);
    let has_full_name = class_name != simple_name;

    let is_collection = is_collection_type(class_name);
    let is_map = is_map_type(class_name);
    let is_set = is_set_type(class_name);
    let is_std_lib = is_collection || is_map || is_set;

    let display_name = if is_std_lib {
        get_collection_display_name(class_name)
    } else {
        &simple_name
    };

    let fields_obj = obj.get("fields").and_then(|f| f.as_object());
    let field_count = fields_obj.map(|f| f.len()).unwrap_or(0);

    let annotation_count = obj
        .get("annotations")
        .and_then(|a| a.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let has_annotations = annotation_count > 0;
    let has_content = has_fields || has_annotations;

    let class_matches_search = matches_search(class_name, &search_query);

    rsx! {
        div {
            margin_left: "{indent}px",
            margin_bottom: "4px",

            div {
                display: "flex",
                align_items: "center",
                gap: "6px",
                padding: "2px 0",
                cursor: if has_content { "pointer" } else { "default" },

                onclick: move |_| {
                    if has_content {
                        expanded.toggle();
                    }
                },

                if has_content {
                    span {
                        color: COLOR_TEXT_SUBTLE,
                        font_size: "12px",
                        width: "12px",

                        if *expanded.read() { "▼" } else { "▶" }
                    }
                } else {
                    span {
                        width: "12px",
                    }
                }

                span {
                    color: if is_std_lib { SYNTAX_TYPE } else { SYNTAX_FUNCTION },
                    font_size: "13px",
                    font_weight: "600",

                    "{display_name}"
                }

                if has_full_name && !is_std_lib {
                    span {
                        color: SYNTAX_COMMENT,
                        font_size: "10px",

                        "({class_name})"
                    }
                }

                {
                    let count_str = if field_count > 0 {
                        format!("{{{}}}", field_count)
                    } else if has_annotations {
                        format!("{{{} annotations}}", annotation_count)
                    } else {
                        "{}".to_string()
                    };

                    rsx! {
                        span {
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",

                            "{count_str}"
                        }
                    }
                }

                if is_java_object {
                    CopyButton { text: class_name.to_string() }
                }
            }

            if *expanded.read() {
                if let Some(fields) = fields_obj {
                    {
                        let filtered_fields: Vec<_> = fields
                            .iter()
                            .filter(|(k, _)| class_matches_search || matches_search(k, &search_query))
                            .collect();

                        if !filtered_fields.is_empty() {
                            rsx! {
                                div {
                                    margin_left: "18px",
                                    border_left: "1px solid {COLOR_BORDER}",
                                    padding_left: "8px",
                                    margin_top: "2px",

                                    for (key, val) in filtered_fields.iter() {
                                        {
                                            let highlight = !search_query.is_empty() && matches_search(key, &search_query);
                                            let val_clone = (*val).clone();
                                            rsx! {
                                                div {
                                                    display: "flex",
                                                    align_items: "flex_start",
                                                    gap: "6px",
                                                    padding: "1px 0",

                                                    span {
                                                        color: if highlight { COLOR_WARNING } else { SYNTAX_KEY },
                                                        font_size: "12px",
                                                        min_width: "80px",

                                                        "{key}:"
                                                    }

                                                    JsonTreeNode {
                                                        value: val_clone,
                                                        depth: depth + 1,
                                                        expand_all,
                                                        search_query: search_query.clone(),
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }

                if has_annotations {
                    div {
                        margin_left: "18px",
                        border_left: "1px dashed {COLOR_OUTLINE_VARIANT}",
                        padding_left: "8px",
                        margin_top: "4px",

                        div {
                            color: COLOR_WARNING,
                            font_size: "10px",
                            margin_bottom: "4px",

                            "Annotations ({annotation_count})"
                        }

                        if let Some(annotations) = obj.get("annotations").and_then(|a| a.as_array()) {
                            for (i, annotation) in annotations.iter().enumerate() {
                                div {
                                    display: "flex",
                                    align_items: "flex_start",
                                    gap: "6px",
                                    padding: "1px 0",

                                    span {
                                        color: COLOR_TEXT_SUBTLE,
                                        font_size: "11px",
                                        min_width: "30px",

                                        "[{i}]"
                                    }

                                    JsonTreeNode {
                                        value: annotation.clone(),
                                        depth: depth + 1,
                                        expand_all,
                                        search_query: search_query.clone(),
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

#[component]
fn JsonArrayNode(
    arr: Vec<JsonValue>,
    depth: usize,
    expand_all: Signal<bool>,
    search_query: String,
) -> Element {
    let mut expanded = use_signal(|| depth == 0 || *expand_all.read());
    let indent = depth * 16;
    let len = arr.len();

    rsx! {
        div {
            margin_left: "{indent}px",

            div {
                display: "flex",
                align_items: "center",
                gap: "6px",
                padding: "2px 0",
                cursor: "pointer",

                onclick: move |_| expanded.toggle(),

                span {
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "12px",
                    width: "12px",

                    if *expanded.read() { "▼" } else { "▶" }
                }

                span {
                    color: SYNTAX_TYPE,
                    font_size: "12px",

                    "[{len}]"
                }
            }

            if *expanded.read() {
                div {
                    margin_left: "18px",
                    border_left: "1px solid {COLOR_BORDER}",
                    padding_left: "8px",

                    for (i, item) in arr.iter().enumerate() {
                        div {
                            display: "flex",
                            align_items: "flex_start",
                            gap: "6px",
                            padding: "1px 0",

                            span {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "11px",
                                min_width: "30px",

                                "{i}:"
                            }

                            JsonTreeNode {
                                value: item.clone(),
                                depth: depth + 1,
                                expand_all,
                                search_query: search_query.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn JsonPrimitiveNode(value: JsonValue, depth: usize) -> Element {
    let indent = depth * 16;
    let (text, color) = match &value {
        JsonValue::Null => ("null".to_string(), SYNTAX_NULL),
        JsonValue::Bool(v) => (v.to_string(), SYNTAX_BOOLEAN),
        JsonValue::Number(v) => (v.to_string(), SYNTAX_NUMBER),
        JsonValue::String(s) => {
            let display = if s.len() > 100 {
                format!("\"{}...\"", &s[..97])
            } else {
                format!("\"{}\"", s)
            };
            (display, SYNTAX_STRING)
        }
        _ => (value.to_string(), SYNTAX_TYPE),
    };

    let copy_text = match &value {
        JsonValue::String(s) => Some(s.clone()),
        _ => None,
    };

    rsx! {
        div {
            display: "flex",
            align_items: "center",
            gap: "6px",
            margin_left: "{indent}px",

            span {
                color: "{color}",
                font_size: "12px",
                font_family: "Consolas, monospace",
                word_break: "break_all",

                "{text}"
            }

            if let Some(s) = copy_text {
                CopyButton { text: s }
            }
        }
    }
}

#[component]
fn CopyButton(text: String) -> Element {
    let mut copied = use_signal(|| false);

    rsx! {
        button {
            font_size: "9px",
            padding: "1px 4px",
            background: if *copied.read() { COLOR_SUCCESS_BG_ALT } else { "transparent" },
            border: "1px solid {COLOR_BORDER}",
            color: if *copied.read() { COLOR_SUCCESS } else { COLOR_TEXT_SUBTLE },
            border_radius: "2px",
            cursor: "pointer",

            onclick: move |_| {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&text).is_ok() {
                        copied.set(true);
                    }
                }
            },

            if *copied.read() { "✓" } else { "复制" }
        }
    }
}
