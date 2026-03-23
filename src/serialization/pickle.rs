use serde_json::Value as JsonValue;
use std::io::Cursor;

pub fn is_pickle_serialization(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    if data[0] == 0x80 {
        let version = data[1];
        if version >= 2 && version <= 5 {
            if let Some(&last) = data.last() {
                return last == b'.';
            }
        }
        if version == 0 || version == 1 {
            return true;
        }
    }

    if matches!(data[0], b'(' | b'c' | b']' | b'}') {
        if let Some(&last) = data.last() {
            return last == b'.';
        }
    }

    false
}

pub fn get_pickle_version(data: &[u8]) -> Option<u8> {
    if data.len() >= 2 && data[0] == 0x80 {
        Some(data[1])
    } else {
        Some(0)
    }
}

pub fn parse_pickle_to_json(data: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(data);

    let value: JsonValue = serde_pickle::from_reader(cursor, serde_pickle::DeOptions::new())
        .map_err(|e| format!("Pickle 解析失败: {}", e))?;

    serde_json::to_string_pretty(&value).map_err(|e| format!("JSON 序列化失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pickle_protocol_2() {
        let mut data = vec![0x80, 0x02];
        data.extend_from_slice(b"]q\x00.");
        assert!(is_pickle_serialization(&data));
    }

    #[test]
    fn test_detect_pickle_protocol_3() {
        let mut data = vec![0x80, 0x03];
        data.extend_from_slice(b"]q\x00.");
        assert!(is_pickle_serialization(&data));
    }

    #[test]
    fn test_detect_pickle_protocol_4() {
        let mut data = vec![0x80, 0x04];
        data.extend_from_slice(b"]q\x00.");
        assert!(is_pickle_serialization(&data));
    }

    #[test]
    fn test_detect_pickle_protocol_5() {
        let mut data = vec![0x80, 0x05];
        data.extend_from_slice(b"]q\x00.");
        assert!(is_pickle_serialization(&data));
    }

    #[test]
    fn test_parse_pickle_none() {
        let data = b"\x80\x04N.";
        let result = parse_pickle_to_json(data).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn test_parse_pickle_bool() {
        let data = b"\x80\x04\x88.";
        let result = parse_pickle_to_json(data).unwrap();
        assert_eq!(result, "true");

        let data = b"\x80\x04\x89.";
        let result = parse_pickle_to_json(data).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_parse_pickle_int() {
        let data = b"\x80\x04K\x7f.";
        let result = parse_pickle_to_json(data).unwrap();
        assert_eq!(result, "127");
    }

    #[test]
    fn test_parse_pickle_string() {
        let data = b"\x80\x04\x95\x05\x00\x00\x00\x00\x00\x00\x00\x8c\x05Hello\x94.";
        let result = parse_pickle_to_json(data).unwrap();
        assert_eq!(result, "\"Hello\"");
    }

    #[test]
    fn test_parse_pickle_list() {
        let data = b"\x80\x04\x95\x05\x00\x00\x00\x00\x00\x00\x00]\x94(K\x01K\x02K\x03e.";
        let result = parse_pickle_to_json(data).unwrap();
        assert_eq!(result, "[\n  1,\n  2,\n  3\n]");
    }

    #[test]
    fn test_parse_pickle_dict() {
        let data = b"\x80\x04\x95\x0b\x00\x00\x00\x00\x00\x00\x00}\x94(\x8c\x01a\x94K\x01\x8c\x01b\x94K\x02u.";
        let result = parse_pickle_to_json(data).unwrap();
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"b\": 2"));
    }
}
