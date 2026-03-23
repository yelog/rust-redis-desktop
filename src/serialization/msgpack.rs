use serde_json::Value as JsonValue;
use std::io::Cursor;

pub fn is_msgpack_serialization(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let first = data[0];

    matches!(
        first,
        0x80..=0x8F
        | 0x90..=0x9F
        | 0xA0..=0xBF
        | 0xC0
        | 0xC2..=0xC3
        | 0xC4..=0xC9
        | 0xCA..=0xCF
        | 0xD0..=0xD8
        | 0xD9..=0xDB
        | 0xDC..=0xDD
        | 0xDE..=0xDF
    )
}

pub fn parse_msgpack_to_json(data: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(data);

    let value: JsonValue =
        rmp_serde::from_read(cursor).map_err(|e| format!("MessagePack 解析失败: {}", e))?;

    serde_json::to_string_pretty(&value).map_err(|e| format!("JSON 序列化失败: {}", e))
}

pub fn get_msgpack_type_name(data: &[u8]) -> &'static str {
    if data.is_empty() {
        return "Empty";
    }

    match data[0] {
        0x80..=0x8F | 0xDE..=0xDF => "Map",
        0x90..=0x9F | 0xDC..=0xDD => "Array",
        0xA0..=0xBF | 0xD9..=0xDB => "String",
        0xC0 => "Nil",
        0xC2..=0xC3 => "Boolean",
        0xC4..=0xC9 => "Binary",
        0xCA..=0xCF | 0xD0..=0xD8 => "Number",
        0x00..=0x7F | 0xE0..=0xFF => "Integer",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_msgpack() {
        assert!(is_msgpack_serialization(&[0x80]));
        assert!(is_msgpack_serialization(&[0x90]));
        assert!(is_msgpack_serialization(&[0xA0]));
        assert!(is_msgpack_serialization(&[0xC0]));
        assert!(is_msgpack_serialization(&[0xC2]));
        assert!(is_msgpack_serialization(&[0xCA]));
        assert!(!is_msgpack_serialization(&[0xFF, 0xFF]));
    }

    #[test]
    fn test_parse_msgpack_nil() {
        let data = [0xC0];
        let result = parse_msgpack_to_json(&data[..]).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn test_parse_msgpack_bool() {
        let data = [0xC3];
        let result = parse_msgpack_to_json(&data[..]).unwrap();
        assert_eq!(result, "true");

        let data = [0xC2];
        let result = parse_msgpack_to_json(&data[..]).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_parse_msgpack_int() {
        let data = [0x7F];
        let result = parse_msgpack_to_json(&data[..]).unwrap();
        assert_eq!(result, "127");
    }

    #[test]
    fn test_parse_msgpack_string() {
        let data = [0xA5, b'H', b'e', b'l', b'l', b'o'];
        let result = parse_msgpack_to_json(&data[..]).unwrap();
        assert_eq!(result, "\"Hello\"");
    }
}
