use serde_json::Value as JsonValue;
use std::io::{Cursor, Read};

pub fn is_msgpack_serialization(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let first = data[0];

    if !matches!(
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
    ) {
        return false;
    }

    let mut cursor = Cursor::new(data);
    match rmp_serde::from_read::<_, JsonValue>(&mut cursor) {
        Ok(value) => {
            let consumed = cursor.position() as usize;
            if consumed != data.len() {
                return false;
            }

            match &value {
                JsonValue::Object(_) => return true,
                JsonValue::Array(arr) if arr.len() >= 2 => return true,
                _ => {}
            }

            if first >= 0x80 && first <= 0x8F {
                let expected_count = (first - 0x80) as usize;
                return expected_count >= 1;
            }

            if first >= 0xA0 && first <= 0xBF {
                let expected_len = (first - 0xA0) as usize;
                return consumed == expected_len + 1 && expected_len >= 2;
            }

            if first >= 0x90 && first <= 0x9F {
                let expected_count = (first - 0x90) as usize;
                return expected_count >= 2;
            }

            false
        }
        Err(_) => false,
    }
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
    fn test_detect_msgpack_not_false_positive() {
        assert!(!is_msgpack_serialization(&[0x91, 0x21, 0x0a]));
        assert!(!is_msgpack_serialization(&[0x90, 0x00]));
        assert!(!is_msgpack_serialization(&[0xA0]));
        assert!(!is_msgpack_serialization(&[0x80]));
        assert!(!is_msgpack_serialization(&[0xFF, 0xFF]));
    }

    #[test]
    fn test_detect_msgpack_array() {
        assert!(is_msgpack_serialization(&[0x92, 0x01, 0x02]));
        assert!(is_msgpack_serialization(&[0x93, 0x01, 0x02, 0x03]));
    }

    #[test]
    fn test_detect_msgpack_map() {
        assert!(is_msgpack_serialization(&[0x81, 0xA1, b'k', 0x01]));
    }

    #[test]
    fn test_detect_msgpack_string() {
        assert!(is_msgpack_serialization(&[0xA5, b'H', b'e', b'l', b'l', b'o']));
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
    fn test_parse_msgpack_array() {
        let data = [0x92, 0x01, 0x02];
        let result = parse_msgpack_to_json(&data[..]).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_parse_msgpack_string() {
        let data = [0xA5, b'H', b'e', b'l', b'l', b'o'];
        let result = parse_msgpack_to_json(&data[..]).unwrap();
        assert_eq!(result, "\"Hello\"");
    }
}
