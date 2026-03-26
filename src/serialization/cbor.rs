use ciborium::Value as CborValue;
use serde_json::Value as JsonValue;
use std::io::Cursor;

pub fn is_cbor_serialization(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let first = data[0];

    if matches!(
        first,
        0x00..=0x17
        | 0x20..=0x37
        | 0x40..=0x57
        | 0x60..=0x77
        | 0x80..=0x97
        | 0x9F
        | 0xA0..=0xB7
        | 0xBF
        | 0xC0..=0xD7
        | 0xD8..=0xDB
        | 0xE0..=0xF7
        | 0xF8..=0xFB
    ) {
        let cursor = Cursor::new(data);
        ciborium::from_reader::<CborValue, _>(cursor).is_ok()
    } else {
        false
    }
}

pub fn parse_cbor_to_json(data: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(data);
    let cbor_value: CborValue =
        ciborium::from_reader(cursor).map_err(|e| format!("CBOR 解析失败: {}", e))?;

    let json = cbor_to_json(&cbor_value);
    serde_json::to_string_pretty(&json).map_err(|e| format!("JSON 序列化失败: {}", e))
}

fn cbor_to_json(value: &CborValue) -> JsonValue {
    match value {
        CborValue::Null => JsonValue::Null,
        CborValue::Bool(b) => JsonValue::Bool(*b),
        CborValue::Integer(i) => {
            let i128_val = i128::from(*i);
            if i128_val >= 0 {
                if let Some(n) = serde_json::Number::from_u128(i128_val as u128) {
                    JsonValue::Number(n)
                } else {
                    JsonValue::Null
                }
            } else {
                JsonValue::Number((i128_val as i64).into())
            }
        }
        CborValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        CborValue::Text(s) => JsonValue::String(s.clone()),
        CborValue::Bytes(b) => {
            let hex: String = b.iter().map(|byte| format!("{:02x}", byte)).collect();
            JsonValue::String(format!("0x{}", hex))
        }
        CborValue::Array(arr) => JsonValue::Array(arr.iter().map(cbor_to_json).collect()),
        CborValue::Map(map) => {
            let mut result = serde_json::Map::new();
            for (key, val) in map {
                let key_str = match key {
                    CborValue::Text(s) => s.clone(),
                    CborValue::Integer(i) => i128::from(*i).to_string(),
                    CborValue::Bytes(b) => {
                        let hex: String = b.iter().map(|byte| format!("{:02x}", byte)).collect();
                        format!("0x{}", hex)
                    }
                    _ => format!("{:?}", key),
                };
                result.insert(key_str, cbor_to_json(val));
            }
            JsonValue::Object(result)
        }
        CborValue::Tag(tag, val) => {
            let mut map = serde_json::Map::new();
            map.insert("__tag__".to_string(), JsonValue::Number((*tag).into()));
            map.insert("value".to_string(), cbor_to_json(val));
            JsonValue::Object(map)
        }
        _ => JsonValue::String(format!("<Unknown CBOR value>")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cbor_empty() {
        assert!(!is_cbor_serialization(&[]));
    }

    #[test]
    fn test_detect_cbor_uint() {
        assert!(is_cbor_serialization(&[0x00]));
        assert!(is_cbor_serialization(&[0x0A]));
        assert!(is_cbor_serialization(&[0x17]));
    }

    #[test]
    fn test_detect_cbor_negint() {
        assert!(is_cbor_serialization(&[0x20]));
        assert!(is_cbor_serialization(&[0x37]));
    }

    #[test]
    fn test_detect_cbor_text() {
        assert!(is_cbor_serialization(&[0x61, b'a']));
        assert!(is_cbor_serialization(&[0x64, b't', b'e', b's', b't']));
    }

    #[test]
    fn test_parse_cbor_null() {
        let result = parse_cbor_to_json(&[0xF6]).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn test_parse_cbor_bool() {
        assert_eq!(parse_cbor_to_json(&[0xF4]).unwrap(), "false");
        assert_eq!(parse_cbor_to_json(&[0xF5]).unwrap(), "true");
    }

    #[test]
    fn test_parse_cbor_int() {
        assert_eq!(parse_cbor_to_json(&[0x00]).unwrap(), "0");
        assert_eq!(parse_cbor_to_json(&[0x0A]).unwrap(), "10");
        assert_eq!(parse_cbor_to_json(&[0x20]).unwrap(), "-1");
    }

    #[test]
    fn test_parse_cbor_string() {
        let data = [0x65, b'H', b'e', b'l', b'l', b'o'];
        let result = parse_cbor_to_json(&data).unwrap();
        assert_eq!(result, "\"Hello\"");
    }

    #[test]
    fn test_parse_cbor_array() {
        let data = [0x82, 0x01, 0x02];
        let result = parse_cbor_to_json(&data).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }
}
