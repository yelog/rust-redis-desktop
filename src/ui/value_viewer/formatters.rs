use crate::redis::KeyType;
use crate::serialization::{
    detect_serialization_format, is_java_serialization, is_protobuf_data, SerializationFormat,
};
use crate::ui::copy_text_to_clipboard;
use std::collections::HashMap;

pub(super) fn base64_decode(data_uri: &str) -> Vec<u8> {
    use base64::{engine::general_purpose, Engine as _};
    if let Some(base64_data) = data_uri.strip_prefix("data:") {
        if let Some(start) = base64_data.find(";base64,") {
            let base64_str = &base64_data[start + 8..];
            return general_purpose::STANDARD
                .decode(base64_str)
                .unwrap_or_default();
        }
    }
    Vec::new()
}

pub(super) fn is_binary_data(data: &[u8]) -> bool {
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

pub(super) fn format_bytes(data: &[u8], format: super::BinaryFormat) -> String {
    match format {
        super::BinaryFormat::Hex => data
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" "),
        super::BinaryFormat::Base64 => {
            use base64::{engine::general_purpose, Engine as _};
            general_purpose::STANDARD.encode(data)
        }
        super::BinaryFormat::JavaSerialized => {
            if is_java_serialization(data) {
                format!(
                    "Java 序列化对象 ({} 字节)\n\n请切换到 Java 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 Java 序列化数据".to_string()
            }
        }
        super::BinaryFormat::Php => {
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
        super::BinaryFormat::MsgPack => {
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
        super::BinaryFormat::Pickle => {
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
        super::BinaryFormat::Kryo => {
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
        super::BinaryFormat::Bitmap => {
            format!(
                "Bitmap 数据 ({} 字节)\n\n请点击 Bitmap 按钮查看可视化",
                data.len()
            )
        }
        super::BinaryFormat::Image => {
            if let Some(format) = detect_image_format(data) {
                format!("{} 图片 ({} 字节)", format, data.len())
            } else {
                "非图片数据".to_string()
            }
        }
        super::BinaryFormat::Protobuf => {
            if is_protobuf_data(data) {
                format!(
                    "Protobuf 数据 ({} 字节)\n\n请切换到 Protobuf 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 Protobuf 数据".to_string()
            }
        }
        super::BinaryFormat::Bson => {
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
        super::BinaryFormat::Cbor => {
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

pub(super) fn detect_image_format(data: &[u8]) -> Option<&'static str> {
    if data.len() < 3 {
        return None;
    }
    if data.len() >= 4 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return Some("PNG");
    }
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Some("JPEG");
    }
    if data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46 {
        return Some("GIF");
    }
    if data.len() >= 12
        && data[0] == 0x52
        && data[1] == 0x49
        && data[2] == 0x46
        && data[3] == 0x46
        && data[8] == 0x57
        && data[9] == 0x45
        && data[10] == 0x42
        && data[11] == 0x50
    {
        return Some("WEBP");
    }
    if data.len() > 12
        && data[0] == 0x00
        && data[1] == 0x00
        && data[2] == 0x00
        && (data[4..8] == [b'i', b'c', b'o', b'n']
            || data[4..8] == [b'p', b'n', b'g', b' ']
            || data[4..8] == [b'j', b'p', b'g', b' '])
    {
        return Some("ICO");
    }
    None
}

pub(super) fn copy_value_to_clipboard(value: &str) -> Result<(), String> {
    copy_text_to_clipboard(value)
}

pub(super) fn sorted_hash_entries(fields: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut entries: Vec<_> = fields
        .iter()
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

pub(super) fn format_memory_usage(bytes: Option<u64>) -> String {
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

pub(super) fn format_ttl_label(ttl: Option<i64>) -> String {
    match ttl {
        Some(ttl) => format!("{ttl}s"),
        None => "永久".to_string(),
    }
}

pub(super) fn value_metric_label(
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
