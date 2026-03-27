use crate::connection::ConnectionPool;
use crate::protobuf_schema::{PROTO_REGISTRY, ProtoRegistry};
use crate::redis::{KeyInfo, KeyType};
use crate::serialization::{
    detect_serialization_format, is_java_serialization, is_protobuf_data, parse_to_json, SerializationFormat,
};
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_ERROR,
    COLOR_ERROR_BG, COLOR_PRIMARY, COLOR_ROW_CREATE_BG, COLOR_ROW_EDIT_BG, COLOR_SUCCESS,
    COLOR_SUCCESS_BG, COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SOFT,
    COLOR_TEXT_SUBTLE, COLOR_WARNING,
};
use crate::ui::editable_field::EditableField;
use crate::ui::icons::{IconCopy, IconEdit, IconTrash};
use crate::ui::java_viewer::JavaSerializedViewer;
use crate::ui::json_viewer::{is_json_content, JsonViewer};
use crate::ui::pagination::LargeKeyWarning;
use crate::ui::{copy_text_to_clipboard, ServerInfoPanel, ToastManager};
use dioxus::prelude::*;
use dioxus::html::geometry::WheelDelta;
use serde_json;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

const PAGE_SIZE: usize = 100;

const LARGE_KEY_THRESHOLD: usize = 1000;
const STATUS_SUCCESS_BG: &str = COLOR_SUCCESS_BG;
const STATUS_ERROR_BG: &str = COLOR_ERROR_BG;
const ROW_CREATE_BG: &str = COLOR_ROW_CREATE_BG;
const ROW_EDIT_BG: &str = COLOR_ROW_EDIT_BG;

#[derive(Clone, Default)]
pub struct PreviewImageData {
    pub data_uri: String,
    pub format: String,
    pub size: String,
}

pub static PREVIEW_IMAGE: GlobalSignal<Option<PreviewImageData>> = Signal::global(|| None);

#[component]
pub fn ImagePreview() -> Element {
    let preview = PREVIEW_IMAGE();

    let Some(ref data) = preview else {
        return rsx! {};
    };

    let data_uri = data.data_uri.clone();
    let format = data.format.clone();
    let size = data.size.clone();

    let data_uri_for_save = data_uri.clone();
    let data_uri_for_img = data_uri.clone();
    let format_for_save = format.clone();
    let mut zoom_level = use_signal(|| 1.0f32);

    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            background: "rgba(0, 0, 0, 0.9)",
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            justify_content: "center",
            z_index: "9999",
            animation: "fadeIn 0.2s ease-out",

            onclick: move |_| {
                *PREVIEW_IMAGE.write() = None;
                zoom_level.set(1.0);
            },

            onkeydown: move |e: Event<KeyboardData>| {
                if e.data().key() == Key::Escape {
                    e.prevent_default();
                    e.stop_propagation();
                    *PREVIEW_IMAGE.write() = None;
                    zoom_level.set(1.0);
                }
            },

            style { {r#"
                @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                }
                @keyframes scaleIn {
                    from { transform: scale(0.9); opacity: 0; }
                    to { transform: scale(1); opacity: 1; }
                }
            "#} }

            div {
                position: "absolute",
                top: "16px",
                right: "16px",
                display: "flex",
                gap: "8px",
                z_index: "10",

                button {
                    padding: "8px 16px",
                    background: "rgba(255, 255, 255, 0.1)",
                    color: "white",
                    border: "1px solid rgba(255, 255, 255, 0.2)",
                    border_radius: "6px",
                    cursor: "pointer",
                    font_size: "14px",

                    onclick: move |e| {
                        e.stop_propagation();
                        let image_data = base64_decode(&data_uri_for_save);
                        let extension = format_for_save.to_lowercase();
                        let file_name = format!("image.{}", extension);
                        
                        spawn(async move {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name(&file_name)
                                .add_filter("Image", &[&extension])
                                .save_file()
                            {
                                let _ = std::fs::write(&path, &image_data);
                            }
                        });
                    },

                    "保存图片"
                }

                button {
                    padding: "8px 12px",
                    background: "rgba(255, 255, 255, 0.1)",
                    color: "white",
                    border: "1px solid rgba(255, 255, 255, 0.2)",
                    border_radius: "6px",
                    cursor: "pointer",
                    font_size: "14px",

                    onclick: move |e| {
                        e.stop_propagation();
                        zoom_level.set(1.0);
                    },

                    "重置"
                }

                button {
                    padding: "8px 16px",
                    background: "rgba(255, 255, 255, 0.1)",
                    color: "white",
                    border: "1px solid rgba(255, 255, 255, 0.2)",
                    border_radius: "6px",
                    cursor: "pointer",
                    font_size: "14px",

                    onclick: move |e| {
                        e.stop_propagation();
                        *PREVIEW_IMAGE.write() = None;
                        zoom_level.set(1.0);
                    },

                    "关闭 (Esc)"
                }
            }

            div {
                width: "100vw",
                height: "100vh",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                animation: "scaleIn 0.2s ease-out",
                overflow: "hidden",

                onclick: |e| e.stop_propagation(),

                onwheel: move |e: Event<WheelData>| {
                    e.stop_propagation();
                    let delta = match e.delta() {
                        WheelDelta::Pixels(p) => {
                            if p.y > 0.0 { -0.1 } else { 0.1 }
                        }
                        WheelDelta::Lines(l) => {
                            if l.y > 0.0 { -0.1 } else { 0.1 }
                        }
                        WheelDelta::Pages(p) => {
                            if p.y > 0.0 { -0.1 } else { 0.1 }
                        }
                    };
                    let current = zoom_level();
                    let new_zoom = (current + delta).clamp(0.1, 5.0);
                    zoom_level.set(new_zoom);
                },

                img {
                    src: "{data_uri_for_img}",
                    max_width: "90vw",
                    max_height: "85vh",
                    object_fit: "contain",
                    transform: "scale({zoom_level})",
                    transform_origin: "center",
                    transition: "transform 0.1s ease-out",
                    draggable: false,
                }
            }

            div {
                position: "absolute",
                bottom: "24px",
                left: "50%",
                transform: "translateX(-50%)",
                display: "flex",
                gap: "16px",
                align_items: "center",

                div {
                    padding: "8px 16px",
                    background: "rgba(0, 0, 0, 0.6)",
                    border_radius: "6px",
                    color: "rgba(255, 255, 255, 0.8)",
                    font_size: "13px",

                    "{format} - {size}"
                }

                div {
                    padding: "8px 16px",
                    background: "rgba(0, 0, 0, 0.6)",
                    border_radius: "6px",
                    color: "rgba(255, 255, 255, 0.8)",
                    font_size: "13px",

                    "缩放: {(zoom_level() * 100.0) as i32}%"
                }
            }
        }
    }
}

fn base64_decode(data_uri: &str) -> Vec<u8> {
    use base64::{engine::general_purpose, Engine as _};
    if let Some(base64_data) = data_uri.strip_prefix("data:") {
        if let Some(start) = base64_data.find(";base64,") {
            let base64_str = &base64_data[start + 8..];
            return general_purpose::STANDARD.decode(base64_str).unwrap_or_default();
        }
    }
    Vec::new()
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum BinaryFormat {
    #[default]
    Hex,
    Base64,
    Image,
    Protobuf,
    JavaSerialized,
    Php,
    MsgPack,
    Pickle,
    Kryo,
    Bitmap,
    Bson,
    Cbor,
}

#[derive(Clone, PartialEq)]
struct HashDeleteTarget {
    field: String,
}

fn is_binary_data(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let format = detect_serialization_format(data);
    if format != SerializationFormat::Unknown {
        return true;
    }

    let non_printable_count = data
        .iter()
        .filter(|&&b| b < 0x20 && b != 0x09 && b != 0x0A && b != 0x0D)
        .count();

    non_printable_count > data.len() / 10
}

fn format_bytes(data: &[u8], format: BinaryFormat) -> String {
    match format {
        BinaryFormat::Hex => data
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" "),
        BinaryFormat::Base64 => {
            use base64::{engine::general_purpose, Engine as _};
            general_purpose::STANDARD.encode(data)
        }
        BinaryFormat::JavaSerialized => {
            if is_java_serialization(data) {
                format!(
                    "Java 序列化对象 ({} 字节)\n\n请切换到 Java 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 Java 序列化数据".to_string()
            }
        }
        BinaryFormat::Php => {
            let detected = detect_serialization_format(data);
            if detected == SerializationFormat::Php {
                format!(
                    "PHP 序列化数据 ({} 字节)\n\n请切换到 PHP 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 PHP 序列化数据".to_string()
            }
        }
        BinaryFormat::MsgPack => {
            let detected = detect_serialization_format(data);
            if detected == SerializationFormat::MsgPack {
                format!(
                    "MessagePack 数据 ({} 字节)\n\n请切换到 MsgPack 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 MessagePack 数据".to_string()
            }
        }
        BinaryFormat::Pickle => {
            let detected = detect_serialization_format(data);
            if detected == SerializationFormat::Pickle {
                format!(
                    "Python Pickle 数据 ({} 字节)\n\n请切换到 Pickle 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 Pickle 数据".to_string()
            }
        }
        BinaryFormat::Kryo => {
            let detected = detect_serialization_format(data);
            if matches!(
                detected,
                SerializationFormat::Kryo | SerializationFormat::Fst
            ) {
                let format_name = if detected == SerializationFormat::Fst {
                    "FST"
                } else {
                    "Kryo"
                };
                format!(
                    "{} 数据 ({} 字节)\n\n请切换到 Kryo 视图查看解析结果",
                    format_name,
                    data.len()
                )
            } else {
                "非 Kryo/FST 数据".to_string()
            }
        }
        BinaryFormat::Bitmap => {
            format!(
                "Bitmap 数据 ({} 字节)\n\n请点击 Bitmap 按钮查看可视化",
                data.len()
            )
        }
        BinaryFormat::Image => {
            if let Some(format) = detect_image_format(data) {
                format!("{} 图片 ({} 字节)", format, data.len())
            } else {
                "非图片数据".to_string()
            }
        }
        BinaryFormat::Protobuf => {
            if is_protobuf_data(data) {
                format!("Protobuf 数据 ({} 字节)\n\n请切换到 Protobuf 视图查看解析结果", data.len())
            } else {
                "非 Protobuf 数据".to_string()
            }
        }
        BinaryFormat::Bson => {
            let detected = detect_serialization_format(data);
            if detected == SerializationFormat::Bson {
                format!(
                    "BSON 数据 ({} 字节)\n\n请切换到 BSON 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 BSON 数据".to_string()
            }
        }
        BinaryFormat::Cbor => {
            let detected = detect_serialization_format(data);
            if detected == SerializationFormat::Cbor {
                format!(
                    "CBOR 数据 ({} 字节)\n\n请切换到 CBOR 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 CBOR 数据".to_string()
            }
        }
    }
}

fn detect_image_format(data: &[u8]) -> Option<&'static str> {
    if data.len() < 3 {
        return None;
    }
    // PNG: 89 50 4E 47
    if data.len() >= 4 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return Some("PNG");
    }
    // JPEG: FF D8 FF
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Some("JPEG");
    }
    // GIF: 47 49 46
    if data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46 {
        return Some("GIF");
    }
    // WEBP: 52 49 46 46 ... 57 45 42 50
    if data.len() >= 12
        && data[0] == 0x52 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x46
        && data[8] == 0x57 && data[9] == 0x45 && data[10] == 0x42 && data[11] == 0x50
    {
        return Some("WEBP");
    }
    // ICO
    if data.len() > 12
        && data[0] == 0x00 && data[1] == 0x00 && data[2] == 0x00
        && (data[4..8] == [b'i', b'c', b'o', b'n'] || data[4..8] == [b'p', b'n', b'g', b' '] || data[4..8] == [b'j', b'p', b'g', b' '])
    {
        return Some("ICO");
    }
    None
}

fn copy_value_to_clipboard(value: &str) -> Result<(), String> {
    copy_text_to_clipboard(value)
}

fn sorted_hash_entries(fields: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut entries: Vec<_> = fields
        .iter()
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

fn format_memory_usage(bytes: Option<u64>) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        Some(bytes) if bytes >= GB => format!("{:.2} GB", bytes as f64 / GB as f64),
        Some(bytes) if bytes >= MB => format!("{:.2} MB", bytes as f64 / MB as f64),
        Some(bytes) if bytes >= KB => format!("{:.2} KB", bytes as f64 / KB as f64),
        Some(bytes) => format!("{bytes} B"),
        None => "--".to_string(),
    }
}

fn format_ttl_label(ttl: Option<i64>) -> String {
    match ttl {
        Some(ttl) => format!("{ttl}s"),
        None => "永久".to_string(),
    }
}

fn value_metric_label(
    key_type: &KeyType,
    string_value: &str,
    hash_value: &HashMap<String, String>,
    list_value: &[String],
    set_value: &[String],
    zset_value: &[(String, f64)],
    stream_value: &[(String, Vec<(String, String)>)],
) -> String {
    match key_type {
        KeyType::String => format!("长度: {}", string_value.chars().count()),
        KeyType::Hash => format!("字段: {}", hash_value.len()),
        KeyType::List => format!("元素: {}", list_value.len()),
        KeyType::Set => format!("成员: {}", set_value.len()),
        KeyType::ZSet => format!("成员: {}", zset_value.len()),
        KeyType::Stream => format!("条目: {}", stream_value.len()),
        KeyType::JSON => format!("JSON: {} 字符", string_value.chars().count()),
        KeyType::None => "--".to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
async fn load_key_data(
    pool: ConnectionPool,
    key: String,
    mut key_info: Signal<Option<KeyInfo>>,
    mut string_value: Signal<String>,
    mut hash_value: Signal<HashMap<String, String>>,
    mut list_value: Signal<Vec<String>>,
    mut set_value: Signal<Vec<String>>,
    mut zset_value: Signal<Vec<(String, f64)>>,
    mut stream_value: Signal<Vec<(String, Vec<(String, String)>)>>,
    mut is_binary: Signal<bool>,
    mut binary_format: Signal<BinaryFormat>,
    mut serialization_data: Signal<Option<(SerializationFormat, Vec<u8>)>>,
    mut binary_bytes: Signal<Vec<u8>>,
    mut bitmap_info: Signal<Option<crate::redis::BitmapInfo>>,
    mut loading: Signal<bool>,
    mut hash_cursor: Signal<u64>,
    mut hash_total: Signal<usize>,
    mut hash_has_more: Signal<bool>,
    mut list_has_more: Signal<bool>,
    mut list_total: Signal<usize>,
    mut set_cursor: Signal<u64>,
    mut set_total: Signal<usize>,
    mut set_has_more: Signal<bool>,
    mut zset_cursor: Signal<u64>,
    mut zset_total: Signal<usize>,
    mut zset_has_more: Signal<bool>,
) -> Result<(), String> {
    if key.is_empty() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        list_value.set(Vec::new());
        set_value.set(Vec::new());
        zset_value.set(Vec::new());
        stream_value.set(Vec::new());
        is_binary.set(false);
        serialization_data.set(None);
        bitmap_info.set(None);
        loading.set(false);
        return Ok(());
    }

    loading.set(true);

    let load_result = async {
        let info = pool
            .get_key_info(&key)
            .await
            .map_err(|e| format!("读取 key 信息失败: {e}"))?;

        tracing::info!("Key info loaded: {:?}", info.key_type);
        key_info.set(Some(info.clone()));

        match info.key_type {
            KeyType::String => {
                let bytes = pool
                    .get_string_bytes(&key)
                    .await
                    .map_err(|e| format!("读取字符串值失败: {e}"))?;

                tracing::info!("String value loaded: {} bytes", bytes.len());

                if bytes.len() >= 4 {
                    tracing::info!("First 10 bytes: {:02x?}", &bytes[..10.min(bytes.len())]);
                }

                if is_binary_data(&bytes) {
                    is_binary.set(true);
                    binary_bytes.set(bytes.clone());

                    if detect_image_format(&bytes).is_some() {
                        serialization_data.set(None);
                        binary_format.set(BinaryFormat::Image);
                    } else {
                        let detected_format = detect_serialization_format(&bytes);
                        if detected_format != SerializationFormat::Unknown {
                            tracing::info!("Detected serialization format: {:?}", detected_format);
                            serialization_data.set(Some((detected_format, bytes.clone())));
                            binary_format.set(match detected_format {
                                SerializationFormat::Java => BinaryFormat::JavaSerialized,
                                SerializationFormat::Php => BinaryFormat::Php,
                                SerializationFormat::MsgPack => BinaryFormat::MsgPack,
                                SerializationFormat::Pickle => BinaryFormat::Pickle,
                                SerializationFormat::Kryo => BinaryFormat::Kryo,
                                SerializationFormat::Fst => BinaryFormat::Kryo,
                                SerializationFormat::Protobuf => BinaryFormat::Protobuf,
                                SerializationFormat::Bson => BinaryFormat::Bson,
                                SerializationFormat::Cbor => BinaryFormat::Cbor,
                                _ => BinaryFormat::Hex,
                            });
                        } else {
                            serialization_data.set(None);
                            if bytes.len() <= 1024 {
                                if let Ok(info) = pool.get_bitmap_info(&key).await {
                                    if info.set_bits_count > 0 {
                                        bitmap_info.set(Some(info));
                                        binary_format.set(BinaryFormat::Bitmap);
                                    }
                                }
                            }
                        }
                    }

                    let formatted = format_bytes(&bytes, binary_format());
                    string_value.set(formatted);
                } else {
                    binary_bytes.set(Vec::new());
                    is_binary.set(false);
                    serialization_data.set(None);
                    match String::from_utf8(bytes) {
                        Ok(s) => string_value.set(s),
                        Err(_) => {
                            is_binary.set(true);
                            let bytes = pool
                                .get_string_bytes(&key)
                                .await
                                .map_err(|e| format!("读取字符串值失败: {e}"))?;
                            string_value.set(format_bytes(&bytes, binary_format()));
                        }
                    }
                }
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                list_has_more.set(false);
                list_total.set(0);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
            }
            KeyType::Hash => {
                let total = pool
                    .hash_len(&key)
                    .await
                    .map_err(|e| format!("获取 hash 长度失败: {e}"))?;
                let (cursor, items) = pool
                    .get_hash_page(&key, 0, PAGE_SIZE)
                    .await
                    .map_err(|e| format!("读取 hash 数据失败: {e}"))?;
                let fields: HashMap<String, String> = items.into_iter().collect();
                tracing::info!("Hash loaded: {} fields (total: {})", fields.len(), total);
                hash_value.set(fields);
                hash_cursor.set(cursor);
                hash_total.set(total as usize);
                hash_has_more.set(cursor != 0);
                string_value.set(String::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                list_has_more.set(false);
                list_total.set(0);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::List => {
                let total = pool
                    .list_len(&key)
                    .await
                    .map_err(|e| format!("获取 list 长度失败: {e}"))?;
                let count = PAGE_SIZE.min(total as usize);
                let items = pool
                    .get_list_range(&key, 0, (count - 1) as i64)
                    .await
                    .map_err(|e| format!("读取 list 数据失败: {e}"))?;
                tracing::info!("List loaded: {} items (total: {})", items.len(), total);
                list_value.set(items.clone());
                list_has_more.set(items.len() == PAGE_SIZE && items.len() < total as usize);
                list_total.set(total as usize);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::Set => {
                let total = pool
                    .set_len(&key)
                    .await
                    .map_err(|e| format!("获取 set 长度失败: {e}"))?;
                let (cursor, items) = pool
                    .get_set_page(&key, 0, PAGE_SIZE)
                    .await
                    .map_err(|e| format!("读取 set 数据失败: {e}"))?;
                tracing::info!("Set loaded: {} members (total: {})", items.len(), total);
                set_value.set(items);
                set_cursor.set(cursor);
                set_total.set(total as usize);
                set_has_more.set(cursor != 0);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                zset_value.set(Vec::new());
                stream_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                list_has_more.set(false);
                list_total.set(0);
                zset_cursor.set(0);
                zset_total.set(0);
                zset_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::ZSet => {
                let total = pool
                    .zset_card(&key)
                    .await
                    .map_err(|e| format!("获取 zset 长度失败: {e}"))?;
                let (cursor, items) = pool
                    .get_zset_page(&key, 0, PAGE_SIZE)
                    .await
                    .map_err(|e| format!("读取 zset 数据失败: {e}"))?;
                tracing::info!("ZSet loaded: {} members (total: {})", items.len(), total);
                zset_value.set(items);
                zset_cursor.set(cursor);
                zset_total.set(total as usize);
                zset_has_more.set(cursor != 0);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                stream_value.set(Vec::new());
                hash_cursor.set(0);
                hash_total.set(0);
                hash_has_more.set(false);
                list_has_more.set(false);
                list_total.set(0);
                set_cursor.set(0);
                set_total.set(0);
                set_has_more.set(false);
                is_binary.set(false);
                serialization_data.set(None);
            }
            KeyType::Stream => {
                let entries = pool
                    .stream_range(&key, "-", "+")
                    .await
                    .map_err(|e| format!("读取 stream 数据失败: {e}"))?;
                tracing::info!("Stream loaded: {} entries", entries.len());
                stream_value.set(entries);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                is_binary.set(false);
                serialization_data.set(None);
            }
            _ => {
                tracing::info!("Type: {:?}", info.key_type);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                stream_value.set(Vec::new());
                is_binary.set(false);
                serialization_data.set(None);
            }
        }

        Ok::<(), String>(())
    }
    .await;

    if load_result.is_err() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        list_value.set(Vec::new());
        set_value.set(Vec::new());
        zset_value.set(Vec::new());
        stream_value.set(Vec::new());
        is_binary.set(false);
        serialization_data.set(None);
    }

    loading.set(false);
    load_result
}

async fn load_more_hash(
    pool: ConnectionPool,
    key: String,
    mut hash_value: Signal<HashMap<String, String>>,
    cursor: u64,
    mut hash_cursor: Signal<u64>,
    mut hash_has_more: Signal<bool>,
    mut hash_loading_more: Signal<bool>,
    mut hash_total: Signal<usize>,
) {
    if hash_loading_more() || !hash_has_more() {
        return;
    }

    hash_loading_more.set(true);

    match pool.get_hash_page(&key, cursor, PAGE_SIZE).await {
        Ok((new_cursor, items)) => {
            let mut current = hash_value();
            for (field, value) in items {
                current.insert(field, value);
            }
            hash_value.set(current);
            hash_cursor.set(new_cursor);
            hash_has_more.set(new_cursor != 0);
            hash_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 hash 数据失败: {}", e);
            hash_loading_more.set(false);
        }
    }
}

async fn load_more_zset(
    pool: ConnectionPool,
    key: String,
    mut zset_value: Signal<Vec<(String, f64)>>,
    cursor: u64,
    mut zset_cursor: Signal<u64>,
    mut zset_has_more: Signal<bool>,
    mut zset_loading_more: Signal<bool>,
) {
    if zset_loading_more() || !zset_has_more() {
        return;
    }

    zset_loading_more.set(true);

    match pool.get_zset_page(&key, cursor, PAGE_SIZE).await {
        Ok((new_cursor, items)) => {
            let mut current = zset_value();
            current.extend(items);
            zset_value.set(current);
            zset_cursor.set(new_cursor);
            zset_has_more.set(new_cursor != 0);
            zset_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 zset 数据失败: {}", e);
            zset_loading_more.set(false);
        }
    }
}

async fn load_more_set(
    pool: ConnectionPool,
    key: String,
    mut set_value: Signal<Vec<String>>,
    cursor: u64,
    mut set_cursor: Signal<u64>,
    mut set_has_more: Signal<bool>,
    mut set_loading_more: Signal<bool>,
) {
    if set_loading_more() || !set_has_more() {
        return;
    }

    set_loading_more.set(true);

    match pool.get_set_page(&key, cursor, PAGE_SIZE).await {
        Ok((new_cursor, items)) => {
            let mut current = set_value();
            current.extend(items);
            set_value.set(current);
            set_cursor.set(new_cursor);
            set_has_more.set(new_cursor != 0);
            set_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 set 数据失败: {}", e);
            set_loading_more.set(false);
        }
    }
}

async fn load_more_list(
    pool: ConnectionPool,
    key: String,
    mut list_value: Signal<Vec<String>>,
    mut list_has_more: Signal<bool>,
    mut list_loading_more: Signal<bool>,
    total: usize,
) {
    if list_loading_more() || !list_has_more() {
        return;
    }

    list_loading_more.set(true);
    let offset = list_value().len() as i64;

    match pool
        .get_list_range(&key, offset, offset + PAGE_SIZE as i64 - 1)
        .await
    {
        Ok(items) => {
            let mut current = list_value();
            current.extend(items.clone());
            list_value.set(current);
            list_has_more.set(items.len() == PAGE_SIZE && list_value().len() < total);
            list_loading_more.set(false);
        }
        Err(e) => {
            tracing::error!("加载更多 list 数据失败: {}", e);
            list_loading_more.set(false);
        }
    }
}

async fn search_hash_server(
    pool: ConnectionPool,
    key: String,
    pattern: String,
    mut hash_value: Signal<HashMap<String, String>>,
    mut hash_cursor: Signal<u64>,
    mut hash_has_more: Signal<bool>,
    mut hash_loading_more: Signal<bool>,
) {
    hash_loading_more.set(true);

    let redis_pattern = if pattern.is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", pattern)
    };

    let mut cursor: u64 = 0;
    let mut all_items: HashMap<String, String> = HashMap::new();
    let max_iterations = 1000;
    let mut iterations = 0;

    loop {
        match pool
            .hash_scan_match(&key, &redis_pattern, cursor, PAGE_SIZE)
            .await
        {
            Ok((new_cursor, items)) => {
                for (field, value) in items {
                    all_items.insert(field, value);
                }
                cursor = new_cursor;
                iterations += 1;

                if cursor == 0 || iterations >= max_iterations {
                    break;
                }

                if !all_items.is_empty() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("搜索 hash 数据失败: {}", e);
                hash_loading_more.set(false);
                return;
            }
        }
    }

    hash_value.set(all_items);
    hash_cursor.set(cursor);
    hash_has_more.set(cursor != 0);
    hash_loading_more.set(false);
}

async fn search_zset_server(
    pool: ConnectionPool,
    key: String,
    pattern: String,
    mut zset_value: Signal<Vec<(String, f64)>>,
    mut zset_cursor: Signal<u64>,
    mut zset_has_more: Signal<bool>,
    mut zset_loading_more: Signal<bool>,
) {
    zset_loading_more.set(true);

    let redis_pattern = if pattern.is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", pattern)
    };

    let mut cursor: u64 = 0;
    let mut all_items: Vec<(String, f64)> = Vec::new();
    let max_iterations = 1000;
    let mut iterations = 0;

    loop {
        match pool
            .zset_scan_match(&key, &redis_pattern, cursor, PAGE_SIZE)
            .await
        {
            Ok((new_cursor, items)) => {
                all_items.extend(items);
                cursor = new_cursor;
                iterations += 1;

                if cursor == 0 || iterations >= max_iterations {
                    break;
                }

                if !all_items.is_empty() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("搜索 zset 数据失败: {}", e);
                zset_loading_more.set(false);
                return;
            }
        }
    }

    zset_value.set(all_items);
    zset_cursor.set(cursor);
    zset_has_more.set(cursor != 0);
    zset_loading_more.set(false);
}

async fn search_set_server(
    pool: ConnectionPool,
    key: String,
    pattern: String,
    mut set_value: Signal<Vec<String>>,
    mut set_cursor: Signal<u64>,
    mut set_has_more: Signal<bool>,
    mut set_loading_more: Signal<bool>,
) {
    set_loading_more.set(true);

    let redis_pattern = if pattern.is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", pattern)
    };

    let mut cursor: u64 = 0;
    let mut all_items: Vec<String> = Vec::new();
    let max_iterations = 1000;
    let mut iterations = 0;

    loop {
        match pool
            .set_scan_match(&key, &redis_pattern, cursor, PAGE_SIZE)
            .await
        {
            Ok((new_cursor, items)) => {
                all_items.extend(items);
                cursor = new_cursor;
                iterations += 1;

                if cursor == 0 || iterations >= max_iterations {
                    break;
                }

                if !all_items.is_empty() {
                    break;
                }
            }
            Err(e) => {
                tracing::error!("搜索 set 数据失败: {}", e);
                set_loading_more.set(false);
                return;
            }
        }
    }

    set_value.set(all_items);
    set_cursor.set(cursor);
    set_has_more.set(cursor != 0);
    set_loading_more.set(false);
}

#[component]
pub fn ValueViewer(
    connection_pool: ConnectionPool,
    connection_version: u32,
    selected_key: Signal<String>,
    on_refresh: EventHandler<()>,
) -> Element {
    let key_info = use_signal(|| None::<KeyInfo>);
    let mut string_value = use_signal(String::new);
    let hash_value = use_signal(HashMap::new);
    let list_value = use_signal(Vec::new);
    let set_value = use_signal(Vec::new);
    let zset_value = use_signal(Vec::new);
    let stream_value = use_signal(Vec::new);
    let loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut is_binary = use_signal(|| false);
    let mut binary_format = use_signal(BinaryFormat::default);
    let mut serialization_data = use_signal(|| None::<(SerializationFormat, Vec<u8>)>);
    let mut binary_bytes = use_signal(Vec::<u8>::new);

    let mut hash_search = use_signal(String::new);
    let mut hash_status_message = use_signal(String::new);
    let mut hash_status_error = use_signal(|| false);
    let mut editing_hash_field = use_signal(|| None::<String>);
    let mut editing_hash_key = use_signal(String::new);
    let mut editing_hash_value = use_signal(String::new);
    let mut creating_hash_row = use_signal(|| false);
    let mut new_hash_key = use_signal(String::new);
    let mut new_hash_value = use_signal(String::new);
    let mut deleting_hash_field = use_signal(|| None::<HashDeleteTarget>);
    let mut deleting_hash_field_exiting = use_signal(|| false);
    let mut hash_action = use_signal(|| None::<String>);

    let mut list_status_message = use_signal(String::new);
    let mut list_status_error = use_signal(|| false);
    let mut new_list_value = use_signal(String::new);
    let mut list_action = use_signal(|| None::<String>);
    let mut editing_list_index = use_signal(|| None::<usize>);
    let mut editing_list_value = use_signal(String::new);

    let mut set_status_message = use_signal(String::new);
    let mut set_status_error = use_signal(|| false);
    let mut new_set_member = use_signal(String::new);
    let mut set_action = use_signal(|| None::<String>);
    let mut set_search = use_signal(String::new);
    let mut deleting_set_member = use_signal(|| None::<String>);
    let mut editing_set_member = use_signal(|| None::<String>);
    let mut editing_set_member_value = use_signal(String::new);

    let mut zset_status_message = use_signal(String::new);
    let mut zset_status_error = use_signal(|| false);
    let mut new_zset_member = use_signal(String::new);
    let mut new_zset_score = use_signal(String::new);
    let mut zset_action = use_signal(|| None::<String>);
    let mut zset_search = use_signal(String::new);
    let mut deleting_zset_member = use_signal(|| None::<String>);
    let mut editing_zset_member = use_signal(|| None::<String>);
    let mut editing_zset_score = use_signal(String::new);

    let mut list_page = use_signal(|| 0usize);
    let mut list_total = use_signal(|| 0usize);
    let mut list_cursor = use_signal(|| 0u64);
    let mut list_has_more = use_signal(|| false);
    let mut list_loading_more = use_signal(|| false);
    let mut set_page = use_signal(|| 0usize);
    let mut set_total = use_signal(|| 0usize);
    let mut set_cursor = use_signal(|| 0u64);
    let mut set_has_more = use_signal(|| false);
    let mut set_loading_more = use_signal(|| false);
    let mut zset_page = use_signal(|| 0usize);
    let mut zset_total = use_signal(|| 0usize);
    let mut zset_cursor = use_signal(|| 0u64);
    let mut zset_has_more = use_signal(|| false);
    let mut zset_loading_more = use_signal(|| false);
    let mut hash_cursor = use_signal(|| 0u64);
    let mut hash_total = use_signal(|| 0usize);
    let mut hash_has_more = use_signal(|| false);
    let mut hash_loading_more = use_signal(|| false);
    let mut show_large_key_warning = use_signal(|| false);
    let mut memory_usage = use_signal(|| None::<u64>);
    let mut ttl_input = use_signal(String::new);
    let mut toast_manager = use_context::<Signal<ToastManager>>();
    let mut ttl_processing = use_signal(|| false);
    let mut delete_key_confirm = use_signal(|| false);
    let mut delete_key_processing = use_signal(|| false);

    let mut bitmap_info = use_signal(|| None::<crate::redis::BitmapInfo>);
    let mut bitmap_editing_offset = use_signal(String::new);
    let mut bitmap_editing_value = use_signal(String::new);

    let mut stream_status_message = use_signal(String::new);
    let mut stream_status_error = use_signal(|| false);
    let mut stream_search = use_signal(String::new);
    let mut deleting_stream_entry = use_signal(|| None::<String>);
    let mut deleting_stream_entry_exiting = use_signal(|| false);

    let pool = connection_pool.clone();
    let pool_for_edit = connection_pool.clone();
    let pool_for_reload = connection_pool.clone();
    let pool_for_meta = connection_pool.clone();

    use_effect(move || {
        let key = selected_key.read().clone();

        hash_search.set(String::new());
        hash_status_message.set(String::new());
        hash_status_error.set(false);
        editing_hash_field.set(None);
        editing_hash_key.set(String::new());
        editing_hash_value.set(String::new());
        creating_hash_row.set(false);
        new_hash_key.set(String::new());
        new_hash_value.set(String::new());
        deleting_hash_field.set(None);
        hash_action.set(None);
        is_binary.set(false);
        serialization_data.set(None);
        binary_format.set(BinaryFormat::default());

        list_status_message.set(String::new());
        list_status_error.set(false);
        new_list_value.set(String::new());
        list_action.set(None);
        editing_list_index.set(None);
        editing_list_value.set(String::new());
        list_page.set(0);
        list_total.set(0);

        set_status_message.set(String::new());
        set_status_error.set(false);
        new_set_member.set(String::new());
        set_action.set(None);
        set_search.set(String::new());
        deleting_set_member.set(None);
        editing_set_member.set(None);
        editing_set_member_value.set(String::new());
        set_page.set(0);
        set_total.set(0);

        zset_status_message.set(String::new());
        zset_status_error.set(false);
        new_zset_member.set(String::new());
        new_zset_score.set(String::new());
        zset_action.set(None);
        zset_search.set(String::new());
        deleting_zset_member.set(None);
        editing_zset_member.set(None);
        editing_zset_score.set(String::new());
        zset_page.set(0);
        zset_total.set(0);

        show_large_key_warning.set(false);
        memory_usage.set(None);
        ttl_input.set(String::new());
        ttl_processing.set(false);
        delete_key_confirm.set(false);
        delete_key_processing.set(false);

        let pool = pool.clone();

        spawn(async move {
            tracing::info!("Loading key: {}", key);

            if let Err(error) = load_key_data(
                pool,
                key,
                key_info,
                string_value,
                hash_value,
                list_value,
                set_value,
                zset_value,
                stream_value,
                is_binary,
                binary_format,
                serialization_data,
                binary_bytes,
                bitmap_info,
                loading,
                hash_cursor,
                hash_total,
                hash_has_more,
                list_has_more,
                list_total,
                set_cursor,
                set_total,
                set_has_more,
                zset_cursor,
                zset_total,
                zset_has_more,
            )
            .await
            {
                tracing::error!("{error}");
                hash_status_message.set(error);
                hash_status_error.set(true);
            }
        });
    });

    use_effect(move || {
        let key = selected_key.read().clone();
        let format = binary_format();

        if key.is_empty() || !is_binary() {
            return;
        }

        let pool = pool_for_reload.clone();

        spawn(async move {
            match pool.get_string_bytes(&key).await {
                Ok(bytes) => string_value.set(format_bytes(&bytes, format)),
                Err(error) => tracing::error!("Failed to reload binary string bytes: {}", error),
            }
        });
    });

    use_effect(move || {
        if let Some(info) = key_info() {
            let key = info.name.clone();
            ttl_input.set(
                info.ttl
                    .map(|ttl| ttl.to_string())
                    .unwrap_or_else(|| "-1".to_string()),
            );

            let pool = pool_for_meta.clone();
            spawn(async move {
                match pool.memory_usage(&key).await {
                    Ok(usage) => memory_usage.set(usage),
                    Err(error) => {
                        tracing::error!("Failed to load memory usage: {}", error);
                        memory_usage.set(None);
                    }
                }
            });
        } else {
            ttl_input.set(String::new());
            memory_usage.set(None);
        }
    });

    let key_for_edit = selected_key;

    let info = key_info();
    let is_loading = loading();
    let str_val = string_value();
    let hash_val = hash_value();
    let list_val = list_value();
    let set_val = set_value();
    let zset_val = zset_value();
    let stream_val = stream_value();
    let display_key = selected_key.read().clone();

    let status_message = hash_status_message();
    let status_color = if hash_status_error() {
        COLOR_ERROR
    } else {
        COLOR_ACCENT
    };
    let active_hash_action = hash_action();
    let editing_field_name = editing_hash_field();
    let delete_target = deleting_hash_field();

    let sorted_entries = sorted_hash_entries(&hash_val);
    let search_value = hash_search();
    let normalized_search = search_value.trim().to_lowercase();
    let filtered_entries: Vec<(String, String)> = sorted_entries
        .into_iter()
        .filter(|(field, value)| {
            if normalized_search.is_empty() {
                true
            } else {
                field.to_lowercase().contains(&normalized_search)
                    || value.to_lowercase().contains(&normalized_search)
            }
        })
        .collect();

    let set_search_val = set_search();
    let normalized_set_search = set_search_val.trim().to_lowercase();
    let filtered_set_members: Vec<String> = set_val
        .iter()
        .filter(|member| {
            if normalized_set_search.is_empty() {
                true
            } else {
                member.to_lowercase().contains(&normalized_set_search)
            }
        })
        .cloned()
        .collect();

    let zset_search_val = zset_search();
    let normalized_zset_search = zset_search_val.trim().to_lowercase();
    let filtered_zset_members: Vec<(String, f64)> = zset_val
        .iter()
        .filter(|(member, _)| {
            if normalized_zset_search.is_empty() {
                true
            } else {
                member.to_lowercase().contains(&normalized_zset_search)
            }
        })
        .cloned()
        .collect();

    rsx! {
                div {
                    flex: "1",
                    height: "100%",
                    background: COLOR_BG,
                    display: "flex",
                    flex_direction: "column",

                    if !display_key.is_empty() {
                        div {
                            padding: "18px 20px 16px",
                            border_bottom: "1px solid {COLOR_BORDER}",
                            background: COLOR_BG,

                            div {
                            display: "flex",
                            justify_content: "space_between",
                            align_items: "flex_end",
                            gap: "16px",
                            flex_wrap: "wrap",

                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "6px",

                                span {
                                    color: COLOR_PRIMARY,
                                    font_size: "11px",
                                    font_weight: "800",
                                    text_transform: "uppercase",
                                    letter_spacing: "0.18em",

                                    "值编辑器"
                                }

                                if let Some(ref info) = info {
                                    div {
                                        display: "flex",
                                        align_items: "center",
                                        gap: "10px",
                                        flex_wrap: "wrap",

                                        span {
                                            color: COLOR_ACCENT,
                                            font_size: "15px",
                                            font_weight: "700",
                                            font_family: "Consolas, 'Courier New', monospace",

                                            "{display_key}"
                                        }

                                        button {
                                            width: "22px",
                                            height: "22px",
                                            background: "transparent",
                                            border: "1px solid {COLOR_BORDER}",
                                            border_radius: "4px",
                                            cursor: "pointer",
                                            display: "flex",
                                            align_items: "center",
                                            justify_content: "center",
                                            color: COLOR_TEXT_SECONDARY,
                                            title: "复制路径",
                                            onclick: {
                                                let key = display_key.clone();
                                                move |_| match copy_value_to_clipboard(&key) {
                                                    Ok(_) => {
        toast_manager.write().success("Key 路径已复制");
                                                    }
                                                    Err(error) => {
        toast_manager.write().error(&format!("复制失败：{error}"));
                                                    }
                                                }
                                            },

                                            IconCopy { size: Some(12) }
                                        }

                                        if delete_key_confirm() {
                                            div {
                                                display: "flex",
                                                align_items: "center",
                                                gap: "4px",

                                                button {
                                                    padding: "3px 8px",
                                                    background: COLOR_ERROR,
                                                    color: COLOR_TEXT_CONTRAST,
                                                    border: "none",
                                                    border_radius: "4px",
                                                    cursor: "pointer",
                                                    font_size: "11px",
                                                    disabled: delete_key_processing(),
                                                    onclick: {
                                                        let pool = connection_pool.clone();
                                                        let key = display_key.clone();
                                                        move |_| {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            spawn(async move {
                                                                delete_key_processing.set(true);

                                                                match pool.delete_key(&key).await {
                                                                    Ok(_) => {
                                                                        delete_key_confirm.set(false);
                                                                        toast_manager.write().success("Key 已删除");
                                                                        selected_key.set(String::new());
                                                                        on_refresh.call(());
                                                                    }
                                                                    Err(error) => {
                                                                        toast_manager.write().error(&format!("删除失败：{error}"));
                                                                    }
                                                                }

                                                                delete_key_processing.set(false);
                                                            });
                                                        }
                                                    },

                                                    if delete_key_processing() { "..." } else { "确认" }
                                                }

                                                button {
                                                    padding: "3px 8px",
                                                    background: COLOR_BG_TERTIARY,
                                                    color: COLOR_TEXT,
                                                    border: "1px solid {COLOR_BORDER}",
                                                    border_radius: "4px",
                                                    cursor: "pointer",
                                                    font_size: "11px",
                                                    onclick: move |_| delete_key_confirm.set(false),

                                                    "取消"
                                                }
                                            }
                                        } else {
                                            button {
                                                width: "22px",
                                                height: "22px",
                                                background: "transparent",
                                                border: "1px solid rgba(255, 180, 171, 0.30)",
                                                border_radius: "4px",
                                                cursor: "pointer",
                                                display: "flex",
                                                align_items: "center",
                                                justify_content: "center",
                                                color: COLOR_ERROR,
                                                title: "删除",
                                                onclick: move |_| delete_key_confirm.set(true),

                                                IconTrash { size: Some(12) }
                                            }
                                        }

                                        span {
                                            padding: "4px 8px",
                                            border_radius: "999px",
                                            background: "rgba(255, 180, 166, 0.10)",
                                            border: "1px solid rgba(255, 180, 166, 0.20)",
                                            color: COLOR_PRIMARY,
                                            font_size: "11px",
                                            font_weight: "700",
                                            text_transform: "uppercase",
                                            letter_spacing: "0.12em",

                                            "{info.key_type}"
                                        }

                                        {
                                            let memory_badge = format_memory_usage(memory_usage());

                                            rsx! {
                                                span {
                                                    padding: "4px 8px",
                                                    border_radius: "6px",
                                                    background: COLOR_BG_TERTIARY,
                                                    color: COLOR_TEXT_SECONDARY,
                                                    font_size: "11px",

                                                    "内存: {memory_badge}"
                                                }
                                            }
                                        }

                                        div {
                                            display: "flex",
                                            align_items: "center",
                                            gap: "4px",

                                            span {
                                                color: COLOR_TEXT_SECONDARY,
                                                font_size: "11px",

                                                "TTL:"
                                            }

                                            input {
                                                width: "72px",
                                                min_width: "72px",
                                                max_width: "72px",
                                                box_sizing: "border-box",
                                                flex_shrink: "0",
                                                padding: "2px 6px",
                                                background: COLOR_BG_TERTIARY,
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "4px",
                                                color: COLOR_TEXT,
                                                font_size: "11px",
                                                font_family: "Consolas, 'Courier New', monospace",
                                                text_align: "center",
                                                r#type: "text",
                                                value: "{ttl_input}",
                                                placeholder: "秒",
                                                oninput: move |event| ttl_input.set(event.value()),
                                            }

                                            button {
                                                padding: "2px 6px",
                                                background: COLOR_PRIMARY,
                                                color: COLOR_TEXT_CONTRAST,
                                                border: "none",
                                                border_radius: "4px",
                                                cursor: "pointer",
                                                font_size: "10px",
                                                disabled: ttl_processing(),
                                                onclick: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    move |_| {
                                                        let ttl_text = ttl_input().trim().to_string();
                                                        if ttl_text.is_empty() {
                                                            toast_manager.write().error("请输入 TTL");
                                                            return;
                                                        }

                                                        let ttl = match ttl_text.parse::<i64>() {
                                                            Ok(ttl) if ttl > 0 || ttl == -1 => ttl,
                                                            _ => {
                                                                toast_manager.write().error("TTL 必须大于 0 或 -1 表示永久");
                                                                return;
                                                            }
                                                        };

                                                        let pool = pool.clone();
                                                        let key = key.clone();
                                                        spawn(async move {
                                                            ttl_processing.set(true);

                                                            if ttl == -1 {
                                                                match pool.remove_ttl(&key).await {
                                                                    Ok(_) => {
                                                                        toast_manager.write().success("已设为永久");
                                                                        if let Err(error) = load_key_data(
                                                                            pool.clone(),
                                                                            key.clone(),
                                                                            key_info,
                                                                            string_value,
                                                                            hash_value,
                                                                            list_value,
                                                                            set_value,
                                                                            zset_value,
                                                                            stream_value,
                                                                            is_binary,
                                                                            binary_format,
                                                                            serialization_data,
                binary_bytes,
                                                                            bitmap_info,
                                                                            loading,
                                                                            hash_cursor,
                                                                            hash_total,
                                                                            hash_has_more,
                                                                            list_has_more,
                                                                            list_total,
                                                                            set_cursor,
                                                                            set_total,
                                                                            set_has_more,
                                                                            zset_cursor,
                                                                            zset_total,
                                                                            zset_has_more,
                                                                        ).await {
                                                                            tracing::error!("{error}");
                                                                        } else {
                                                                            on_refresh.call(());
                                                                        }
                                                                    }
                                                                    Err(error) => {
                                                                        toast_manager.write().error(&format!("设置失败：{error}"));
                                                                    }
                                                                }
                                                            } else {
                                                                match pool.set_ttl(&key, ttl).await {
                                                                    Ok(_) => {
                                                                        toast_manager.write().success("TTL 已更新");
                                                                        if let Err(error) = load_key_data(
                                                                            pool.clone(),
                                                                            key.clone(),
                                                                            key_info,
                                                                            string_value,
                                                                            hash_value,
                                                                            list_value,
                                                                            set_value,
                                                                            zset_value,
                                                                            stream_value,
                                                                            is_binary,
                                                                            binary_format,
                                                                            serialization_data,
                binary_bytes,
                                                                            bitmap_info,
                                                                            loading,
                                                                            hash_cursor,
                                                                            hash_total,
                                                                            hash_has_more,
                                                                            list_has_more,
                                                                            list_total,
                                                                            set_cursor,
                                                                            set_total,
                                                                            set_has_more,
                                                                            zset_cursor,
                                                                            zset_total,
                                                                            zset_has_more,
                                                                        ).await {
                                                                            tracing::error!("{error}");
                                                                        } else {
                                                                            on_refresh.call(());
                                                                        }
                                                                    }
                                                                    Err(error) => {
                                                                        toast_manager.write().error(&format!("TTL 更新失败：{error}"));
                                                                    }
                                                                }
                                                            }

                                                            ttl_processing.set(false);
                                                        });
                                                    }
                                                },

                                                if ttl_processing() { "..." } else { "✓" }
                                            }
                                        }
                                    }
                                } else {
                                    span {
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "13px",

                                        "选择一个 Key 以查看和编辑详情"
                                    }
                                }
                            }
                        }
                    }
                    }

                    div {
                        flex: "1",
                        overflow: "hidden",
                        padding: "18px",

                        if is_loading {
                            div {
                                height: "100%",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                color: COLOR_TEXT_SECONDARY,
                                text_align: "center",
                                border: "1px solid {COLOR_BORDER}",
                                border_radius: "12px",
                                background: COLOR_BG_SECONDARY,

                                "正在加载 Key 内容..."
                            }
                        } else if display_key.is_empty() {
                            ServerInfoPanel {
                                connection_pool: connection_pool.clone(),
                                connection_version: connection_version,
                                auto_refresh_interval: 0,
                            }
                        } else if let Some(info) = info.clone() {
                            {
                                let value_metric = value_metric_label(
                                    &info.key_type,
                                    &str_val,
                                    &hash_val,
                                    &list_val,
                                    &set_val,
                                    &zset_val,
                                    &stream_val,
                                );
                                let ttl_badge = format_ttl_label(info.ttl);
                                let memory_badge = format_memory_usage(memory_usage());

                                rsx! {
                                div {
                                    height: "100%",
                                    display: "flex",
                                    flex_wrap: "wrap",
                                    gap: "18px",
                                    align_items: "stretch",

                                    div {
                                        flex: "1 1 640px",
                                        min_width: "320px",
                                        display: "flex",
                                        flex_direction: "column",
                                        gap: "14px",
        overflow: "hidden",

                                        div {
                                            flex: "1",
                                            min_height: "0",
                                            overflow: "hidden",
                                            padding: "16px",
                                            border: "1px solid {COLOR_BORDER}",
                                            border_radius: "12px",
                                            background: COLOR_BG_SECONDARY,

        match info.key_type {
                                        KeyType::String => {
                                            let is_json = !is_binary() && is_json_content(&str_val);
                                            let serialization_info = serialization_data();
                                            let detected_format = serialization_info.as_ref().map(|(f, _)| *f);
                                            let is_serialized = serialization_info.is_some();

                                            rsx! {
                                                div {
                                                    if is_binary() {
                                                        div {
                                                            display: "flex",
                                                            gap: "8px",
                                                            align_items: "center",
                                                            margin_bottom: "12px",
                                                            flex_wrap: "wrap",

                                                            if is_serialized {
                                                                span {
                                                                    color: COLOR_SUCCESS,
                                                                    font_size: "12px",

                                                                    match detected_format {
                                                                        Some(SerializationFormat::Java) => "Java 序列化对象",
                                                                        Some(SerializationFormat::Php) => "PHP 序列化数据",
                                                                        Some(SerializationFormat::MsgPack) => "MessagePack 数据",
                                                                        Some(SerializationFormat::Pickle) => "Python Pickle 数据",
                                                                        Some(SerializationFormat::Kryo) => "Kryo 序列化数据",
                                                                        Some(SerializationFormat::Fst) => "FST 序列化数据",
                                                                        _ => "序列化数据",
                                                                    }
                                                                }
                                                            } else {
                                                                span {
                                                                    color: COLOR_WARNING,
                                                                    font_size: "12px",

                                                                    "二进制数据"
                                                                }
                                                            }

                                                            button {
                                                                padding: "4px 8px",
                                                                background: if binary_format() == BinaryFormat::Hex { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                color: if binary_format() == BinaryFormat::Hex { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                border: "none",
                                                                border_radius: "4px",
                                                                cursor: "pointer",
                                                                font_size: "12px",
                                                                onclick: move |_| binary_format.set(BinaryFormat::Hex),

                                                                "Hex"
                                                            }

                                                            button {
                                                                padding: "4px 8px",
                                                                background: if binary_format() == BinaryFormat::Base64 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                color: if binary_format() == BinaryFormat::Base64 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                border: "none",
                                                                border_radius: "4px",
                                                                cursor: "pointer",
                                                                font_size: "12px",
                                                                onclick: move |_| binary_format.set(BinaryFormat::Base64),

                                                                "Base64"
                                                            }

                                                            {
                                                                let bytes = binary_bytes();
                                                                let is_image = detect_image_format(&bytes).is_some();
                                                                if is_image {
                                                                    rsx! {
                                                                        button {
                                                                            padding: "4px 8px",
                                                                            background: if binary_format() == BinaryFormat::Image { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                            color: if binary_format() == BinaryFormat::Image { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                            border: "none",
                                                                            border_radius: "4px",
                                                                            cursor: "pointer",
                                                                            font_size: "12px",
                                                                            onclick: move |_| binary_format.set(BinaryFormat::Image),

                                                                            "图片"
                                                                        }
                                                                    }
                                                                } else {
                                                                    rsx! { div {} }
                                                                }
                                                            }

                                                            if detected_format == Some(SerializationFormat::Java) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::JavaSerialized { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::JavaSerialized { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::JavaSerialized),

                                                                    "Java解析"
                                                                }
                                                            }

                                                            if detected_format == Some(SerializationFormat::Php) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Php { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Php { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Php),

                                                                    "PHP解析"
                                                                }
                                                            }

                                                            if detected_format == Some(SerializationFormat::MsgPack) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::MsgPack { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::MsgPack { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::MsgPack),

                                                                    "MsgPack"
                                                                }
                                                            }

                                                            if detected_format == Some(SerializationFormat::Pickle) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Pickle { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Pickle { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Pickle),

                                                                    "Pickle"
                                                                }
                                                            }

                                                            if matches!(detected_format, Some(SerializationFormat::Kryo) | Some(SerializationFormat::Fst)) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Kryo { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Kryo { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Kryo),

                                                                    "Kryo"
                                                                }
                                                            }

                                                            if detected_format == Some(SerializationFormat::Bson) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Bson { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Bson { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Bson),

                                                                    "BSON"
                                                                }
                                                            }

                                                            if detected_format == Some(SerializationFormat::Cbor) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Cbor { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Cbor { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Cbor),

                                                                    "CBOR"
                                                                }
                                                            }

                                                            if !is_serialized {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Bitmap { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Bitmap { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: {
                                                                        let pool = connection_pool.clone();
                                                                        let key = display_key.clone();
                                                                        move |_| {
                                                                            let pool = pool.clone();
                                                                            let key = key.clone();
                                                                            spawn(async move {
                                                                                match pool.get_bitmap_info(&key).await {
                                                                                    Ok(info) => {
                                                                                        bitmap_info.set(Some(info));
                                                                                        binary_format.set(BinaryFormat::Bitmap);
                                                                                    }
                                                                                    Err(e) => {
                                                                                        toast_manager.write().error(&format!("加载 Bitmap 失败: {}", e));
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    },

"Bitmap"
                                                                 }
                                                            }

                                                            if detected_format == Some(SerializationFormat::Protobuf) || is_protobuf_data(string_value().as_bytes()) {
                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Protobuf { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Protobuf { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Protobuf),

                                                                    "Protobuf"
                                                                }
                                                            }

                                                            button {
                                                                padding: "4px 10px",
                                                                background: "rgba(47, 133, 90, 0.16)",
                                                                color: COLOR_SUCCESS,
                                                                border: "1px solid rgba(104, 211, 145, 0.28)",
                                                                border_radius: "6px",
                                                                cursor: "pointer",
                                                                font_size: "12px",
                                                                display: "flex",
                                                                align_items: "center",
                                                                gap: "4px",
                                                                title: "复制",
                                                                onclick: move |_| {
                                                                    let current_format = binary_format();
                                                                    let serial_info = serialization_data();
                                                                    let current_str = string_value();
                                                                    let copy_text = match current_format {
                                                                        BinaryFormat::JavaSerialized
                                                                        | BinaryFormat::Php
                                                                        | BinaryFormat::MsgPack
                                                                        | BinaryFormat::Pickle
                                                                        | BinaryFormat::Kryo
                                                                        | BinaryFormat::Protobuf
                                                                        | BinaryFormat::Bson
                                                                        | BinaryFormat::Cbor => {
                                                                            if let Some((fmt, data)) = serial_info.as_ref() {
                                                                                parse_to_json(data, *fmt).unwrap_or(current_str)
                                                                            } else {
                                                                                current_str
                                                                            }
                                                                        }
                                                                        _ => current_str,
                                                                    };
                                                                    match copy_text_to_clipboard(&copy_text) {
                                                                        Ok(_) => {
                                                                            toast_manager.write().success("复制成功");
                                                                        }
                                                                        Err(e) => {
                                                                            toast_manager.write().error(&format!("复制失败：{}", e));
                                                                        }
                                                                    }
                                                                },

                                                                IconCopy { size: Some(14) }
                                                                "复制"
                                                            }
                                                        }
                                                    }

                                                    if is_binary() {
                                                        match binary_format() {
                                                            BinaryFormat::JavaSerialized => {
                                                                if let Some((SerializationFormat::Java, ref data)) = serialization_info {
                                                                    rsx! {
                                                                        JavaSerializedViewer {
                                                                            data: data.clone(),
                                                                        }
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div {
                                                                            padding: "16px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            border_radius: "8px",
                                                                            color: COLOR_TEXT_SECONDARY,

                                                                            "解析失败"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            BinaryFormat::Php => {
                                                                if let Some((SerializationFormat::Php, ref data)) = serialization_info {
                                                                    match parse_to_json(data, SerializationFormat::Php) {
                                                                        Ok(json_str) => rsx! {
                                                                            JsonViewer {
                                                                                value: json_str,
                                                                                editable: false,
                                                                                on_change: move |_| {},
                                                                            }
                                                                        },
                                                                        Err(e) => rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_ERROR_BG,
                                                                                border_radius: "8px",
                                                                                color: COLOR_ERROR,

                                                                                "PHP 解析错误: {e}"
                                                                            }
                                                                        },
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div {
                                                                            padding: "16px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            border_radius: "8px",
                                                                            color: COLOR_TEXT_SECONDARY,

                                                                            "解析失败"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            BinaryFormat::MsgPack => {
                                                                if let Some((SerializationFormat::MsgPack, ref data)) = serialization_info {
                                                                    match parse_to_json(data, SerializationFormat::MsgPack) {
                                                                        Ok(json_str) => rsx! {
                                                                            JsonViewer {
                                                                                value: json_str,
                                                                                editable: false,
                                                                                on_change: move |_| {},
                                                                            }
                                                                        },
                                                                        Err(e) => rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_ERROR_BG,
                                                                                border_radius: "8px",
                                                                                color: COLOR_ERROR,

                                                                                "MsgPack 解析错误: {e}"
                                                                            }
                                                                        },
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div {
                                                                            padding: "16px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            border_radius: "8px",
                                                                            color: COLOR_TEXT_SECONDARY,

                                                                            "解析失败"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            BinaryFormat::Pickle => {
                                                                if let Some((SerializationFormat::Pickle, ref data)) = serialization_info {
                                                                    match parse_to_json(data, SerializationFormat::Pickle) {
                                                                        Ok(json_str) => rsx! {
                                                                            JsonViewer {
                                                                                value: json_str,
                                                                                editable: false,
                                                                                on_change: move |_| {},
                                                                            }
                                                                        },
                                                                        Err(e) => rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_ERROR_BG,
                                                                                border_radius: "8px",
                                                                                color: COLOR_ERROR,

                                                                                "Pickle 解析错误: {e}"
                                                                            }
                                                                        },
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div {
                                                                            padding: "16px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            border_radius: "8px",
                                                                            color: COLOR_TEXT_SECONDARY,

                                                                            "解析失败"
                                                                        }
                                                                    }
                                                                }
                                                            }
BinaryFormat::Kryo => {
                                                                 if let Some((ref format, ref data)) = serialization_info {
                                                                     if matches!(format, SerializationFormat::Kryo | SerializationFormat::Fst) {
                                                                         match parse_to_json(data, *format) {
                                                                             Ok(json_str) => rsx! {
                                                                                 JsonViewer {
                                                                                     value: json_str,
                                                                                     editable: false,
                                                                                     on_change: move |_| {},
                                                                                 }
                                                                             },
                                                                             Err(e) => rsx! {
                                                                                 div {
                                                                                     padding: "16px",
                                                                                     background: COLOR_ERROR_BG,
                                                                                     border_radius: "8px",
                                                                                     color: COLOR_ERROR,

                                                                                     "Kryo/FST 解析错误: {e}"
                                                                                 }
                                                                             },
                                                                         }
                                                                     } else {
                                                                         rsx! {
                                                                             div {
                                                                                 padding: "16px",
                                                                                 background: COLOR_BG_TERTIARY,
                                                                                 border_radius: "8px",
                                                                                 color: COLOR_TEXT_SECONDARY,

                                                                                 "非 Kryo/FST 数据"
                                                                             }
                                                                         }
                                                                     }
                                                                 } else {
                                                                     rsx! {
                                                                         div {
                                                                             padding: "16px",
                                                                             background: COLOR_BG_TERTIARY,
                                                                             border_radius: "8px",
                                                                             color: COLOR_TEXT_SECONDARY,

                                                                             "解析失败"
                                                                         }
                                                                     }
                                                                 }
                                                             }
                                                             BinaryFormat::Bson => {
                                                                 if let Some((SerializationFormat::Bson, ref data)) = serialization_info {
                                                                     match parse_to_json(data, SerializationFormat::Bson) {
                                                                         Ok(json_str) => rsx! {
                                                                             JsonViewer {
                                                                                 value: json_str,
                                                                                 editable: false,
                                                                                 on_change: move |_| {},
                                                                             }
                                                                         },
                                                                         Err(e) => rsx! {
                                                                             div {
                                                                                 padding: "16px",
                                                                                 background: COLOR_ERROR_BG,
                                                                                 border_radius: "8px",
                                                                                 color: COLOR_ERROR,

                                                                                 "BSON 解析错误: {e}"
                                                                             }
                                                                         },
                                                                     }
                                                                 } else {
                                                                     rsx! {
                                                                         div {
                                                                             padding: "16px",
                                                                             background: COLOR_BG_TERTIARY,
                                                                             border_radius: "8px",
                                                                             color: COLOR_TEXT_SECONDARY,

                                                                             "解析失败"
                                                                         }
                                                                     }
                                                                 }
                                                             }
                                                             BinaryFormat::Cbor => {
                                                                 if let Some((SerializationFormat::Cbor, ref data)) = serialization_info {
                                                                     match parse_to_json(data, SerializationFormat::Cbor) {
                                                                         Ok(json_str) => rsx! {
                                                                             JsonViewer {
                                                                                 value: json_str,
                                                                                 editable: false,
                                                                                 on_change: move |_| {},
                                                                             }
                                                                         },
                                                                         Err(e) => rsx! {
                                                                             div {
                                                                                 padding: "16px",
                                                                                 background: COLOR_ERROR_BG,
                                                                                 border_radius: "8px",
                                                                                 color: COLOR_ERROR,

                                                                                 "CBOR 解析错误: {e}"
                                                                             }
                                                                         },
                                                                     }
                                                                 } else {
                                                                     rsx! {
                                                                         div {
                                                                             padding: "16px",
                                                                             background: COLOR_BG_TERTIARY,
                                                                             border_radius: "8px",
                                                                             color: COLOR_TEXT_SECONDARY,

                                                                             "解析失败"
                                                                         }
                                                                     }
                                                                 }
                                                             }
                                                             BinaryFormat::Image => {
                                                                let bytes = binary_bytes();
                                                                tracing::info!("Image preview: {} bytes, first 10: {:02x?}", bytes.len(), &bytes[..10.min(bytes.len())]);
                                                                if let Some(format) = detect_image_format(&bytes) {
                                                                    use base64::{engine::general_purpose, Engine as _};
                                                                    let base64_data = general_purpose::STANDARD.encode(&bytes);
                                                                    let mime_type = match format {
                                                                        "PNG" => "image/png",
                                                                        "JPEG" => "image/jpeg",
                                                                        "GIF" => "image/gif",
                                                                        "WEBP" => "image/webp",
                                                                        "BMP" => "image/bmp",
                                                                        _ => "application/octet-stream",
                                                                    };
                                                                    let data_uri = format!("data:{};base64,{}", mime_type, base64_data);
                                                                    
                                                                    let temp_dir = std::env::temp_dir();
                                                                    let file_name = format!("redis_image_{}.{}", uuid::Uuid::new_v4(), format.to_lowercase());
                                                                    let file_path = temp_dir.join(&file_name);
                                                                    let file_path_clone = file_path.clone();
                                                                    let bytes_clone = bytes.clone();
                                                                    let file_size_formatted = format_memory_usage(Some(bytes.len() as u64));
                                                                    let format_str = format.to_string();
                                                                    let data_uri_for_preview = data_uri.clone();
                                                                    let format_for_preview = format_str.clone();
                                                                    let size_for_preview = file_size_formatted.clone();

                                                                    rsx! {
                                                                        div {
                                                                            display: "flex",
                                                                            flex_direction: "column",
                                                                            align_items: "center",
                                                                            gap: "12px",

                                                                            div {
                                                                                padding: "12px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,
                                                                                font_size: "13px",

                                                                                "{format} 图片 - {file_size_formatted}"
                                                                            }

                                                                            div {
                                                                                max_width: "100%",
                                                                                max_height: "500px",
                                                                                overflow: "auto",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                padding: "8px",

                                                                                img {
                                                                                    src: "{data_uri}",
                                                                                    max_width: "100%",
                                                                                    max_height: "500px",
                                                                                    object_fit: "contain",
                                                                                    border_radius: "4px",
                                                                                    cursor: "pointer",
                                                                                    transition: "transform 0.2s, box-shadow 0.2s",

                                                                                    onclick: move |_| {
                                                                                        *PREVIEW_IMAGE.write() = Some(PreviewImageData {
                                                                                            data_uri: data_uri_for_preview.clone(),
                                                                                            format: format_for_preview.clone(),
                                                                                            size: size_for_preview.clone(),
                                                                                        });
                                                                                    },
                                                                                }
                                                                            }

                                                                            button {
                                                                                padding: "8px 16px",
                                                                                background: COLOR_PRIMARY,
                                                                                color: COLOR_TEXT_CONTRAST,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                font_size: "13px",

                                                                                onclick: move |_| {
                                                                                    let _ = std::fs::write(&file_path_clone, &bytes_clone);
                                                                                    let _ = open::that(&file_path_clone);
                                                                                },

                                                                                "用系统图片查看器打开"
                                                                            }
                                                                        }
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div {
                                                                            padding: "16px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            border_radius: "8px",
                                                                            color: COLOR_TEXT_SECONDARY,

                                                                            "非图片数据"
                                                                        }
                                                                    }
                                                                }
                                                            }
BinaryFormat::Protobuf => {
                                                                if let Some((SerializationFormat::Protobuf, ref data)) = serialization_info {
                                                                    rsx! {
                                                                        ProtobufViewer {
                                                                            data: data.clone(),
                                                                        }
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div {
                                                                            padding: "16px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            border_radius: "8px",
                                                                            color: COLOR_TEXT_SECONDARY,

                                                                            "非 Protobuf 数据"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            BinaryFormat::Bitmap => {
                                                                if let Some(ref info) = bitmap_info() {
                                                                    rsx! {
                                                                        BitmapViewer {
                                                                            info: info.clone(),
                                                                            pool: connection_pool.clone(),
                                                                            redis_key: display_key.clone(),
                                                                            on_update: {
                                                                                let pool = connection_pool.clone();
                                                                                let key = display_key.clone();
                                                                                move || {
                                                                                    let pool = pool.clone();
                                                                                    let key = key.clone();
                                                                                    spawn(async move {
                                                                                        if let Ok(new_info) = pool.get_bitmap_info(&key).await {
                                                                                            bitmap_info.set(Some(new_info));
                                                                                        }
                                                                                    });
                                                                                }
                                                                            },
                                                                        }
                                                                    }
                                                                } else {
                                                                    rsx! {
                                                                        div {
                                                                            padding: "16px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            border_radius: "8px",
                                                                            color: COLOR_TEXT_SECONDARY,

                                                                            "点击 \"Bitmap\" 按钮加载可视化数据"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            _ => {
                                                                rsx! {
                                                                    EditableField {
                                                                        label: String::new(),
                                                                        value: str_val.clone(),
                                                                        multiline: true,
                                                                        editable: false,
                                                                        on_change: move |_| {},
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else if is_json {
                                                    JsonViewer {
                                                        value: str_val.clone(),
                                                        editable: true,
                                                        on_change: {
                                                            let pool = pool_for_edit.clone();
                                                            let key_sig = key_for_edit.clone();
                                                            move |new_val: String| {
                                                                let pool = pool.clone();
                                                                let key = key_sig.read().clone();
                                                                let val = new_val.clone();
                                                                spawn(async move {
                                                                    saving.set(true);
                                                                    if pool.set_string_value(&key, &val).await.is_ok() {
                                                                        string_value.set(val);
                                                                        on_refresh.call(());
                                                                    }
                                                                    saving.set(false);
                                                                });
                                                            }
                                                        },
                                                    }
                                                } else {
                                                    EditableField {
                                                        label: "Value".to_string(),
                                                        value: str_val.clone(),
                                                        editable: !is_binary(),
                                                        multiline: true,
                                                        on_change: {
                                                            let pool = pool_for_edit.clone();
                                                            let key_sig = key_for_edit.clone();
                                                            move |new_val: String| {
                                                                let pool = pool.clone();
                                                                let key = key_sig.read().clone();
                                                                let val = new_val.clone();
                                                                spawn(async move {
                                                                    saving.set(true);
                                                                    if pool.set_string_value(&key, &val).await.is_ok() {
                                                                        string_value.set(val);
                                                                        on_refresh.call(());
                                                                    }
                                                                    saving.set(false);
                                                                });
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    KeyType::Hash => {
                                        rsx! {
                                            div {
                                                display: "flex",
                                                flex_direction: "column",
                                                height: "100%",
                                                min_height: "0",

                                                div {
                                                    display: "flex",
                                                    justify_content: "space_between",
                                                    align_items: "center",
                                                    gap: "12px",
                                                    flex_wrap: "wrap",
                                                    margin_bottom: "12px",

                                                    div {
                                                        display: "flex",
                                                        gap: "8px",
                                                        align_items: "center",
                                                        flex_wrap: "wrap",

                                                        input {
                                                            width: "280px",
                                                            max_width: "100%",
                                                            padding: "8px 10px",
                                                            background: COLOR_BG_TERTIARY,
                                                            border: "1px solid {COLOR_BORDER}",
                                                            border_radius: "6px",
                                                            color: COLOR_TEXT,
                                                            value: "{search_value}",
                                                            placeholder: "搜索 key 或 value",
                                                            oninput: {
                                                                let pool = connection_pool.clone();
                                                                let key = display_key.clone();
                                                                move |event| {
                                                                    let value = event.value();
                                                                    let was_empty = hash_search().is_empty();
                                                                    hash_search.set(value.clone());

                                                                    if value.is_empty() && !was_empty {
                                                                        let pool = pool.clone();
                                                                        let key = key.clone();
                                                                        spawn(async move {
                                                                            if let Err(e) = load_key_data(
                                                                                pool,
                                                                                key,
                                                                                key_info,
                                                                                string_value,
                                                                                hash_value,
                                                                                list_value,
                                                                                set_value,
                                                                                zset_value,
                                                                                stream_value,
                                                                                is_binary,
                                                                                binary_format,
                                                                                serialization_data,
                    binary_bytes,
                                                                                bitmap_info,
                                                                                loading,
                                                                                hash_cursor,
                                                                                hash_total,
                                                                                hash_has_more,
                                                                                list_has_more,
                                                                                list_total,
                                                                                set_cursor,
                                                                                set_total,
                                                                                set_has_more,
                                                                                zset_cursor,
                                                                                zset_total,
                                                                                zset_has_more,
                                                                            ).await {
                                                                                tracing::error!("重新加载 hash 数据失败: {}", e);
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            },
                                                        }

                                                        if hash_total() > PAGE_SIZE {
                                                            button {
                                                                padding: "6px 10px",
                                                                background: if hash_search().len() >= 2 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                color: if hash_search().len() >= 2 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT_SECONDARY },
                                                                border: "1px solid {COLOR_BORDER}",
                                                                border_radius: "6px",
                                                                cursor: if hash_search().len() >= 2 { "pointer" } else { "not-allowed" },
                                                                font_size: "12px",
                                                                disabled: hash_search().len() < 2 || hash_loading_more(),
                                                                onclick: {
                                                                    let pool = connection_pool.clone();
                                                                    let key = display_key.clone();
                                                                    move |_| {
                                                                        let pool = pool.clone();
                                                                        let key = key.clone();
                                                                        let pattern = hash_search();
                                                                        spawn(async move {
                                                                            search_hash_server(
                                                                                pool,
                                                                                key,
                                                                                pattern,
                                                                                hash_value,
                                                                                hash_cursor,
                                                                                hash_has_more,
                                                                                hash_loading_more,
                                                                            ).await;
                                                                        });
                                                                    }
                                                                },

                                                                if hash_loading_more() { "搜索中..." } else { "服务端搜索" }
                                                            }
                                                        }

                                                        button {
                                                            padding: "8px 12px",
                                                            background: "#0e639c",
                                                            color: COLOR_TEXT_CONTRAST,
                                                            border: "none",
                                                            border_radius: "6px",
                                                            cursor: "pointer",
                                                            disabled: active_hash_action.is_some(),
                                                            onclick: move |_| {
                                                                creating_hash_row.set(true);
                                                                editing_hash_field.set(None);
                                                                editing_hash_key.set(String::new());
                                                                editing_hash_value.set(String::new());
                                                                new_hash_key.set(String::new());
                                                                new_hash_value.set(String::new());
                                                                hash_status_message.set(String::new());
                                                                hash_status_error.set(false);
                                                            },

                                                            "+ 新增行"
                                                        }

                                                        div {
                                                            color: COLOR_TEXT_SECONDARY,
                                                            font_size: "13px",

                                                            "Hash Fields ({hash_val.len()}/{hash_total()})"
                                                        }
                                                    }

                                                    button {
                                                        margin_left: "auto",
                                                        flex_shrink: "0",
                                                        padding: "6px 10px",
                                                        background: "rgba(47, 133, 90, 0.16)",
                                                        color: COLOR_SUCCESS,
                                                        border: "1px solid rgba(104, 211, 145, 0.28)",
                                                        border_radius: "6px",
                                                        cursor: "pointer",
                                                        display: "flex",
                                                        align_items: "center",
                                                        gap: "4px",
                                                        title: "复制",
                                                        onclick: {
                                                            let hash = hash_val.clone();
                                                            move |_| {
                                                                let json = serde_json::to_string_pretty(&hash).unwrap_or_default();
                                                                match copy_value_to_clipboard(&json) {
                                                                    Ok(_) => {
                                                                        toast_manager.write().success("复制成功");
                                                                    }
                                                                    Err(error) => {
                                                                        toast_manager.write().error(&format!("复制失败：{error}"));
                                                                    }
                                                                }
                                                            }
                                                        },

                                                        IconCopy { size: Some(14) }
                                                        "复制"
                                                    }
                                                }
                                            }

                                            div {
                                                flex: "1",
                                                min_height: "0",
                                                overflow_x: "auto",
                                                overflow_y: "auto",
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "8px",
                                                background: COLOR_BG_SECONDARY,
                                                onscroll: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    move |e| {
                                                        let scroll_top = e.data().scroll_top() as i32;
                                                        let scroll_height = e.data().scroll_height() as i32;
                                                        let client_height = e.data().client_height() as i32;
                                                        
                                                        if hash_has_more() && !hash_loading_more() && scroll_height - scroll_top - client_height < 200 {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let cursor = hash_cursor();
                                                            let total = hash_total();
                                                            spawn(async move {
                                                                load_more_hash(
                                                                    pool,
                                                                    key,
                                                                    hash_value,
                                                                    cursor,
                                                                    hash_cursor,
                                                                    hash_has_more,
                                                                    hash_loading_more,
                                                                    hash_total,
                                                                ).await;
                                                            });
                                                        }
                                                    }
                                                },

                                                table {
                                                    width: "100%",
                                                    border_collapse: "collapse",

                                                    thead {
                                                        tr {
                                                            background: COLOR_BG_TERTIARY,
                                                            border_bottom: "1px solid {COLOR_BORDER}",

                                                            th {
                                                                width: "72px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "ID"
                                                            }

                                                            th {
                                                                width: "32%",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "key"
                                                            }

                                                            th {
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "value"
                                                            }

                                                            th {
                                                                width: "156px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "action"
                                                            }
                                                        }
                                                    }

                                                    tbody {
                                                        if creating_hash_row() {
                                                            tr {
                                                                background: ROW_CREATE_BG,
                                                                border_bottom: "1px solid {COLOR_BORDER}",

                                                                td {
                                                                    padding: "12px",
                                                                    color: COLOR_ACCENT,
                                                                    vertical_align: "top",

                                                                    "+"
                                                                }

                                                                td {
                                                                    padding: "12px",
                                                                    vertical_align: "top",

                                                                    input {
                                                                        width: "100%",
                                                                        padding: "8px 10px",
                                                                        background: COLOR_BG,
                                                                        border: "1px solid {COLOR_BORDER}",
                                                                        border_radius: "6px",
                                                                        color: COLOR_TEXT,
                                                                        value: "{new_hash_key}",
                                                                        placeholder: "输入 field key",
                                                                        oninput: move |event| new_hash_key.set(event.value()),
                                                                    }
                                                                }

                                                                td {
                                                                    padding: "12px",
                                                                    vertical_align: "top",

                                                                    textarea {
                                                                        width: "100%",
                                                                        min_height: "92px",
                                                                        padding: "8px 10px",
                                                                        background: COLOR_BG,
                                                                        border: "1px solid {COLOR_BORDER}",
                                                                        border_radius: "6px",
                                                                        color: COLOR_TEXT,
                                                                        font_family: "Consolas, 'Courier New', monospace",
                                                                        resize: "vertical",
                                                                        value: "{new_hash_value}",
                                                                        placeholder: "输入 field value",
                                                                        oninput: move |event| new_hash_value.set(event.value()),
                                                                    }
                                                                }

                                                                td {
                                                                    padding: "12px",
                                                                    vertical_align: "top",

                                                                    div {
                                                                        display: "flex",
                                                                        gap: "8px",
                                                                        align_items: "center",
                                                                        flex_wrap: "wrap",

                                                                        button {
                                                                            padding: "6px 10px",
                                                                            background: "#38a169",
                                                                            color: COLOR_TEXT_CONTRAST,
                                                                            border: "none",
                                                                            border_radius: "6px",
                                                                            cursor: "pointer",
                                                                            disabled: active_hash_action.is_some(),
                                                                            onclick: {
                                                                                let pool = connection_pool.clone();
                                                                                let key = display_key.clone();
                                                                                let existing_hash = hash_val.clone();
                                                                                move |_| {
                                                                                    let pool = pool.clone();
                                                                                    let key = key.clone();
                                                                                    let existing_hash = existing_hash.clone();
                                                                                    let field = new_hash_key().trim().to_string();
                                                                                    let value = new_hash_value();
                                                                                    spawn(async move {
                                                                                        if field.is_empty() {
                                                                                            hash_status_message.set("新增失败：key 不能为空".to_string());
                                                                                            hash_status_error.set(true);
                                                                                            return;
                                                                                        }

                                                                                        if existing_hash.contains_key(&field) {
                                                                                            hash_status_message.set("新增失败：key 已存在".to_string());
                                                                                            hash_status_error.set(true);
                                                                                            return;
                                                                                        }

                                                                                        hash_action.set(Some("create".to_string()));
                                                                                        hash_status_message.set(String::new());
                                                                                        hash_status_error.set(false);

                                                                                        match pool.hash_set_field(&key, &field, &value).await {
                                                                                            Ok(_) => {
                                                                                                creating_hash_row.set(false);
                                                                                                new_hash_key.set(String::new());
                                                                                                new_hash_value.set(String::new());
                                                                                                hash_status_message.set("新增成功".to_string());
                                                                                                hash_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                     pool.clone(),
                                                                                                     key.clone(),
                                                                                                     key_info,
                                                                                                     string_value,
                                                                                                     hash_value,
                                                                                                     list_value,
                                                                                                     set_value,
                                                                                                     zset_value,
                                                                                                     stream_value,
                                                                                                     is_binary,
                                                                                                     binary_format,
                                                                                                     serialization_data,
                binary_bytes,
                                                                                                     bitmap_info,
                                                                                                     loading,
                                                                                                     hash_cursor,
                                                                                                     hash_total,
                                                                                                     hash_has_more,
                                                                                                     list_has_more,
                                                                                                     list_total,
                                                                                                     set_cursor,
                                                                                                     set_total,
                                                                                                     set_has_more,
                                                                                                     zset_cursor,
                                                                                                     zset_total,
                                                                                                     zset_has_more,
                                                                                                 )
                                                                                                 .await
                                                                                                 {
                                                                                                    tracing::error!("{error}");
                                                                                                    hash_status_message.set(error);
                                                                                                    hash_status_error.set(true);
                                                                                                } else {
                                                                                                    on_refresh.call(());
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                tracing::error!("Failed to create hash field: {}", error);
                                                                                                hash_status_message.set(format!("新增失败：{error}"));
                                                                                                hash_status_error.set(true);
                                                                                            }
                                                                                        }

                                                                                        hash_action.set(None);
                                                                                    });
                                                                                }
                                                                            },

                                                                            if active_hash_action.as_deref() == Some("create") { "保存中..." } else { "保存" }
                                                                        }

                                                                        button {
                                                                            padding: "6px 10px",
                                                                            background: COLOR_BG_TERTIARY,
                                                                            color: COLOR_TEXT,
                                                                            border: "none",
                                                                            border_radius: "6px",
                                                                            cursor: "pointer",
                                                                            disabled: active_hash_action.is_some(),
                                                                            onclick: move |_| {
                                                                                creating_hash_row.set(false);
                                                                                new_hash_key.set(String::new());
                                                                                new_hash_value.set(String::new());
                                                                            },

                                                                            "取消"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        for (index, (field, value)) in filtered_entries.iter().enumerate() {
                                                            if editing_field_name.as_deref() == Some(field.as_str()) {
                                                                tr {
                                                                    key: "edit-{field}",
                                                                    background: ROW_EDIT_BG,
                                                                    border_bottom: "1px solid {COLOR_BORDER}",

                                                                    td {
                                                                        padding: "12px",
                                                                        color: COLOR_ACCENT,
                                                                        vertical_align: "top",

                                                                        "{index + 1}"
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        vertical_align: "top",

                                                                        input {
                                                                            width: "100%",
                                                                            padding: "8px 10px",
                                                                            background: COLOR_BG,
                                                                            border: "1px solid {COLOR_BORDER}",
                                                                            border_radius: "6px",
                                                                            color: COLOR_TEXT,
                                                                            value: "{editing_hash_key}",
                                                                            oninput: move |event| editing_hash_key.set(event.value()),
                                                                        }
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        vertical_align: "top",

                                                                        textarea {
                                                                            width: "100%",
                                                                            min_height: "92px",
                                                                            padding: "8px 10px",
                                                                            background: COLOR_BG,
                                                                            border: "1px solid {COLOR_BORDER}",
                                                                            border_radius: "6px",
                                                                            color: COLOR_TEXT,
                                                                            font_family: "Consolas, 'Courier New', monospace",
                                                                            resize: "vertical",
                                                                            value: "{editing_hash_value}",
                                                                            oninput: move |event| editing_hash_value.set(event.value()),
                                                                        }
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        vertical_align: "top",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "8px",
                                                                            align_items: "center",
                                                                            flex_wrap: "wrap",

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: "#38a169",
                                                                                color: COLOR_TEXT_CONTRAST,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: active_hash_action.is_some(),
                                                                                onclick: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    let original_field = field.clone();
                                                                                    let existing_hash = hash_val.clone();
                                                                                    move |_| {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        let original_field = original_field.clone();
                                                                                        let existing_hash = existing_hash.clone();
                                                                                        let next_field = editing_hash_key().trim().to_string();
                                                                                        let next_value = editing_hash_value();
                                                                                        spawn(async move {
                                                                                            if next_field.is_empty() {
                                                                                                hash_status_message.set("保存失败：key 不能为空".to_string());
                                                                                                hash_status_error.set(true);
                                                                                                return;
                                                                                            }

                                                                                            if next_field != original_field && existing_hash.contains_key(&next_field) {
                                                                                                hash_status_message.set("保存失败：目标 key 已存在".to_string());
                                                                                                hash_status_error.set(true);
                                                                                                return;
                                                                                            }

                                                                                            hash_action.set(Some(format!("save:{}", original_field)));
                                                                                            hash_status_message.set(String::new());
                                                                                            hash_status_error.set(false);

                                                                                            let save_result = if next_field == original_field {
                                                                                                pool.hash_set_field(&key, &next_field, &next_value).await
                                                                                            } else {
                                                                                                match pool.hash_set_field(&key, &next_field, &next_value).await {
                                                                                                    Ok(_) => pool.hash_delete_field(&key, &original_field).await.map(|_| ()),
                                                                                                    Err(error) => Err(error),
                                                                                                }
                                                                                            };

                                                                                            match save_result {
                                                                                                Ok(_) => {
                                                                                                    editing_hash_field.set(None);
                                                                                                    editing_hash_key.set(String::new());
                                                                                                    editing_hash_value.set(String::new());
                                                                                                    hash_status_message.set("保存成功".to_string());
                                                                                                    hash_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                         pool.clone(),
                                                                                                         key.clone(),
                                                                                                         key_info,
                                                                                                         string_value,
                                                                                                         hash_value,
                                                                                                         list_value,
                                                                                                         set_value,
                                                                                                         zset_value,
                                                                                                         stream_value,
                                                                                                         is_binary,
                                                                                                         binary_format,
                                                                                                         serialization_data,
                binary_bytes,
                                                                                                         bitmap_info,
                                                                                                         loading,
                                                                                                         hash_cursor,
                                                                                                         hash_total,
                                                                                                         hash_has_more,
                                                                                                         list_has_more,
                                                                                                         list_total,
                                                                                                         set_cursor,
                                                                                                         set_total,
                                                                                                         set_has_more,
                                                                                                         zset_cursor,
                                                                                                         zset_total,
                                                                                                         zset_has_more,
                                                                                                     )
                                                                                                     .await
                                                                                                     {
                                                                                                        tracing::error!("{error}");
                                                                                                        hash_status_message.set(error);
                                                                                                        hash_status_error.set(true);
                                                                                                    } else {
                                                                                                        on_refresh.call(());
                                                                                                    }
                                                                                                }
                                                                                                Err(error) => {
                                                                                                    tracing::error!("Failed to save hash field: {}", error);
                                                                                                    hash_status_message.set(format!("保存失败：{error}"));
                                                                                                    hash_status_error.set(true);
                                                                                                }
                                                                                            }

                                                                                            hash_action.set(None);
                                                                                        });
                                                                                    }
                                                                                },

                                                                                if active_hash_action.as_deref() == Some(format!("save:{field}").as_str()) { "保存中..." } else { "保存" }
                                                                            }

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                color: COLOR_TEXT,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: active_hash_action.is_some(),
                                                                                onclick: move |_| {
                                                                                    editing_hash_field.set(None);
                                                                                    editing_hash_key.set(String::new());
                                                                                    editing_hash_value.set(String::new());
                                                                                },

                                                                                "取消"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                tr {
                                                                    key: "{field}",
                                                                    border_bottom: "1px solid {COLOR_BORDER}",
                                                                    background: COLOR_BG_SECONDARY,

                                                                    td {
                                                                        padding: "12px",
                                                                        color: COLOR_TEXT_SECONDARY,
                                                                        vertical_align: "top",

                                                                        "{index + 1}"
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        vertical_align: "top",

                                                                        div {
                                                                            color: COLOR_ACCENT,
                                                                            font_size: "13px",
                                                                            line_height: "1.5",
                                                                            word_break: "break-all",

                                                                            "{field}"
                                                                        }
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        vertical_align: "top",

                                                                        div {
                                                                            color: COLOR_TEXT,
                                                                            font_size: "13px",
                                                                            line_height: "1.6",
                                                                            white_space: "pre-wrap",
                                                                            word_break: "break-all",

                                                                            "{value}"
                                                                        }
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        vertical_align: "top",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "6px",
                                                                            align_items: "center",
                                                                            flex_wrap: "nowrap",

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(47, 133, 90, 0.16)",
                                                                                color: "#68d391",
                                                                                border: "1px solid rgba(104, 211, 145, 0.28)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: active_hash_action.is_some(),
                                                                                title: "复制值",
                                                                                aria_label: "复制值",
    onclick: {
                                                                    let value = value.clone();
                                                                    move |_| {
                                                                        match copy_value_to_clipboard(&value) {
                                                                            Ok(_) => {
                                                                                toast_manager.write().success("复制成功");
                                                                            }
                                                                            Err(error) => {
                                                                                tracing::error!("Failed to copy hash value: {}", error);
                                                                                toast_manager.write().error(&format!("复制失败：{error}"));
                                                                            }
                                                                        }
                                                                    }
                                                                },

                                                                                IconCopy { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(49, 130, 206, 0.18)",
                                                                                color: "#63b3ed",
                                                                                border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: active_hash_action.is_some(),
                                                                                title: "编辑 key-value",
                                                                                aria_label: "编辑 key-value",
                                                                                onclick: {
                                                                                    let field = field.clone();
                                                                                    let value = value.clone();
                                                                                    move |_| {
                                                                                        creating_hash_row.set(false);
                                                                                        new_hash_key.set(String::new());
                                                                                        new_hash_value.set(String::new());
                                                                                        editing_hash_field.set(Some(field.clone()));
                                                                                        editing_hash_key.set(field.clone());
                                                                                        editing_hash_value.set(value.clone());
                                                                                        hash_status_message.set(String::new());
                                                                                        hash_status_error.set(false);
                                                                                    }
                                                                                },

                                                                                IconEdit { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(197, 48, 48, 0.18)",
                                                                                color: "#f87171",
                                                                                border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: active_hash_action.is_some(),
                                                                                onclick: {
                                                                                    let field = field.clone();
                                                                                    move |_| {
                                                                                        deleting_hash_field.set(Some(HashDeleteTarget { field: field.clone() }));
                                                                                    }
                                                                                },
                                                                                title: "删除",
                                                                                aria_label: "删除",

                                                                                IconTrash { size: Some(15) }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if filtered_entries.is_empty() && !creating_hash_row() {
                                                            tr {
                                                                td {
                                                                    colspan: "4",
                                                                    padding: "20px 12px",
                                                                    color: COLOR_TEXT_SECONDARY,
                                                                    text_align: "center",

                                                                    if normalized_search.is_empty() {
                                                                        "当前 hash 没有字段"
                                                                    } else {
                                                                        "没有匹配的字段"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if let Some(target) = deleting_hash_field() {
                                                div {
                                                    position: "fixed",
                                                    top: "0",
                                                    left: "0",
                                                    right: "0",
                                                    bottom: "0",
                                                    background: "rgba(0, 0, 0, 0.7)",
                                                    display: "flex",
                                                    align_items: "center",
                                                    justify_content: "center",
                                                    z_index: "1000",
                                                    animation: if deleting_hash_field_exiting() { "backdropFadeOut 0.2s ease-out forwards" } else { "backdropFadeIn 0.2s ease-out" },

                                                    style {
                                                        r#"
                                                        @keyframes backdropFadeIn {{
                                                            from {{ opacity: 0; }}
                                                            to {{ opacity: 1; }}
                                                        }}
                                                        @keyframes backdropFadeOut {{
                                                            from {{ opacity: 1; }}
                                                            to {{ opacity: 0; }}
                                                        }}
                                                        @keyframes modalFadeIn {{
                                                            from {{ opacity: 0; transform: scale(0.9); }}
                                                            to {{ opacity: 1; transform: scale(1); }}
                                                        }}
                                                        @keyframes modalFadeOut {{
                                                            from {{ opacity: 1; transform: scale(1); }}
                                                            to {{ opacity: 0; transform: scale(0.9); }}
                                                        }}
                                                        "#
                                                    }

                                                    div {
                                                        width: "420px",
                                                        max_width: "calc(100vw - 32px)",
                                                        background: COLOR_BG_SECONDARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "10px",
                                                        padding: "20px",
                                                        animation: if deleting_hash_field_exiting() { "modalFadeOut 0.2s ease-out forwards" } else { "modalFadeIn 0.2s ease-out" },

                                                        h3 {
                                                            color: COLOR_TEXT,
                                                            margin: "0 0 12px 0",
                                                            font_size: "18px",

                                                            "确认删除"
                                                        }

                                                        p {
                                                            color: COLOR_TEXT_SOFT,
                                                            margin: "0 0 18px 0",
                                                            line_height: "1.6",
                                                            word_break: "break-all",

                                                            "确定删除 hash field '{target.field}' 吗？"
                                                        }

                                                        div {
                                                            display: "flex",
                                                            justify_content: "flex_end",
                                                            gap: "8px",

                                                            button {
                                                                padding: "8px 12px",
                                                                background: COLOR_BG_TERTIARY,
                                                                color: COLOR_TEXT,
                                                                border: "none",
                                                                border_radius: "6px",
                                                                cursor: "pointer",
                                                                disabled: active_hash_action.is_some(),
                                                                onclick: {
                                                                    let mut deleting_hash_field = deleting_hash_field.clone();
                                                                    let mut deleting_hash_field_exiting = deleting_hash_field_exiting.clone();
                                                                    move |_| {
                                                                        deleting_hash_field_exiting.set(true);
                                                                        let mut dhf = deleting_hash_field.clone();
                                                                        let mut dhfe = deleting_hash_field_exiting.clone();
                                                                        spawn(async move {
                                                                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                                                            dhf.set(None);
                                                                            dhfe.set(false);
                                                                        });
                                                                    }
                                                                },

                                                                "取消"
                                                            }

                                                            button {
                                                                padding: "8px 12px",
                                                                background: "#c53030",
                                                                color: COLOR_TEXT_CONTRAST,
                                                                border: "none",
                                                                border_radius: "6px",
                                                                cursor: "pointer",
                                                                disabled: active_hash_action.is_some(),
                                                                onclick: {
                                                                    let pool = connection_pool.clone();
                                                                    let key = display_key.clone();
                                                                    let field = target.field.clone();
                                                                    let delete_action = format!("delete:{}", target.field);
                                                                    let mut deleting_hash_field = deleting_hash_field.clone();
                                                                    let mut deleting_hash_field_exiting = deleting_hash_field_exiting.clone();
                                                                    move |_| {
                                                                        let pool = pool.clone();
                                                                        let key = key.clone();
                                                                        let field = field.clone();
                                                                        let delete_action = delete_action.clone();
                                                                        spawn(async move {
                                                                            hash_action.set(Some(delete_action));
                                                                            hash_status_message.set(String::new());
                                                                            hash_status_error.set(false);

                                                                            match pool.hash_delete_field(&key, &field).await {
                                                                                Ok(_) => {
                                                                                    deleting_hash_field_exiting.set(true);
                                                                                    let mut dhf = deleting_hash_field.clone();
                                                                                    let mut dhfe = deleting_hash_field_exiting.clone();
                                                                                    if editing_hash_field().as_deref() == Some(field.as_str()) {
                                                                                        editing_hash_field.set(None);
                                                                                        editing_hash_key.set(String::new());
                                                                                        editing_hash_value.set(String::new());
                                                                                    }
hash_status_message.set("删除成功".to_string());
                    hash_status_error.set(false);
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    dhf.set(None);
                    dhfe.set(false);
            if let Err(error) = load_key_data(
                                                                                                     pool.clone(),
                                                                                                     key.clone(),
                                                                                                     key_info,
                                                                                                     string_value,
                                                                                                     hash_value,
                                                                                                     list_value,
                                                                                                     set_value,
                                                                                                     zset_value,
                                                                                                     stream_value,
                                                                                                     is_binary,
                                                                                                     binary_format,
                                                                                                     serialization_data,
                binary_bytes,
                                                                                                     bitmap_info,
                                                                                                     loading,
                                                                                                     hash_cursor,
                                                                                                     hash_total,
                                                                                                     hash_has_more,
                                                                                                     list_has_more,
                                                                                                     list_total,
                                                                                                     set_cursor,
                                                                                                     set_total,
                                                                                                     set_has_more,
                                                                                                     zset_cursor,
                                                                                                     zset_total,
                                                                                                     zset_has_more,
                                                                                                 )
                                                                                     .await
                                                                                     {
                                                                                        tracing::error!("{error}");
                                                                                        hash_status_message.set(error);
                                                                                        hash_status_error.set(true);
                                                                                    } else {
                                                                                        on_refresh.call(());
                                                                                    }
                                                                                }
                                                                                Err(error) => {
                                                                                    tracing::error!("Failed to delete hash field: {}", error);
                                                                                    hash_status_message.set(format!("删除失败：{error}"));
                                                                                    hash_status_error.set(true);
                                                                                }
                                                                            }

                                                                            hash_action.set(None);
                                                                        });
                                                                    }
                                                                },

                                                                if active_hash_action.as_deref() == Some(format!("delete:{}", target.field).as_str()) {
                                                                    "删除中..."
                                                                } else {
                                                                    "确认删除"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if hash_loading_more() {
                                                div {
                                                    padding: "12px",
                                                    text_align: "center",
                                                    color: COLOR_TEXT_SECONDARY,
                                                    font_size: "13px",

                                                    "加载中..."
                                                }
                                            }

                                            if hash_has_more() && !hash_loading_more() {
                                                div {
                                                    padding: "8px",
                                                    text_align: "center",
                                                    color: COLOR_TEXT_SUBTLE,
                                                    font_size: "12px",

                                                    "向下滚动加载更多..."
                                                }
                                            }
                                            }
                                        }
                                    }
                                    KeyType::List => {
                                        rsx! {
                                            div {
                                                display: "flex",
                                                flex_direction: "column",
                                                height: "100%",
                                                min_height: "0",

                                                display: "flex",
                                                justify_content: "space_between",
                                                align_items: "center",
                                                gap: "12px",
                                                margin_bottom: "12px",

                                                div {
                                                    display: "flex",
                                                    gap: "8px",
                                                    align_items: "center",

                                                    input {
                                                        width: "300px",
                                                        padding: "8px 10px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "6px",
                                                        color: COLOR_TEXT,
                                                        value: "{new_list_value}",
                                                        placeholder: "输入新元素值",
                                                        oninput: move |event| new_list_value.set(event.value()),
                                                    }

                                                    button {
                                                        padding: "8px 12px",
                                                        background: "#38a169",
                                                        color: COLOR_TEXT_CONTRAST,
                                                        border: "none",
                                                        border_radius: "6px",
                                                        cursor: "pointer",
                                                        disabled: list_action().is_some(),
                                                        onclick: {
                                                            let pool = connection_pool.clone();
                                                            let key = display_key.clone();
                                                            move |_| {
                                                                let pool = pool.clone();
                                                                let key = key.clone();
                                                                let value = new_list_value();
                                                                spawn(async move {
                                                                    if value.trim().is_empty() {
                                                                        list_status_message.set("值不能为空".to_string());
                                                                        list_status_error.set(true);
                                                                        return;
                                                                    }

                                                                    list_action.set(Some("push".to_string()));
                                                                    list_status_message.set(String::new());
                                                                    list_status_error.set(false);

                                                                    match pool.list_push(&key, &value, true).await {
                                                                        Ok(_) => {
                                                                            new_list_value.set(String::new());
                                                                            list_status_message.set("添加成功".to_string());
                                                                            list_status_error.set(false);
                                                                            if let Err(error) = load_key_data(
                                                                                pool.clone(),
                                                                                key.clone(),
                                                                                key_info,
                                                                                string_value,
                                                                                hash_value,
                                                                                list_value,
                                                                                set_value,
                                                                                zset_value,
                                                                                stream_value,
                                                                                is_binary,
                                                                                binary_format,
                                                                                serialization_data,
                binary_bytes,
                                                                                bitmap_info,
                                                                                loading,
                                                                                hash_cursor,
                                                                                hash_total,
                                                                                hash_has_more,
                                                                                list_has_more,
                                                                                list_total,
                                                                                set_cursor,
                                                                                set_total,
                                                                                set_has_more,
                                                                                zset_cursor,
                                                                                zset_total,
                                                                                zset_has_more,
                                                                            ).await {
                                                                                tracing::error!("{error}");
                                                                            } else {
                                                                                on_refresh.call(());
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            list_status_message.set(format!("添加失败：{error}"));
                                                                            list_status_error.set(true);
                                                                        }
                                                                    }
                                                                    list_action.set(None);
                                                                });
                                                            }
                                                        },

                                                        if list_action().as_deref() == Some("push") { "添加中..." } else { "LPUSH" }
                                                    }

                                                    button {
                                                        padding: "8px 12px",
                                                        background: "#0e639c",
                                                        color: COLOR_TEXT_CONTRAST,
                                                        border: "none",
                                                        border_radius: "6px",
                                                        cursor: "pointer",
                                                        disabled: list_action().is_some(),
                                                        onclick: {
                                                            let pool = connection_pool.clone();
                                                            let key = display_key.clone();
                                                            move |_| {
                                                                let pool = pool.clone();
                                                                let key = key.clone();
                                                                let value = new_list_value();
                                                                spawn(async move {
                                                                    if value.trim().is_empty() {
                                                                        list_status_message.set("值不能为空".to_string());
                                                                        list_status_error.set(true);
                                                                        return;
                                                                    }

                                                                    list_action.set(Some("rpush".to_string()));
                                                                    match pool.list_push(&key, &value, false).await {
                                                                        Ok(_) => {
                                                                            new_list_value.set(String::new());
                                                                            list_status_message.set("添加成功".to_string());
                                                                            list_status_error.set(false);
                                                                            if let Err(error) = load_key_data(
                                                                                pool.clone(),
                                                                                key.clone(),
                                                                                key_info,
                                                                                string_value,
                                                                                hash_value,
                                                                                list_value,
                                                                                set_value,
                                                                                zset_value,
                                                                                stream_value,
                                                                                is_binary,
                                                                                binary_format,
                                                                                serialization_data,
                binary_bytes,
                                                                                bitmap_info,
                                                                                loading,
                                                                                hash_cursor,
                                                                                hash_total,
                                                                                hash_has_more,
                                                                                list_has_more,
                                                                                list_total,
                                                                                set_cursor,
                                                                                set_total,
                                                                                set_has_more,
                                                                                zset_cursor,
                                                                                zset_total,
                                                                                zset_has_more,
                                                                            ).await {
                                                                                tracing::error!("{error}");
                                                                            } else {
                                                                                on_refresh.call(());
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            list_status_message.set(format!("添加失败：{error}"));
                                                                            list_status_error.set(true);
                                                                        }
                                                                    }
                                                                    list_action.set(None);
                                                                });
                                                            }
                                                        },

                                                        "RPUSH"
                                                    }

                                                    div {
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "13px",

                                                        "List Items ({list_val.len()}/{list_total()})"
                                                    }
                                                }

                                                button {
                                                    margin_left: "auto",
                                                    flex_shrink: "0",
                                                    padding: "6px 10px",
                                                    background: "rgba(47, 133, 90, 0.16)",
                                                    color: COLOR_SUCCESS,
                                                    border: "1px solid rgba(104, 211, 145, 0.28)",
                                                    border_radius: "6px",
                                                    cursor: "pointer",
                                                    display: "flex",
                                                    align_items: "center",
                                                    gap: "4px",
                                                    title: "复制",
                                                    onclick: {
                                                        let list = list_val.clone();
                                                        move |_| {
                                                            let json = serde_json::to_string_pretty(&list).unwrap_or_default();
                                                            match copy_value_to_clipboard(&json) {
                                                                Ok(_) => {
                                                                    toast_manager.write().success("复制成功");
                                                                }
                                                                Err(error) => {
                                                                    toast_manager.write().error(&format!("复制失败：{error}"));
                                                                }
                                                            }
                                                        }
                                                    },

                                                    IconCopy { size: Some(14) }
                                                    "复制"
                                                }
                                            }

                                            if !list_status_message.read().is_empty() {
                                                div {
                                                    margin_bottom: "12px",
                                                    padding: "8px 12px",
                                                    background: if list_status_error() { STATUS_ERROR_BG } else { STATUS_SUCCESS_BG },
                                                    border_radius: "6px",
                                                    color: if list_status_error() { COLOR_ERROR } else { COLOR_SUCCESS },
                                                    font_size: "13px",

                                                    "{list_status_message}"
                                                }
                                            }

                                            if list_val.len() > LARGE_KEY_THRESHOLD {
                                                LargeKeyWarning {
                                                    key_type: "List".to_string(),
                                                    size: list_val.len(),
                                                    threshold: LARGE_KEY_THRESHOLD,
                                                }
                                            }

                                            div {
                                                flex: "1",
                                                min_height: "0",
                                                overflow_x: "auto",
                                                overflow_y: "auto",
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "8px",
                                                background: COLOR_BG_SECONDARY,
                                                onscroll: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    move |e| {
                                                        let scroll_top = e.data().scroll_top() as i32;
                                                        let scroll_height = e.data().scroll_height() as i32;
                                                        let client_height = e.data().client_height() as i32;
                                                        
                                                        if list_has_more() && !list_loading_more() && scroll_height - scroll_top - client_height < 200 {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let total = list_total();
                                                            spawn(async move {
                                                                load_more_list(
                                                                    pool,
                                                                    key,
                                                                    list_value,
                                                                    list_has_more,
                                                                    list_loading_more,
                                                                    total,
                                                                ).await;
                                                            });
                                                        }
                                                    }
                                                },

                                                table {
                                                    width: "100%",
                                                    border_collapse: "collapse",

                                                    thead {
                                                        tr {
                                                            background: COLOR_BG_TERTIARY,
                                                            border_bottom: "1px solid {COLOR_BORDER}",

                                                            th {
                                                                width: "72px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Index"
                                                            }

                                                            th {
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Value"
                                                            }

                                                            th {
                                                                width: "156px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Action"
                                                            }
                                                        }
                                                    }

                                                    tbody {
                                                        for (idx, value) in list_val.iter().enumerate() {
                                                            if editing_list_index() == Some(idx) {
                                                                tr {
                                                                    background: ROW_EDIT_BG,
                                                                    border_bottom: "1px solid {COLOR_BORDER}",

                                                                    td {
                                                                        padding: "12px",
                                                                        color: COLOR_ACCENT,

                                                                        "{idx}"
                                                                    }

                                                                    td {
                                                                        padding: "12px",

                                                                        input {
                                                                            width: "100%",
                                                                            padding: "8px 10px",
                                                                            background: COLOR_BG,
                                                                            border: "1px solid {COLOR_BORDER}",
                                                                            border_radius: "6px",
                                                                            color: COLOR_TEXT,
                                                                            value: "{editing_list_value}",
                                                                            oninput: move |event| editing_list_value.set(event.value()),
                                                                        }
                                                                    }

                                                                    td {
                                                                        padding: "12px",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "8px",

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: "#38a169",
                                                                                color: COLOR_TEXT_CONTRAST,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: list_action().is_some(),
                                                                                onclick: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    let idx = idx as i64;
                                                                                    move |_| {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        let new_val = editing_list_value();
                                                                                        spawn(async move {
                                                                                            list_action.set(Some("set".to_string()));
                                                                                            match pool.list_set(&key, idx, &new_val).await {
                                                                                                Ok(_) => {
                                                                                                    editing_list_index.set(None);
                                                                                                    list_status_message.set("修改成功".to_string());
                                                                                                    list_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                         pool.clone(),
                                                                                                         key.clone(),
                                                                                                         key_info,
                                                                                                         string_value,
                                                                                                         hash_value,
                                                                                                         list_value,
                                                                                                         set_value,
                                                                                                         zset_value,
                                                                                                         stream_value,
                                                                                                         is_binary,
                                                                                                         binary_format,
                                                                                                         serialization_data,
                binary_bytes,
                                                                                                         bitmap_info,
                                                                                                         loading,
                                                                                                         hash_cursor,
                                                                                                         hash_total,
                                                                                                         hash_has_more,
                                                                                                         list_has_more,
                                                                                                         list_total,
                                                                                                         set_cursor,
                                                                                                         set_total,
                                                                                                         set_has_more,
                                                                                                         zset_cursor,
                                                                                                         zset_total,
                                                                                                         zset_has_more,
                                                                                                     ).await {
                                                                                                        tracing::error!("{error}");
                                                                                                    } else {
                                                                                                        on_refresh.call(());
                                                                                                    }
                                                                                                }
                                                                                                Err(error) => {
                                                                                                    list_status_message.set(format!("修改失败：{error}"));
                                                                                                    list_status_error.set(true);
                                                                                                }
                                                                                            }
                                                                                            list_action.set(None);
                                                                                        });
                                                                                    }
                                                                                },

                                                                                "保存"
                                                                            }

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                color: COLOR_TEXT,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                onclick: move |_| {
                                                                                    editing_list_index.set(None);
                                                                                },

                                                                                "取消"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                tr {
                                                                    key: "{idx}",
                                                                    border_bottom: "1px solid {COLOR_BORDER}",
                                                                    background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },

                                                                    td {
                                                                        padding: "12px",
                                                                        color: COLOR_TEXT_SECONDARY,

                                                                        "{idx}"
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        color: COLOR_TEXT,
                                                                        font_family: "Consolas, monospace",
                                                                        font_size: "13px",
                                                                        word_break: "break-all",

                                                                        "{value}"
                                                                    }

                                                                    td {
                                                                        padding: "12px",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "6px",

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(47, 133, 90, 0.16)",
                                                                                color: "#68d391",
                                                                                border: "1px solid rgba(104, 211, 145, 0.28)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                title: "复制",
                                                                                onclick: {
                                                                                    let value = value.clone();
                                                                                    move |_| {
                                                                                        match copy_value_to_clipboard(&value) {
                                                                                            Ok(_) => {
                                                                                                toast_manager.write().success("复制成功");
                                                                                            }
                                                                                            Err(error) => {
                                                                                                toast_manager.write().error(&format!("复制失败：{error}"));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                },

                                                                                IconCopy { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(49, 130, 206, 0.18)",
                                                                                color: "#63b3ed",
                                                                                border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                title: "编辑",
                                                                                onclick: {
                                                                                    let value = value.clone();
                                                                                    move |_| {
                                                                                        editing_list_index.set(Some(idx));
                                                                                        editing_list_value.set(value.clone());
                                                                                    }
                                                                                },

                                                                                IconEdit { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(197, 48, 48, 0.18)",
                                                                                color: "#f87171",
                                                                                border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                title: "删除",
                                                                                onclick: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    let value = value.clone();
                                                                                    move |_| {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        let value = value.clone();
                                                                                        spawn(async move {
                                                                                            list_action.set(Some("remove".to_string()));
                                                                                            match pool.list_remove(&key, 1, &value).await {
                                                                                                Ok(_) => {
                                                                                                    list_status_message.set("删除成功".to_string());
                                                                                                    list_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                         pool.clone(),
                                                                                                         key.clone(),
                                                                                                         key_info,
                                                                                                         string_value,
                                                                                                         hash_value,
                                                                                                         list_value,
                                                                                                         set_value,
                                                                                                         zset_value,
                                                                                                         stream_value,
                                                                                                         is_binary,
                                                                                                         binary_format,
                                                                                                         serialization_data,
                binary_bytes,
                                                                                                         bitmap_info,
                                                                                                         loading,
                                                                                                         hash_cursor,
                                                                                                         hash_total,
                                                                                                         hash_has_more,
                                                                                                         list_has_more,
                                                                                                         list_total,
                                                                                                         set_cursor,
                                                                                                         set_total,
                                                                                                         set_has_more,
                                                                                                         zset_cursor,
                                                                                                         zset_total,
                                                                                                         zset_has_more,
                                                                                                     ).await {
                                                                                                        tracing::error!("{error}");
                                                                                                    } else {
                                                                                                        on_refresh.call(());
                                                                                                    }
                                                                                                }
                                                                                                Err(error) => {
                                                                                                    list_status_message.set(format!("删除失败：{error}"));
                                                                                                    list_status_error.set(true);
                                                                                                }
                                                                                            }
                                                                                            list_action.set(None);
                                                                                        });
                                                                                    }
                                                                                },

                                                                                IconTrash { size: Some(15) }
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
                                    KeyType::Set => {
                                        rsx! {
                                            div {
                                                display: "flex",
                                                flex_direction: "column",
                                                height: "100%",
                                                min_height: "0",

                                                display: "flex",
                                                justify_content: "space_between",
                                                align_items: "center",
                                                gap: "12px",
                                                margin_bottom: "12px",

                                                div {
                                                    display: "flex",
                                                    gap: "8px",
                                                    align_items: "center",

                                                    input {
                                                        width: "200px",
                                                        padding: "8px 10px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "6px",
                                                        color: COLOR_TEXT,
                                                        value: "{set_search}",
                                                        placeholder: "搜索成员",
                                                        oninput: {
                                                            let pool = connection_pool.clone();
                                                            let key = display_key.clone();
                                                            move |event| {
                                                                let value = event.value();
                                                                let was_empty = set_search().is_empty();
                                                                set_search.set(value.clone());
                                                                
                                                                if value.is_empty() && !was_empty {
                                                                    let pool = pool.clone();
                                                                    let key = key.clone();
                                                                    spawn(async move {
                                                                        if let Err(e) = load_key_data(
                                                                            pool,
                                                                            key,
                                                                            key_info,
                                                                            string_value,
                                                                            hash_value,
                                                                            list_value,
                                                                            set_value,
                                                                            zset_value,
                                                                            stream_value,
                                                                            is_binary,
                                                                            binary_format,
                                                                            serialization_data,
                binary_bytes,
                                                                            bitmap_info,
                                                                            loading,
                                                                            hash_cursor,
                                                                            hash_total,
                                                                            hash_has_more,
                                                                            list_has_more,
                                                                            list_total,
                                                                            set_cursor,
                                                                            set_total,
                                                                            set_has_more,
                                                                            zset_cursor,
                                                                            zset_total,
                                                                            zset_has_more,
                                                                        ).await {
                                                                            tracing::error!("重新加载 set 数据失败: {}", e);
                                                                        }
                                                                    });
                                                                }
                                                            }
                                                        },
                                                    }

                                                    if set_total() > PAGE_SIZE {
                                                        button {
                                                            padding: "6px 10px",
                                                            background: if set_search().len() >= 2 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                            color: if set_search().len() >= 2 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT_SECONDARY },
                                                            border: "1px solid {COLOR_BORDER}",
                                                            border_radius: "6px",
                                                            cursor: if set_search().len() >= 2 { "pointer" } else { "not-allowed" },
                                                            font_size: "12px",
                                                            disabled: set_search().len() < 2 || set_loading_more(),
                                                            onclick: {
                                                                let pool = connection_pool.clone();
                                                                let key = display_key.clone();
                                                                move |_| {
                                                                    let pool = pool.clone();
                                                                    let key = key.clone();
                                                                    let pattern = set_search();
                                                                    spawn(async move {
                                                                        search_set_server(
                                                                            pool,
                                                                            key,
                                                                            pattern,
                                                                            set_value,
                                                                            set_cursor,
                                                                            set_has_more,
                                                                            set_loading_more,
                                                                        ).await;
                                                                    });
                                                                }
                                                            },

                                                            if set_loading_more() { "搜索中..." } else { "服务端搜索" }
                                                        }
                                                    }

                                                    input {
                                                        width: "200px",
                                                        padding: "8px 10px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "6px",
                                                        color: COLOR_TEXT,
                                                        value: "{new_set_member}",
                                                        placeholder: "输入新成员",
                                                        oninput: move |event| new_set_member.set(event.value()),
                                                    }

                                                    button {
                                                        padding: "8px 12px",
                                                        background: "#38a169",
                                                        color: COLOR_TEXT_CONTRAST,
                                                        border: "none",
                                                        border_radius: "6px",
                                                        cursor: "pointer",
                                                        disabled: set_action().is_some(),
                                                        onclick: {
                                                            let pool = connection_pool.clone();
                                                            let key = display_key.clone();
                                                            move |_| {
                                                                let pool = pool.clone();
                                                                let key = key.clone();
                                                                let member = new_set_member();
                                                                spawn(async move {
                                                                    if member.trim().is_empty() {
                                                                        set_status_message.set("成员不能为空".to_string());
                                                                        set_status_error.set(true);
                                                                        return;
                                                                    }

                                                                    set_action.set(Some("add".to_string()));
                                                                    set_status_message.set(String::new());
                                                                    set_status_error.set(false);

                                                                    match pool.set_add(&key, &member).await {
                                                                        Ok(true) => {
                                                                            new_set_member.set(String::new());
                                                                            set_status_message.set("添加成功".to_string());
                                                                            set_status_error.set(false);
                                                                            if let Err(error) = load_key_data(
                                                                                pool.clone(),
                                                                                key.clone(),
                                                                                key_info,
                                                                                string_value,
                                                                                hash_value,
                                                                                list_value,
                                                                                set_value,
                                                                                zset_value,
                                                                                stream_value,
                                                                                is_binary,
                                                                                binary_format,
                                                                                serialization_data,
                binary_bytes,
                                                                                bitmap_info,
                                                                                loading,
                                                                                hash_cursor,
                                                                                hash_total,
                                                                                hash_has_more,
                                                                                list_has_more,
                                                                                list_total,
                                                                                set_cursor,
                                                                                set_total,
                                                                                set_has_more,
                                                                                zset_cursor,
                                                                                zset_total,
                                                                                zset_has_more,
                                                                            ).await {
                                                                                tracing::error!("{error}");
                                                                            } else {
                                                                                on_refresh.call(());
                                                                            }
                                                                        }
                                                                        Ok(false) => {
                                                                            set_status_message.set("成员已存在".to_string());
                                                                            set_status_error.set(true);
                                                                        }
                                                                        Err(error) => {
                                                                            set_status_message.set(format!("添加失败：{error}"));
                                                                            set_status_error.set(true);
                                                                        }
                                                                    }
                                                                    set_action.set(None);
                                                                });
                                                            }
                                                        },

                                                        if set_action().as_deref() == Some("add") { "添加中..." } else { "添加成员" }
                                                    }

                                                    div {
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "13px",

                                                        "Set Members ({set_val.len()}/{set_total()})"
                                                    }
                                                }

                                                button {
                                                    margin_left: "auto",
                                                    flex_shrink: "0",
                                                    padding: "6px 10px",
                                                    background: "rgba(47, 133, 90, 0.16)",
                                                    color: COLOR_SUCCESS,
                                                    border: "1px solid rgba(104, 211, 145, 0.28)",
                                                    border_radius: "6px",
                                                    cursor: "pointer",
                                                    display: "flex",
                                                    align_items: "center",
                                                    gap: "4px",
                                                    title: "复制",
                                                    onclick: {
                                                        let set = set_val.clone();
                                                        move |_| {
                                                            let json = serde_json::to_string_pretty(&set).unwrap_or_default();
                                                            match copy_value_to_clipboard(&json) {
                                                                Ok(_) => {
                                                                    toast_manager.write().success("复制成功");
                                                                }
                                                                Err(error) => {
                                                                    toast_manager.write().error(&format!("复制失败：{error}"));
                                                                }
                                                            }
                                                        }
                                                    },

                                                    IconCopy { size: Some(14) }
                                                    "复制"
                                                }
                                            }

                                            if !set_status_message.read().is_empty() {
                                                div {
                                                    margin_bottom: "12px",
                                                    padding: "8px 12px",
                                                    background: if set_status_error() { STATUS_ERROR_BG } else { STATUS_SUCCESS_BG },
                                                    border_radius: "6px",
                                                    color: if set_status_error() { COLOR_ERROR } else { COLOR_SUCCESS },
                                                    font_size: "13px",

                                                    "{set_status_message}"
                                                }
                                            }

                                            if set_val.len() > LARGE_KEY_THRESHOLD {
                                                LargeKeyWarning {
                                                    key_type: "Set".to_string(),
                                                    size: set_val.len(),
                                                    threshold: LARGE_KEY_THRESHOLD,
                                                }
                                            }

                                            div {
                                                flex: "1",
                                                min_height: "0",
                                                overflow_x: "auto",
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "8px",
                                                background: COLOR_BG_SECONDARY,
                                                overflow_y: "auto",
                                                onscroll: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    move |e| {
                                                        let scroll_top = e.data().scroll_top() as i32;
                                                        let scroll_height = e.data().scroll_height() as i32;
                                                        let client_height = e.data().client_height() as i32;
                                                        
                                                        if set_has_more() && !set_loading_more() && scroll_height - scroll_top - client_height < 200 {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let cursor = set_cursor();
                                                            spawn(async move {
                                                                load_more_set(
                                                                    pool,
                                                                    key,
                                                                    set_value,
                                                                    cursor,
                                                                    set_cursor,
                                                                    set_has_more,
                                                                    set_loading_more,
                                                                ).await;
                                                            });
                                                        }
                                                    }
                                                },

                                                table {
                                                    width: "100%",
                                                    border_collapse: "collapse",

                                                    thead {
                                                        tr {
                                                            background: COLOR_BG_TERTIARY,
                                                            position: "sticky",
                                                            top: "0",

                                                            th {
                                                                width: "72px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",
                                                                border_bottom: "1px solid {COLOR_BORDER}",

                                                                "#"
                                                            }

                                                            th {
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",
                                                                border_bottom: "1px solid {COLOR_BORDER}",

                                                                "Member"
                                                            }

                                                            th {
                                                                width: "100px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",
                                                                border_bottom: "1px solid {COLOR_BORDER}",

                                                                "Action"
                                                            }
                                                        }
                                                    }

                                                    tbody {
                                                        for (idx, member) in filtered_set_members.iter().enumerate() {
                                                            if editing_set_member() == Some(member.clone()) {
                                                                tr {
                                                                    key: "edit-{member}",
                                                                    background: ROW_EDIT_BG,
                                                                    border_bottom: "1px solid {COLOR_BORDER}",

                                                                    td {
                                                                        padding: "10px 12px",
                                                                        color: COLOR_TEXT_SECONDARY,

                                                                        "{idx + 1}"
                                                                    }

                                                                    td {
                                                                        padding: "10px 12px",

                                                                        input {
                                                                            width: "100%",
                                                                            padding: "8px 10px",
                                                                            background: COLOR_BG,
                                                                            border: "1px solid {COLOR_BORDER}",
                                                                            border_radius: "6px",
                                                                            color: COLOR_TEXT,
                                                                            font_family: "Consolas, monospace",
                                                                            font_size: "13px",
                                                                            value: "{editing_set_member_value}",
                                                                            oninput: move |event| editing_set_member_value.set(event.value()),
                                                                        }
                                                                    }

                                                                    td {
                                                                        padding: "10px 12px",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "6px",

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: "#38a169",
                                                                                color: COLOR_TEXT_CONTRAST,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: set_action().is_some(),
                                                                                onclick: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    let old_member = member.clone();
                                                                                    move |_| {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        let old_member = old_member.clone();
                                                                                        let new_member = editing_set_member_value();
                                                                                        spawn(async move {
                                                                                            if new_member.trim().is_empty() {
                                                                                                set_status_message.set("成员不能为空".to_string());
                                                                                                set_status_error.set(true);
                                                                                                return;
                                                                                            }

                                                                                            set_action.set(Some(format!("edit:{}", old_member)));
                                                                                            set_status_message.set(String::new());
                                                                                            set_status_error.set(false);

                                                                                            let edit_result = if new_member == old_member {
                                                                                                Ok(())
                                                                                            } else {
                                                                                                match pool.set_remove(&key, &old_member).await {
                                                                                                    Ok(_) => pool.set_add(&key, &new_member).await.map(|_| ()),
                                                                                                    Err(e) => Err(e),
                                                                                                }
                                                                                            };

                                                                                            match edit_result {
                                                                                                Ok(_) => {
                                                                                                    editing_set_member.set(None);
                                                                                                    editing_set_member_value.set(String::new());
                                                                                                    set_status_message.set("修改成功".to_string());
                                                                                                    set_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                         pool.clone(),
                                                                                                         key.clone(),
                                                                                                         key_info,
                                                                                                         string_value,
                                                                                                         hash_value,
                                                                                                         list_value,
                                                                                                         set_value,
                                                                                                         zset_value,
                                                                                                         stream_value,
                                                                                                         is_binary,
                                                                                                         binary_format,
                                                                                                         serialization_data,
                binary_bytes,
                                                                                                         bitmap_info,
                                                                                                         loading,
                                                                                                         hash_cursor,
                                                                                                         hash_total,
                                                                                                         hash_has_more,
                                                                                                         list_has_more,
                                                                                                         list_total,
                                                                                                         set_cursor,
                                                                                                         set_total,
                                                                                                         set_has_more,
                                                                                                         zset_cursor,
                                                                                                         zset_total,
                                                                                                         zset_has_more,
                                                                                                     ).await {
                                                                                                        tracing::error!("{error}");
                                                                                                    } else {
                                                                                                        on_refresh.call(());
                                                                                                    }
                                                                                                }
                                                                                                Err(error) => {
                                                                                                    set_status_message.set(format!("修改失败：{error}"));
                                                                                                    set_status_error.set(true);
                                                                                                }
                                                                                            }

                                                                                            set_action.set(None);
                                                                                        });
                                                                                    }
                                                                                },

                                                                                if set_action().as_deref() == Some(format!("edit:{}", member).as_str()) { "保存中..." } else { "保存" }
                                                                            }

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                color: COLOR_TEXT,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                onclick: move |_| {
                                                                                    editing_set_member.set(None);
                                                                                    editing_set_member_value.set(String::new());
                                                                                },

                                                                                "取消"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                tr {
                                                                    key: "{member}",
                                                                    border_bottom: "1px solid {COLOR_BORDER}",
                                                                    background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },

                                                                    td {
                                                                        padding: "10px 12px",
                                                                        color: COLOR_TEXT_SECONDARY,

                                                                        "{idx + 1}"
                                                                    }

                                                                    td {
                                                                        padding: "10px 12px",
                                                                        color: COLOR_TEXT,
                                                                        font_family: "Consolas, monospace",
                                                                        font_size: "13px",
                                                                        word_break: "break-all",

                                                                        "{member}"
                                                                    }

                                                                    td {
                                                                        padding: "10px 12px",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "6px",

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(47, 133, 90, 0.16)",
                                                                                color: "#68d391",
                                                                                border: "1px solid rgba(104, 211, 145, 0.28)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                title: "复制",
                                                                                onclick: {
                                                                                    let member = member.clone();
                                                                                    move |_| {
                                                                                        match copy_value_to_clipboard(&member) {
                                                                                            Ok(_) => {
                                                                                                toast_manager.write().success("复制成功");
                                                                                            }
                                                                                            Err(error) => {
                                                                                                toast_manager.write().error(&format!("复制失败：{error}"));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                },

                                                                                IconCopy { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(49, 130, 206, 0.18)",
                                                                                color: "#63b3ed",
                                                                                border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: set_action().is_some(),
                                                                                title: "编辑",
                                                                                onclick: {
                                                                                    let member = member.clone();
                                                                                    move |_| {
                                                                                        editing_set_member.set(Some(member.clone()));
                                                                                        editing_set_member_value.set(member.clone());
                                                                                    }
                                                                                },

                                                                                IconEdit { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(197, 48, 48, 0.18)",
                                                                                color: "#f87171",
                                                                                border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: set_action().is_some(),
                                                                                title: "删除",
                                                                                onclick: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    let member = member.clone();
                                                                                    move |_| {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        let member = member.clone();
                                                                                        spawn(async move {
                                                                                            set_action.set(Some(format!("delete:{}", member)));
                                                                                            match pool.set_remove(&key, &member).await {
                                                                                                Ok(_) => {
                                                                                                    set_status_message.set("删除成功".to_string());
                                                                                                    set_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                         pool.clone(),
                                                                                                         key.clone(),
                                                                                                         key_info,
                                                                                                         string_value,
                                                                                                         hash_value,
                                                                                                         list_value,
                                                                                                         set_value,
                                                                                                         zset_value,
                                                                                                         stream_value,
                                                                                                         is_binary,
                                                                                                         binary_format,
                                                                                                         serialization_data,
                binary_bytes,
                                                                                                         bitmap_info,
                                                                                                         loading,
                                                                                                         hash_cursor,
                                                                                                         hash_total,
                                                                                                         hash_has_more,
                                                                                                         list_has_more,
                                                                                                         list_total,
                                                                                                         set_cursor,
                                                                                                         set_total,
                                                                                                         set_has_more,
                                                                                                         zset_cursor,
                                                                                                         zset_total,
                                                                                                         zset_has_more,
                                                                                                     ).await {
                                                                                                        tracing::error!("{error}");
                                                                                                    } else {
                                                                                                        on_refresh.call(());
                                                                                                    }
                                                                                                }
                                                                                                Err(error) => {
                                                                                                    set_status_message.set(format!("删除失败：{error}"));
                                                                                                    set_status_error.set(true);
                                                                                                }
                                                                                            }
                                                                                            set_action.set(None);
                                                                                        });
                                                                                    }
                                                                                },

                                                                                IconTrash { size: Some(15) }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if set_loading_more() {
                                                div {
                                                    padding: "12px",
                                                    text_align: "center",
                                                    color: COLOR_TEXT_SECONDARY,
                                                    font_size: "13px",

                                                    "加载中..."
                                                }
                                            }

                                            if set_has_more() && !set_loading_more() {
                                                div {
                                                    padding: "8px",
                                                    text_align: "center",
                                                    color: COLOR_TEXT_SUBTLE,
                                                    font_size: "12px",

                                                    "向下滚动加载更多..."
                                                }
                                            }
                                        }
                                    }
                                    KeyType::ZSet => {
                                        rsx! {
                                            div {
                                                display: "flex",
                                                flex_direction: "column",
                                                height: "100%",
                                                min_height: "0",

                                                display: "flex",
                                                justify_content: "space_between",
                                                align_items: "center",
                                                gap: "12px",
                                                margin_bottom: "12px",
                                                flex_wrap: "wrap",

                                                div {
                                                    display: "flex",
                                                    gap: "8px",
                                                    align_items: "center",
                                                    flex_wrap: "wrap",

                                                    input {
                                                        width: "160px",
                                                        padding: "8px 10px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "6px",
                                                        color: COLOR_TEXT,
                                                        value: "{zset_search}",
                                                        placeholder: "搜索成员",
                                                        oninput: {
                                                            let pool = connection_pool.clone();
                                                            let key = display_key.clone();
                                                            move |event| {
                                                                let value = event.value();
                                                                let was_empty = zset_search().is_empty();
                                                                zset_search.set(value.clone());
                                                                
                                                                if value.is_empty() && !was_empty {
                                                                    let pool = pool.clone();
                                                                    let key = key.clone();
                                                                    spawn(async move {
                                                                        if let Err(e) = load_key_data(
                                                                            pool,
                                                                            key,
                                                                            key_info,
                                                                            string_value,
                                                                            hash_value,
                                                                            list_value,
                                                                            set_value,
                                                                            zset_value,
                                                                            stream_value,
                                                                            is_binary,
                                                                            binary_format,
                                                                            serialization_data,
                binary_bytes,
                                                                            bitmap_info,
                                                                            loading,
                                                                            hash_cursor,
                                                                            hash_total,
                                                                            hash_has_more,
                                                                            list_has_more,
                                                                            list_total,
                                                                            set_cursor,
                                                                            set_total,
                                                                            set_has_more,
                                                                            zset_cursor,
                                                                            zset_total,
                                                                            zset_has_more,
                                                                        ).await {
                                                                            tracing::error!("重新加载 zset 数据失败: {}", e);
                                                                        }
                                                                    });
                                                                }
                                                            }
                                                        },
                                                    }

                                                    if zset_total() > PAGE_SIZE {
                                                        button {
                                                            padding: "6px 10px",
                                                            background: if zset_search().len() >= 2 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                            color: if zset_search().len() >= 2 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT_SECONDARY },
                                                            border: "1px solid {COLOR_BORDER}",
                                                            border_radius: "6px",
                                                            cursor: if zset_search().len() >= 2 { "pointer" } else { "not-allowed" },
                                                            font_size: "12px",
                                                            disabled: zset_search().len() < 2 || zset_loading_more(),
                                                            onclick: {
                                                                let pool = connection_pool.clone();
                                                                let key = display_key.clone();
                                                                move |_| {
                                                                    let pool = pool.clone();
                                                                    let key = key.clone();
                                                                    let pattern = zset_search();
                                                                    spawn(async move {
                                                                        search_zset_server(
                                                                            pool,
                                                                            key,
                                                                            pattern,
                                                                            zset_value,
                                                                            zset_cursor,
                                                                            zset_has_more,
                                                                            zset_loading_more,
                                                                        ).await;
                                                                    });
                                                                }
                                                            },

                                                            if zset_loading_more() { "搜索中..." } else { "服务端搜索" }
                                                        }
                                                    }

                                                    input {
                                                        width: "80px",
                                                        padding: "8px 10px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "6px",
                                                        color: COLOR_TEXT,
                                                        value: "{new_zset_score}",
                                                        placeholder: "Score",
                                                        oninput: move |event| new_zset_score.set(event.value()),
                                                    }

                                                    input {
                                                        width: "200px",
                                                        padding: "8px 10px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "6px",
                                                        color: COLOR_TEXT,
                                                        value: "{new_zset_member}",
                                                        placeholder: "输入新成员",
                                                        oninput: move |event| new_zset_member.set(event.value()),
                                                    }

                                                    button {
                                                        padding: "8px 12px",
                                                        background: "#38a169",
                                                        color: COLOR_TEXT_CONTRAST,
                                                        border: "none",
                                                        border_radius: "6px",
                                                        cursor: "pointer",
                                                        disabled: zset_action().is_some(),
                                                        onclick: {
                                                            let pool = connection_pool.clone();
                                                            let key = display_key.clone();
                                                            move |_| {
                                                                let pool = pool.clone();
                                                                let key = key.clone();
                                                                let member = new_zset_member();
                                                                let score_str = new_zset_score();
                                                                spawn(async move {
                                                                    if member.trim().is_empty() {
                                                                        zset_status_message.set("成员不能为空".to_string());
                                                                        zset_status_error.set(true);
                                                                        return;
                                                                    }

                                                                    let score: f64 = match score_str.parse() {
                                                                        Ok(s) => s,
                                                                        Err(_) => {
                                                                            zset_status_message.set("Score 必须是有效数字".to_string());
                                                                            zset_status_error.set(true);
                                                                            return;
                                                                        }
                                                                    };

                                                                    zset_action.set(Some("add".to_string()));
                                                                    zset_status_message.set(String::new());
                                                                    zset_status_error.set(false);

                                                                    match pool.zset_add(&key, &member, score).await {
                                                                        Ok(_) => {
                                                                            new_zset_member.set(String::new());
                                                                            new_zset_score.set(String::new());
                                                                            zset_status_message.set("添加成功".to_string());
                                                                            zset_status_error.set(false);
                                                                            if let Err(error) = load_key_data(
                                                                                pool.clone(),
                                                                                key.clone(),
                                                                                key_info,
                                                                                string_value,
                                                                                hash_value,
                                                                                list_value,
                                                                                set_value,
                                                                                zset_value,
                                                                                stream_value,
                                                                                is_binary,
                                                                                binary_format,
                                                                                serialization_data,
                binary_bytes,
                                                                                bitmap_info,
                                                                                loading,
                                                                                hash_cursor,
                                                                                hash_total,
                                                                                hash_has_more,
                                                                                list_has_more,
                                                                                list_total,
                                                                                set_cursor,
                                                                                set_total,
                                                                                set_has_more,
                                                                                zset_cursor,
                                                                                zset_total,
                                                                                zset_has_more,
                                                                            ).await {
                                                                                tracing::error!("{error}");
                                                                            } else {
                                                                                on_refresh.call(());
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            zset_status_message.set(format!("添加失败：{error}"));
                                                                            zset_status_error.set(true);
                                                                        }
                                                                    }
                                                                    zset_action.set(None);
                                                                });
                                                            }
                                                        },

                                                        if zset_action().as_deref() == Some("add") { "添加中..." } else { "添加成员" }
                                                    }

                                                    div {
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "13px",

                                                        "ZSet Members ({zset_val.len()}/{zset_total()})"
                                                    }
                                                }

                                                button {
                                                    margin_left: "auto",
                                                    flex_shrink: "0",
                                                    padding: "6px 10px",
                                                    background: "rgba(47, 133, 90, 0.16)",
                                                    color: COLOR_SUCCESS,
                                                    border: "1px solid rgba(104, 211, 145, 0.28)",
                                                    border_radius: "6px",
                                                    cursor: "pointer",
                                                    display: "flex",
                                                    align_items: "center",
                                                    gap: "4px",
                                                    title: "复制",
                                                    onclick: {
                                                        let zset = zset_val.clone();
                                                        move |_| {
                                                            let json = serde_json::to_string_pretty(&zset).unwrap_or_default();
                                                            match copy_value_to_clipboard(&json) {
                                                                Ok(_) => {
                                                                    toast_manager.write().success("复制成功");
                                                                }
                                                                Err(error) => {
                                                                    toast_manager.write().error(&format!("复制失败：{error}"));
                                                                }
                                                            }
                                                        }
                                                    },

                                                    IconCopy { size: Some(14) }
                                                    "复制"
                                                }
                                            }

                                            if !zset_status_message.read().is_empty() {
                                                div {
                                                    margin_bottom: "12px",
                                                    padding: "8px 12px",
                                                    background: if zset_status_error() { STATUS_ERROR_BG } else { STATUS_SUCCESS_BG },
                                                    border_radius: "6px",
                                                    color: if zset_status_error() { COLOR_ERROR } else { COLOR_SUCCESS },
                                                    font_size: "13px",

                                                    "{zset_status_message}"
                                                }
                                            }

                                            if zset_val.len() > LARGE_KEY_THRESHOLD {
                                                LargeKeyWarning {
                                                    key_type: "ZSet".to_string(),
                                                    size: zset_val.len(),
                                                    threshold: LARGE_KEY_THRESHOLD,
                                                }
                                            }

                                            div {
                                                flex: "1",
                                                min_height: "0",
                                                overflow_x: "auto",
                                                overflow_y: "auto",
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "8px",
                                                background: COLOR_BG_SECONDARY,
                                                onscroll: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    move |e| {
                                                        let scroll_top = e.data().scroll_top() as i32;
                                                        let scroll_height = e.data().scroll_height() as i32;
                                                        let client_height = e.data().client_height() as i32;
                                                        
                                                        if zset_has_more() && !zset_loading_more() && scroll_height - scroll_top - client_height < 200 {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let cursor = zset_cursor();
                                                            spawn(async move {
                                                                load_more_zset(
                                                                    pool,
                                                                    key,
                                                                    zset_value,
                                                                    cursor,
                                                                    zset_cursor,
                                                                    zset_has_more,
                                                                    zset_loading_more,
                                                                ).await;
                                                            });
                                                        }
                                                    }
                                                },

                                                table {
                                                    width: "100%",
                                                    border_collapse: "collapse",

                                                    thead {
                                                        tr {
                                                            background: COLOR_BG_TERTIARY,
                                                            border_bottom: "1px solid {COLOR_BORDER}",

                                                            th {
                                                                width: "72px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "#"
                                                            }

                                                            th {
                                                                width: "100px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Score"
                                                            }

                                                            th {
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Member"
                                                            }

                                                            th {
                                                                width: "100px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Action"
                                                            }
                                                        }
                                                    }

                                                    tbody {
                                                        for (idx, (member, score)) in filtered_zset_members.iter().enumerate() {
                                                            if editing_zset_member() == Some(member.clone()) {
                                                                tr {
                                                                    background: ROW_EDIT_BG,
                                                                    border_bottom: "1px solid {COLOR_BORDER}",

                                                                    td {
                                                                        padding: "12px",
                                                                        color: COLOR_TEXT_SECONDARY,

                                                                        "{idx + 1}"
                                                                    }

                                                                    td {
                                                                        padding: "12px",

                                                                        input {
                                                                            width: "80px",
                                                                            padding: "8px 10px",
                                                                            background: COLOR_BG,
                                                                            border: "1px solid {COLOR_BORDER}",
                                                                            border_radius: "6px",
                                                                            color: COLOR_TEXT,
                                                                            value: "{editing_zset_score}",
                                                                            oninput: move |event| editing_zset_score.set(event.value()),
                                                                        }
                                                                    }

                                                                    td {
                                                                        padding: "12px",
                                                                        color: COLOR_ACCENT,

                                                                        "{member}"
                                                                    }

                                                                    td {
                                                                        padding: "12px",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "8px",

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: "#38a169",
                                                                                color: COLOR_TEXT_CONTRAST,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: zset_action().is_some(),
                                                                                onclick: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    let member = member.clone();
                                                                                    move |_| {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        let member = member.clone();
                                                                                        let score_str = editing_zset_score();
                                                                                        spawn(async move {
                                                                                            let score: f64 = match score_str.parse() {
                                                                                                Ok(s) => s,
                                                                                                Err(_) => {
                                                                                                    zset_status_message.set("Score 必须是有效数字".to_string());
                                                                                                    zset_status_error.set(true);
                                                                                                    return;
                                                                                                }
                                                                                            };

                                                                                            zset_action.set(Some("update".to_string()));
                                                                                            match pool.zset_add(&key, &member, score).await {
                                                                                                Ok(_) => {
                                                                                                    editing_zset_member.set(None);
                                                                                                    zset_status_message.set("修改成功".to_string());
                                                                                                    zset_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                         pool.clone(),
                                                                                                         key.clone(),
                                                                                                         key_info,
                                                                                                         string_value,
                                                                                                         hash_value,
                                                                                                         list_value,
                                                                                                         set_value,
                                                                                                         zset_value,
                                                                                                         stream_value,
                                                                                                         is_binary,
                                                                                                         binary_format,
                                                                                                         serialization_data,
                binary_bytes,
                                                                                                         bitmap_info,
                                                                                                         loading,
                                                                                                         hash_cursor,
                                                                                                         hash_total,
                                                                                                         hash_has_more,
                                                                                                         list_has_more,
                                                                                                         list_total,
                                                                                                         set_cursor,
                                                                                                         set_total,
                                                                                                         set_has_more,
                                                                                                         zset_cursor,
                                                                                                         zset_total,
                                                                                                         zset_has_more,
                                                                                                     ).await {
                                                                                                        tracing::error!("{error}");
                                                                                                    } else {
                                                                                                        on_refresh.call(());
                                                                                                    }
                                                                                                }
                                                                                                Err(error) => {
                                                                                                    zset_status_message.set(format!("修改失败：{error}"));
                                                                                                    zset_status_error.set(true);
                                                                                                }
                                                                                            }
                                                                                            zset_action.set(None);
                                                                                        });
                                                                                    }
                                                                                },

                                                                                "保存"
                                                                            }

                                                                            button {
                                                                                padding: "6px 10px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                color: COLOR_TEXT,
                                                                                border: "none",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                onclick: move |_| {
                                                                                    editing_zset_member.set(None);
                                                                                },

                                                                                "取消"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                tr {
                                                                    key: "{member}",
                                                                    border_bottom: "1px solid {COLOR_BORDER}",
                                                                    background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },

                                                                    td {
                                                                        padding: "10px 12px",
                                                                        color: COLOR_TEXT_SECONDARY,

                                                                        "{idx + 1}"
                                                                    }

                                                                    td {
                                                                        padding: "10px 12px",
                                                                        color: COLOR_WARNING,
                                                                        font_family: "Consolas, monospace",
                                                                        font_size: "13px",

                                                                        "{score}"
                                                                    }

                                                                    td {
                                                                        padding: "10px 12px",
                                                                        color: COLOR_TEXT,
                                                                        font_family: "Consolas, monospace",
                                                                        font_size: "13px",
                                                                        word_break: "break-all",

                                                                        "{member}"
                                                                    }

                                                                    td {
                                                                        padding: "10px 12px",

                                                                        div {
                                                                            display: "flex",
                                                                            gap: "6px",

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(47, 133, 90, 0.16)",
                                                                                color: "#68d391",
                                                                                border: "1px solid rgba(104, 211, 145, 0.28)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                title: "复制",
                                                                                onclick: {
                                                                                    let member = member.clone();
                                                                                    move |_| {
                                                                                        match copy_value_to_clipboard(&member) {
                                                                                            Ok(_) => {
                                                                                                toast_manager.write().success("复制成功");
                                                                                            }
                                                                                            Err(error) => {
                                                                                                toast_manager.write().error(&format!("复制失败：{error}"));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                },

                                                                                IconCopy { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(49, 130, 206, 0.18)",
                                                                                color: "#63b3ed",
                                                                                border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                title: "修改 Score",
                                                                                onclick: {
                                                                                    let member = member.clone();
                                                                                    let score = *score;
                                                                                    move |_| {
                                                                                        editing_zset_member.set(Some(member.clone()));
                                                                                        editing_zset_score.set(score.to_string());
                                                                                    }
                                                                                },

                                                                                IconEdit { size: Some(15) }
                                                                            }

                                                                            button {
                                                                                width: "32px",
                                                                                height: "32px",
                                                                                display: "flex",
                                                                                align_items: "center",
                                                                                justify_content: "center",
                                                                                background: "rgba(197, 48, 48, 0.18)",
                                                                                color: "#f87171",
                                                                                border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                                border_radius: "6px",
                                                                                cursor: "pointer",
                                                                                disabled: zset_action().is_some(),
                                                                                title: "删除",
                                                                                onclick: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    let member = member.clone();
                                                                                    move |_| {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        let member = member.clone();
                                                                                        spawn(async move {
                                                                                            zset_action.set(Some(format!("delete:{}", member)));
                                                                                            match pool.zset_remove(&key, &member).await {
                                                                                                Ok(_) => {
                                                                                                    zset_status_message.set("删除成功".to_string());
                                                                                                    zset_status_error.set(false);
if let Err(error) = load_key_data(
                                                                                                         pool.clone(),
                                                                                                         key.clone(),
                                                                                                         key_info,
                                                                                                         string_value,
                                                                                                         hash_value,
                                                                                                         list_value,
                                                                                                         set_value,
                                                                                                         zset_value,
                                                                                                         stream_value,
                                                                                                         is_binary,
                                                                                                         binary_format,
                                                                                                         serialization_data,
                binary_bytes,
                                                                                                         bitmap_info,
                                                                                                         loading,
                                                                                                         hash_cursor,
                                                                                                         hash_total,
                                                                                                         hash_has_more,
                                                                                                         list_has_more,
                                                                                                         list_total,
                                                                                                         set_cursor,
                                                                                                         set_total,
                                                                                                         set_has_more,
                                                                                                         zset_cursor,
                                                                                                         zset_total,
                                                                                                         zset_has_more,
                                                                                                     ).await {
                                                                                                        tracing::error!("{error}");
                                                                                                    } else {
                                                                                                        on_refresh.call(());
                                                                                                    }
                                                                                                }
                                                                                                Err(error) => {
                                                                                                    zset_status_message.set(format!("删除失败：{error}"));
                                                                                                    zset_status_error.set(true);
                                                                                                }
                                                                                            }
                                                                                            zset_action.set(None);
                                                                                        });
                                                                                    }
                                                                                },

                                                                                IconTrash { size: Some(15) }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if zset_loading_more() {
                                                div {
                                                    padding: "12px",
                                                    text_align: "center",
                                                    color: COLOR_TEXT_SECONDARY,
                                                    font_size: "13px",

                                                    "加载中..."
                                                }
                                            }

                                            if zset_has_more() && !zset_loading_more() {
                                                div {
                                                    padding: "8px",
                                                    text_align: "center",
                                                    color: COLOR_TEXT_SUBTLE,
                                                    font_size: "12px",

                                                    "向下滚动加载更多..."
                                                }
                                            }
                                        }
                                    }
                                    KeyType::Stream => {
                                        let stream_val = stream_value.read().clone();
                                        let filtered_entries: Vec<_> = if stream_search.read().is_empty() {
                                            stream_val.clone()
                                        } else {
                                            stream_val.iter().filter(|(id, fields)| {
                                                id.to_lowercase().contains(&stream_search.read().to_lowercase()) ||
                                                fields.iter().any(|(k, v)| {
                                                    k.to_lowercase().contains(&stream_search.read().to_lowercase()) ||
                                                    v.to_lowercase().contains(&stream_search.read().to_lowercase())
                                                })
                                            }).cloned().collect()
                                        };

                                        rsx! {
                                            div {
                                                display: "flex",
                                                flex_direction: "column",
                                                height: "100%",
                                                min_height: "0",

                                                display: "flex",
                                                justify_content: "space_between",
                                                align_items: "center",
                                                gap: "12px",
                                                margin_bottom: "12px",
                                                flex_wrap: "wrap",

                                                div {
                                                    display: "flex",
                                                    gap: "8px",
                                                    align_items: "center",
                                                    flex_wrap: "wrap",

                                                    input {
                                                        width: "200px",
                                                        padding: "8px 10px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        border_radius: "6px",
                                                        color: COLOR_TEXT,
                                                        value: "{stream_search}",
                                                        placeholder: "搜索 ID 或字段",
                                                        oninput: move |event| stream_search.set(event.value()),
                                                    }

                                                    if stream_search.read().is_empty() {
                                                        div {
                                                            color: COLOR_TEXT_SECONDARY,
                                                            font_size: "13px",

                                                            "Stream Entries ({stream_val.len()})"
                                                        }
                                                    } else {
                                                        div {
                                                            color: COLOR_TEXT_SECONDARY,
                                                            font_size: "13px",

                                                            "Stream Entries ({filtered_entries.len()}/{stream_val.len()})"
                                                        }
                                                    }
                                                }

                                                button {
                                                    margin_left: "auto",
                                                    flex_shrink: "0",
                                                    padding: "6px 10px",
                                                    background: "rgba(47, 133, 90, 0.16)",
                                                    color: COLOR_SUCCESS,
                                                    border: "1px solid rgba(104, 211, 145, 0.28)",
                                                    border_radius: "6px",
                                                    cursor: "pointer",
                                                    display: "flex",
                                                    align_items: "center",
                                                    gap: "4px",
                                                    title: "复制",
                                                    onclick: {
                                                        let stream = stream_val.clone();
                                                        move |_| {
                                                            let json = serde_json::to_string_pretty(&stream).unwrap_or_default();
                                                            match copy_value_to_clipboard(&json) {
                                                                Ok(_) => {
                                                                    toast_manager.write().success("复制成功");
                                                                }
                                                                Err(error) => {
                                                                    toast_manager.write().error(&format!("复制失败：{error}"));
                                                                }
                                                            }
                                                        }
                                                    },

                                                    IconCopy { size: Some(14) }
                                                    "复制"
                                                }
                                            }

                                            if !stream_status_message.read().is_empty() {
                                                div {
                                                    margin_bottom: "12px",
                                                    padding: "8px 12px",
                                                    background: if stream_status_error() { STATUS_ERROR_BG } else { STATUS_SUCCESS_BG },
                                                    border_radius: "6px",
                                                    color: if stream_status_error() { COLOR_ERROR } else { COLOR_SUCCESS },
                                                    font_size: "13px",

                                                    "{stream_status_message}"
                                                }
                                            }

                                            div {
                                                flex: "1",
                                                min_height: "0",
                                                overflow_x: "auto",
                                                overflow_y: "auto",
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "8px",
                                                background: COLOR_BG_SECONDARY,
                                                onscroll: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    move |e| {
                                                        let scroll_top = e.data().scroll_top() as i32;
                                                        let scroll_height = e.data().scroll_height() as i32;
                                                        let client_height = e.data().client_height() as i32;
                                                        
                                                        if hash_has_more() && !hash_loading_more() && scroll_height - scroll_top - client_height < 200 {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let cursor = hash_cursor();
                                                            let total = hash_total();
                                                            spawn(async move {
                                                                load_more_hash(
                                                                    pool,
                                                                    key,
                                                                    hash_value,
                                                                    cursor,
                                                                    hash_cursor,
                                                                    hash_has_more,
                                                                    hash_loading_more,
                                                                    hash_total,
                                                                ).await;
                                                            });
                                                        }
                                                    }
                                                },

                                                table {
                                                    width: "100%",
                                                    border_collapse: "collapse",

                                                    thead {
                                                        tr {
                                                            background: COLOR_BG_TERTIARY,
                                                            border_bottom: "1px solid {COLOR_BORDER}",

                                                            th {
                                                                width: "200px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Entry ID"
                                                            }

                                                            th {
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "left",

                                                                "Fields"
                                                            }

                                                            th {
    width: "30px",
                                                                padding: "12px",
                                                                color: COLOR_TEXT_SECONDARY,
                                                                font_size: "12px",
                                                                font_weight: "600",
                                                                text_align: "center",

                                                                ""
                                                            }
                                                        }
                                                    }

                                                    tbody {
                                                        for (entry_id, fields) in filtered_entries.iter() {
                                                            tr {
                                                                border_bottom: "1px solid {COLOR_BORDER}",

                                                                td {
                                                                    padding: "10px 12px",
                                                                    color: COLOR_TEXT,
                                                                    font_size: "12px",
                                                                    font_family: "Consolas, monospace",

                                                                    "{entry_id}"
                                                                }

                                                                td {
                                                                    padding: "10px 12px",

                                                                    div {
                                                                        display: "flex",
                                                                        flex_wrap: "wrap",
                                                                        gap: "8px",

                                                                        for (field_key, field_value) in fields.iter() {
                                                                            div {
                                                                                padding: "4px 8px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "4px",
                                                                                font_size: "11px",

                                                                                span {
                                                                                    color: "#818cf8",
                                                                                    font_family: "Consolas, monospace",

                                                                                    "{field_key}:"
                                                                                }
                                                                                span {
                                                                                    color: COLOR_TEXT,
                                                                                    margin_left: "4px",

                                                                                    "{field_value}"
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }

                                                                td {
                                                                    padding: "10px 12px",
                                                                    text_align: "center",

                                                                    button {
                                                                        padding: "4px 8px",
                                                                        background: "transparent",
                                                                        color: COLOR_ERROR,
                                                                        border: "none",
                                                                        border_radius: "4px",
                                                                        cursor: "pointer",
                                                                        opacity: 0.7,
                                                                        onclick: {
                                                                            let entry_id = entry_id.clone();
                                                                            move |_| {
                                                                                deleting_stream_entry.set(Some(entry_id.clone()));
                                                                            }
                                                                        },

                                                                        IconTrash { size: Some(15) }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if filtered_entries.is_empty() {
                                                            tr {
                                                                td {
                                                                    colspan: 3,
                                                                    padding: "32px",
                                                                    text_align: "center",
                                                                    color: COLOR_TEXT_SECONDARY,

                                                                    if stream_search.read().is_empty() {
                                                                        "暂无数据"
                                                                    } else {
                                                                        "未找到匹配的条目"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if let Some(entry_id) = deleting_stream_entry.read().clone() {
                                                div {
                                                    position: "fixed",
                                                    top: "0",
                                                    left: "0",
                                                    right: "0",
                                                    bottom: "0",
                                                    background: "rgba(0, 0, 0, 0.5)",
                                                    display: "flex",
                                                    align_items: "center",
                                                    justify_content: "center",
                                                    z_index: "1000",
                                                    animation: if deleting_stream_entry_exiting() { "backdropFadeOut 0.2s ease-out forwards" } else { "backdropFadeIn 0.2s ease-out" },

                                                    style {
                                                        r#"
                                                        @keyframes backdropFadeIn {{
                                                            from {{ opacity: 0; }}
                                                            to {{ opacity: 1; }}
                                                        }}
                                                        @keyframes backdropFadeOut {{
                                                            from {{ opacity: 1; }}
                                                            to {{ opacity: 0; }}
                                                        }}
                                                        @keyframes modalFadeIn {{
                                                            from {{ opacity: 0; transform: scale(0.95); }}
                                                            to {{ opacity: 1; transform: scale(1); }}
                                                        }}
                                                        @keyframes modalFadeOut {{
                                                            from {{ opacity: 1; transform: scale(1); }}
                                                            to {{ opacity: 0; transform: scale(0.95); }}
                                                        }}
                                                        "#
                                                    }

                                                    div {
                                                        padding: "24px",
                                                        background: COLOR_BG_SECONDARY,
                                                        border_radius: "12px",
                                                        max_width: "400px",
                                                        width: "90%",
                                                        animation: if deleting_stream_entry_exiting() { "modalFadeOut 0.2s ease-out forwards" } else { "modalFadeIn 0.2s ease-out" },

                                                        h3 {
                                                            margin_bottom: "16px",
                                                            color: COLOR_TEXT,
                                                            font_size: "16px",
                                                            font_weight: "600",

                                                            "删除 Stream Entry"
                                                        }

                                                        p {
                                                            margin_bottom: "24px",
                                                            color: COLOR_TEXT_SECONDARY,
                                                            font_size: "14px",

                                                            "确定要删除 entry \"{entry_id}\" 吗？"
                                                        }

                                                        div {
                                                            display: "flex",
                                                            gap: "12px",
                                                            justify_content: "flex_end",

                                                            button {
                                                                padding: "8px 16px",
                                                                background: COLOR_BG_TERTIARY,
                                                                color: COLOR_TEXT,
                                                                border: "1px solid {COLOR_BORDER}",
                                                                border_radius: "6px",
                                                                cursor: "pointer",
                                                                onclick: {
                                                                    let mut deleting_stream_entry = deleting_stream_entry.clone();
                                                                    let mut deleting_stream_entry_exiting = deleting_stream_entry_exiting.clone();
                                                                    move |_| {
                                                                        deleting_stream_entry_exiting.set(true);
                                                                        let mut dse = deleting_stream_entry.clone();
                                                                        let mut dsee = deleting_stream_entry_exiting.clone();
                                                                        spawn(async move {
                                                                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                                                            dse.set(None);
                                                                            dsee.set(false);
                                                                        });
                                                                    }
                                                                },

                                                                "取消"
                                                            }

                                                            button {
                                                                padding: "8px 16px",
                                                                background: COLOR_ERROR,
                                                                color: COLOR_TEXT_CONTRAST,
                                                                border: "none",
                                                                border_radius: "6px",
                                                                cursor: "pointer",
                                                                onclick: {
                                                                    let pool = connection_pool.clone();
                                                                    let key = display_key.clone();
                                                                    let entry_id = entry_id.clone();
                                                                    let mut deleting_stream_entry = deleting_stream_entry.clone();
                                                                    let mut deleting_stream_entry_exiting = deleting_stream_entry_exiting.clone();
                                                                    move |_| {
                                                                        let pool = pool.clone();
                                                                        let key = key.clone();
                                                                        let entry_id = entry_id.clone();
                                                                        spawn(async move {
                                                                            match pool.stream_delete(&key, &entry_id).await {
                                                                                Ok(true) => {
                                                                                    stream_status_message.set("删除成功".to_string());
                                                                                    stream_status_error.set(false);
                                                                                    deleting_stream_entry_exiting.set(true);
                                                                                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                                                                    deleting_stream_entry.set(None);
                                                                                    deleting_stream_entry_exiting.set(false);
if let Err(error) = load_key_data(
                                                                                         pool.clone(),
                                                                                         key.clone(),
                                                                                         key_info,
                                                                                         string_value,
                                                                                         hash_value,
                                                                                         list_value,
                                                                                         set_value,
                                                                                         zset_value,
                                                                                         stream_value,
                                                                                         is_binary,
                                                                                         binary_format,
                                                                                         serialization_data,
                binary_bytes,
                                                                                         bitmap_info,
                                                                                         loading,
                                                                                         hash_cursor,
                                                                                         hash_total,
                                                                                         hash_has_more,
                                                                                         list_has_more,
                                                                                         list_total,
                                                                                         set_cursor,
                                                                                         set_total,
                                                                                         set_has_more,
                                                                                         zset_cursor,
                                                                                         zset_total,
                                                                                         zset_has_more,
                                                                                     ).await {
                                                                                        tracing::error!("{error}");
                                                                                    } else {
                                                                                        on_refresh.call(());
                                                                                    }
                                                                                }
                                                                                Ok(false) => {
                                                                                    stream_status_message.set("Entry 不存在".to_string());
                                                                                    stream_status_error.set(true);
                                                                                }
                                                                                Err(error) => {
                                                                                    stream_status_message.set(format!("删除失败：{error}"));
                                                                                    stream_status_error.set(true);
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                },

                                                                "删除"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                                    rsx! {
                                                        div {
                                                            color: COLOR_TEXT_SECONDARY,

                                                            "暂不支持该类型的编辑"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            }
                        } else {
                            div {
                                height: "100%",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                color: COLOR_TEXT_SECONDARY,
                                text_align: "center",
                                border: "1px solid {COLOR_BORDER}",
                                border_radius: "12px",
                                background: COLOR_BG_SECONDARY,

                                "未能加载 Key 数据"
                            }
                        }
                    }
                }
            }
}

#[component]
pub fn BitmapViewer(
    info: crate::redis::BitmapInfo,
    pool: ConnectionPool,
    redis_key: String,
    on_update: EventHandler<()>,
) -> Element {
    let mut editing_offset = use_signal(String::new);
    let mut editing_value = use_signal(|| "1".to_string());

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "16px",

            div {
                display: "flex",
                gap: "16px",
                flex_wrap: "wrap",

                div {
                    padding: "8px 12px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",

                        "总字节数: "
                    }
                    span {
                        color: COLOR_TEXT,
                        font_size: "12px",
                        font_weight: "600",

                        "{info.total_bytes}"
                    }
                }

                div {
                    padding: "8px 12px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",

                        "总位数: "
                    }
                    span {
                        color: COLOR_TEXT,
                        font_size: "12px",
                        font_weight: "600",

                        "{info.total_bits}"
                    }
                }

                div {
                    padding: "8px 12px",
                    background: "rgba(47, 133, 90, 0.16)",
                    border_radius: "6px",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",

                        "已设置位: "
                    }
                    span {
                        color: COLOR_SUCCESS,
                        font_size: "12px",
                        font_weight: "600",

                        "{info.set_bits_count}"
                    }
                }
            }

            div {
                span {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",
                    font_weight: "600",
                    margin_bottom: "8px",
                    display: "block",

                    "已设置的位 (offset):"
                }

                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "6px",
                    max_height: "120px",
                    overflow_y: "auto",
                    padding: "8px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    for offset in info.set_bits.iter().take(200) {
                        span {
                            padding: "2px 8px",
                            background: "rgba(99, 102, 241, 0.20)",
                            color: "#818cf8",
                            border_radius: "4px",
                            font_size: "11px",
                            font_family: "Consolas, monospace",

                            "{offset}"
                        }
                    }
                    if info.set_bits.len() > 200 {
                        span {
                            padding: "2px 8px",
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",

                            "... 还有 {info.set_bits.len() - 200} 个"
                        }
                    }
                }
            }

            div {
                span {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",
                    font_weight: "600",
                    margin_bottom: "8px",
                    display: "block",

                    "二进制视图:"
                }

                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "4px",
                    font_family: "Consolas, monospace",
                    font_size: "11px",
                    max_height: "200px",
                    overflow_y: "auto",
                    padding: "8px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    for (byte_idx, byte) in info.raw_bytes.iter().enumerate().take(64) {
                        div {
                            display: "flex",
                            flex_direction: "column",
                            align_items: "center",
                            gap: "2px",

                            div {
                                display: "flex",
                                gap: "1px",

                                for bit_idx in 0..8 {
                                    { let bit_val = (*byte >> (7 - bit_idx)) & 1; rsx! {
                                        div {
                                            width: "12px",
                                            height: "12px",
                                            background: if bit_val == 1 { "#22c55e" } else { COLOR_BG },
                                            border_radius: "2px",
                                        }
                                    }}
                                }
                            }

                            span {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "9px",

                                "{byte_idx}"
                            }
                        }
                    }
                    if info.raw_bytes.len() > 64 {
                        span {
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",

                            "... 共 {info.raw_bytes.len()} 字节"
                        }
                    }
                }
            }

            div {
                span {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",
                    font_weight: "600",
                    margin_bottom: "8px",
                    display: "block",

                    "设置/修改位:"
                }

                div {
                    display: "flex",
                    gap: "8px",
                    align_items: "center",

                    input {
                        width: "100px",
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        placeholder: "Offset",
                        value: "{editing_offset}",
                        oninput: move |e| editing_offset.set(e.value()),
                    }

                    select {
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        value: "{editing_value}",
                        onchange: move |e| editing_value.set(e.value()),

                        option {
                            value: "0",

                            "设为 0"
                        }
                        option {
                            value: "1",

                            "设为 1"
                        }
                    }

                    button {
                        padding: "6px 12px",
                        background: COLOR_PRIMARY,
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: {
                            let pool = pool.clone();
                            let redis_key = redis_key.clone();
                            move |_| {
                                let offset_str = editing_offset();
                                let value_str = editing_value();
                                if let Ok(offset) = offset_str.parse::<u64>() {
                                    let value = value_str == "1";
                                    let pool = pool.clone();
                                    let redis_key = redis_key.clone();
                                    spawn(async move {
                                        if pool.set_bit(&redis_key, offset, value).await.is_ok() {
                                            on_update.call(());
                                        }
                                    });
                                }
                            }
                        },

                        "应用"
                    }
                }
            }
        }
    }
}

#[component]
fn ProtobufViewer(data: Vec<u8>) -> Element {
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
                                    Err(e) => {
                                        import_error.set(Some(e));
                                    }
                                }
                            }
                        });
                    },

                    "导入 .proto 文件"
                }

                if has_schema {
                    select {
                        padding: "6px 12px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT,
                        border: format!("1px solid {}", COLOR_BORDER),
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",

                        onchange: move |e| {
                            selected_message.set(e.value());
                        },

                        option {
                            value: "",
                            "Raw 解析"
                        }

                        for msg in messages.iter() {
                            option {
                                value: msg.full_name.clone(),
                                selected: selected_message() == msg.full_name,

                                "{msg.name}"
                            }
                        }
                    }

                    button {
                        padding: "4px 8px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT_SECONDARY,
                        border: format!("1px solid {}", COLOR_BORDER),
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

            if let Some(ref err) = import_error() {
                div {
                    padding: "8px 12px",
                    background: COLOR_ERROR_BG,
                    border_radius: "4px",
                    color: COLOR_ERROR,
                    font_size: "12px",

                    "导入错误: {err}"
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

            match json_result {
                Some(json) => rsx! {
                    JsonViewer {
                        value: serde_json::to_string_pretty(&json).unwrap_or_default(),
                        editable: false,
                        on_change: move |_| {},
                    }
                },
                None => rsx! {
                    div {
                        padding: "16px",
                        background: COLOR_ERROR_BG,
                        border_radius: "8px",
                        color: COLOR_ERROR,

                        "Protobuf 解析失败"
                    }
                },
            }
        }
    }
}
