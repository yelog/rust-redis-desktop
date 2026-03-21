use crate::serialization::java_converters::{
    get_collection_display_name, is_collection_type, is_map_type, is_set_type,
};
use crate::serialization::{simplify_class_name, Content, Parser};
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

    rsx! {
        div {
            padding: "16px",
            background: "#252526",
            border: "1px solid #3c3c3c",
            border_radius: "8px",

            div {
                display: "flex",
                align_items: "center",
                justify_content: "space_between",
                margin_bottom: "16px",
                padding_bottom: "12px",
                border_bottom: "1px solid #3c3c3c",

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

                    svg {
                        width: "20",
                        height: "20",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "#22c55e",
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
                        color: "#22c55e",
                        font_size: "14px",
                        font_weight: "600",

                        "Java 序列化对象"
                    }
                }

                div {
                    display: "flex",
                    gap: "4px",

                    button {
                        font_size: "11px",
                        padding: "4px 8px",
                        background: "#333",
                        border: "1px solid #444",
                        color: "#ccc",
                        border_radius: "4px",
                        cursor: "pointer",

                        onclick: move |_| expand_all.set(true),

                        "全部展开"
                    }

                    button {
                        font_size: "11px",
                        padding: "4px 8px",
                        background: "#333",
                        border: "1px solid #444",
                        color: "#ccc",
                        border_radius: "4px",
                        cursor: "pointer",

                        onclick: move |_| expand_all.set(false),

                        "全部折叠"
                    }
                }
            }

            {
                let result = parse_result.read();
                match result.as_ref() {
                    Some(Ok(content)) => {
                        match content {
                            Content::Object(value) => {
                                match serde_json::to_value(value) {
                                    Ok(json) => rsx! { 
                                        JsonTreeNode { 
                                            value: json, 
                                            depth: 0,
                                            expand_all,
                                        } 
                                    },
                                    Err(_) => rsx! {
                                        div {
                                            color: "#888",
                                            "序列化失败"
                                        }
                                    },
                                }
                            },
                            Content::Block(bytes) => rsx! {
                                div {
                                    color: "#888",
                                    "原始数据块 ({bytes.len()} 字节)"
                                }
                            },
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div {
                            color: "#f44336",
                            padding: "12px",
                            background: "#2d1f1f",
                            border_radius: "4px",

                            "解析错误: {e}"
                        }
                    },
                    None => rsx! {
                        div {
                            color: "#888",
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
fn JsonTreeNode(value: JsonValue, depth: usize, expand_all: Signal<bool>) -> Element {
    match &value {
        JsonValue::Object(obj) => rsx! {
            JsonObjectNode { obj: obj.clone(), depth, expand_all }
        },
        JsonValue::Array(arr) => rsx! {
            JsonArrayNode { arr: arr.clone(), depth, expand_all }
        },
        _ => rsx! {
            JsonPrimitiveNode { value: value.clone(), depth }
        }
    }
}

#[component]
fn JsonObjectNode(
    obj: serde_json::Map<String, JsonValue>,
    depth: usize,
    expand_all: Signal<bool>,
) -> Element {
    let mut expanded = use_signal(|| depth == 0 || *expand_all.read());
    
    let has_fields = !obj.is_empty();
    let indent = depth * 16;

    let is_java_object = obj.contains_key("class") && obj.contains_key("fields");
    let class_name = obj.get("class").and_then(|v| v.as_str()).unwrap_or("Object");
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

    rsx! {
        div {
            margin_left: "{indent}px",
            margin_bottom: "4px",

            div {
                display: "flex",
                align_items: "center",
                gap: "6px",
                padding: "2px 0",
                cursor: if has_fields || has_annotations { "pointer" } else { "default" },

                onclick: move |_| {
                    if has_fields || has_annotations {
                        expanded.toggle();
                    }
                },

                if has_fields || has_annotations {
                    span {
                        color: "#888",
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
                    color: if is_std_lib { "#dcdcaa" } else { "#4ec9b0" },
                    font_size: "13px",
                    font_weight: "600",

                    "{display_name}"
                }

                if has_full_name && !is_std_lib {
                    span {
                        color: "#6b7280",
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
                            color: "#888",
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
                    if !fields.is_empty() {
                        div {
                            margin_left: "18px",
                            border_left: "1px solid #3c3c3c",
                            padding_left: "8px",
                            margin_top: "2px",

                            for (key, val) in fields.iter() {
                                div {
                                    display: "flex",
                                    align_items: "flex_start",
                                    gap: "6px",
                                    padding: "1px 0",

                                    span {
                                        color: "#9cdcfe",
                                        font_size: "12px",
                                        min_width: "80px",

                                        "{key}:"
                                    }

                                    JsonTreeNode {
                                        value: val.clone(),
                                        depth: depth + 1,
                                        expand_all,
                                    }
                                }
                            }
                        }
                    }
                }

                if has_annotations {
                    div {
                        margin_left: "18px",
                        border_left: "1px dashed #444",
                        padding_left: "8px",
                        margin_top: "4px",

                        div {
                            color: "#f59e0b",
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
                                        color: "#666",
                                        font_size: "11px",
                                        min_width: "30px",

                                        "[{i}]"
                                    }

                                    JsonTreeNode {
                                        value: annotation.clone(),
                                        depth: depth + 1,
                                        expand_all,
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
                    color: "#888",
                    font_size: "12px",
                    width: "12px",

                    if *expanded.read() { "▼" } else { "▶" }
                }

                span {
                    color: "#dcdcaa",
                    font_size: "12px",

                    "[{len}]"
                }
            }

            if *expanded.read() {
                div {
                    margin_left: "18px",
                    border_left: "1px solid #3c3c3c",
                    padding_left: "8px",

                    for (i, item) in arr.iter().enumerate() {
                        div {
                            display: "flex",
                            align_items: "flex_start",
                            gap: "6px",
                            padding: "1px 0",

                            span {
                                color: "#888",
                                font_size: "11px",
                                min_width: "30px",

                                "{i}:"
                            }

                            JsonTreeNode {
                                value: item.clone(),
                                depth: depth + 1,
                                expand_all,
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
        JsonValue::Null => ("null".to_string(), "#808080"),
        JsonValue::Bool(v) => (v.to_string(), "#569cd6"),
        JsonValue::Number(v) => (v.to_string(), "#b5cea8"),
        JsonValue::String(s) => {
            let display = if s.len() > 100 {
                format!("\"{}...\"", &s[..97])
            } else {
                format!("\"{}\"", s)
            };
            (display, "#ce9178")
        }
        _ => (value.to_string(), "#dcdcaa"),
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
            background: if *copied.read() { "#1e4620" } else { "transparent" },
            border: "1px solid #3c3c3c",
            color: if *copied.read() { "#22c55e" } else { "#666" },
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