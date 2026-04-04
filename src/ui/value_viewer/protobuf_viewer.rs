use crate::protobuf_schema::PROTO_REGISTRY;
use crate::serialization::{parse_to_json, SerializationFormat};
use crate::theme::{
    COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_ERROR, COLOR_ERROR_BG, COLOR_PRIMARY, COLOR_TEXT,
    COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
};
use crate::ui::JsonViewer;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;

#[component]
pub(super) fn ProtobufViewer(data: Vec<u8>) -> Element {
    let mut selected_message = use_signal(|| String::new());
    let mut import_error = use_signal(|| None::<String>);

    let registry = PROTO_REGISTRY();
    let messages = registry.list_messages();
    let has_schema = !messages.is_empty();

    let json_result: Option<JsonValue> = if !selected_message().is_empty() {
        registry.decode_with_schema(&data, &selected_message()).ok()
    } else {
        parse_to_json(&data, SerializationFormat::Protobuf)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    };

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "12px",

            div {
                display: "flex",
                gap: "8px",
                align_items: "center",
                flex_wrap: "wrap",

                button {
                    padding: "6px 12px",
                    background: COLOR_PRIMARY,
                    color: COLOR_TEXT_CONTRAST,
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "12px",

                    onclick: move |_| {
                        spawn(async move {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Proto", &["proto"])
                                .pick_file()
                            {
                                let mut reg = PROTO_REGISTRY.write();
                                match reg.import_file(&path) {
                                    Ok(names) => {
                                        if !names.is_empty() {
                                            selected_message.set(names[0].clone());
                                        }
                                        import_error.set(None);
                                    }
                                    Err(e) => import_error.set(Some(e)),
                                }
                            }
                        });
                    },

                    "导入 .proto 文件"
                }

                if has_schema {
                    select {
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        value: "{selected_message}",
                        onchange: move |e| selected_message.set(e.value()),

                        option { value: "", "Raw 解析" }
                        for message in messages.iter() {
                            option {
                                value: message.full_name.clone(),
                                selected: selected_message() == message.full_name,
                                "{message.name}"
                            }
                        }
                    }

                    button {
                        padding: "4px 8px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "11px",
                        onclick: move |_| {
                            PROTO_REGISTRY.write().clear();
                            selected_message.set(String::new());
                        },
                        "清除 Schema"
                    }
                }
            }

            if let Some(error) = import_error() {
                div {
                    padding: "10px 12px",
                    background: COLOR_ERROR_BG,
                    border_radius: "8px",
                    color: COLOR_ERROR,
                    font_size: "12px",
                    "Schema 导入失败：{error}"
                }
            }

            if has_schema {
                div {
                    padding: "8px 12px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "4px",
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "11px",
                    "已加载 {messages.len()} 个消息类型"
                }
            }

            if let Some(json) = json_result {
                JsonViewer {
                    value: serde_json::to_string_pretty(&json).unwrap_or_default(),
                    editable: false,
                    on_change: move |_| {},
                }
            } else {
                div {
                    padding: "12px 14px",
                    background: COLOR_BG_TERTIARY,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "8px",
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",
                    "Protobuf 解析失败"
                }
            }
        }
    }
}
