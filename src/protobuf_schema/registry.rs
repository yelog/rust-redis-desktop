use crate::protobuf_schema::types::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ProtoRegistry {
    files: HashMap<String, ProtoFile>,
    messages: HashMap<String, MessageDef>,
}

impl ProtoRegistry {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            messages: HashMap::new(),
        }
    }

    pub fn import_file(&mut self, path: &Path) -> Result<Vec<String>, String> {
        let proto_file = crate::protobuf_schema::parse_proto_file(path)?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut message_names = Vec::new();

        for msg in &proto_file.messages {
            message_names.push(msg.full_name.clone());
            self.messages.insert(msg.full_name.clone(), msg.clone());
        }

        self.files.insert(file_name, proto_file);

        Ok(message_names)
    }

    pub fn import_content(&mut self, content: &str) -> Result<Vec<String>, String> {
        let proto_file = crate::protobuf_schema::parse_proto_content(content)?;

        let mut message_names = Vec::new();

        for msg in &proto_file.messages {
            message_names.push(msg.full_name.clone());
            self.messages.insert(msg.full_name.clone(), msg.clone());
        }

        self.files.insert("inline".to_string(), proto_file);

        Ok(message_names)
    }

    pub fn get_message(&self, full_name: &str) -> Option<&MessageDef> {
        self.messages.get(full_name)
    }

    pub fn list_messages(&self) -> Vec<&MessageDef> {
        self.messages.values().collect()
    }

    pub fn clear(&mut self) {
        self.files.clear();
        self.messages.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn decode_with_schema(&self, data: &[u8], message_name: &str) -> Result<JsonValue, String> {
        let msg = self
            .get_message(message_name)
            .ok_or_else(|| format!("消息类型未找到: {}", message_name))?;

        decode_message(data, msg)
    }
}

fn decode_message(data: &[u8], msg: &MessageDef) -> Result<JsonValue, String> {
    let mut result = serde_json::Map::new();
    let mut pos = 0;

    while pos < data.len() {
        if pos + 1 > data.len() {
            break;
        }

        let (tag, bytes_read) = decode_varint(&data[pos..])?;
        pos += bytes_read;

        let field_number = (tag >> 3) as u32;
        let wire_type = (tag & 0x07) as u32;

        let field = msg.get_field(field_number);

        let (value, bytes_read) = decode_field(&data[pos..], wire_type, field)?;
        pos += bytes_read;

        let key = field
            .map(|f| f.name.clone())
            .unwrap_or_else(|| format!("field_{}", field_number));

        if result.contains_key(&key) {
            let existing = result.get(&key).cloned();
            if let Some(JsonValue::Array(mut arr)) = existing {
                arr.push(value);
                result.insert(key, JsonValue::Array(arr));
            } else if let Some(old) = existing {
                result.insert(key, JsonValue::Array(vec![old, value]));
            }
        } else {
            result.insert(key, value);
        }
    }

    Ok(JsonValue::Object(result))
}

fn decode_field(
    data: &[u8],
    wire_type: u32,
    field: Option<&FieldDef>,
) -> Result<(JsonValue, usize), String> {
    match wire_type {
        0 => {
            let (value, bytes_read) = decode_varint(data)?;
            let json_value = match field.map(|f| &f.field_type) {
                Some(FieldType::Bool) => JsonValue::Bool(value != 0),
                Some(FieldType::SInt32) => JsonValue::Number(zigzag_decode(value as i64).into()),
                Some(FieldType::SInt64) => JsonValue::Number(zigzag_decode(value as i64).into()),
                _ => JsonValue::Number(value.into()),
            };
            Ok((json_value, bytes_read))
        }
        1 => {
            if data.len() < 8 {
                return Err("数据不完整: 需要 8 字节".to_string());
            }
            let bytes: [u8; 8] = data[..8].try_into().unwrap();
            let json_value = match field.map(|f| &f.field_type) {
                Some(FieldType::Double) => {
                    let val = f64::from_le_bytes(bytes);
                    serde_json::Number::from_f64(val)
                        .map(JsonValue::Number)
                        .unwrap_or(JsonValue::Null)
                }
                Some(FieldType::Fixed64) | Some(FieldType::UInt64) => {
                    JsonValue::Number(u64::from_le_bytes(bytes).into())
                }
                Some(FieldType::SFixed64) | Some(FieldType::Int64) => {
                    JsonValue::Number(i64::from_le_bytes(bytes).into())
                }
                _ => JsonValue::Number(u64::from_le_bytes(bytes).into()),
            };
            Ok((json_value, 8))
        }
        2 => {
            let (len, len_bytes) = decode_varint(data)?;
            let len = len as usize;

            if data.len() < len_bytes + len {
                return Err(format!("数据不完整: 需要 {} 字节", len_bytes + len));
            }

            let bytes = &data[len_bytes..len_bytes + len];

            let json_value = match field.map(|f| &f.field_type) {
                Some(FieldType::String) => {
                    let s = String::from_utf8_lossy(bytes).to_string();
                    JsonValue::String(s)
                }
                Some(FieldType::Message(_)) => {
                    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                    JsonValue::String(format!("<embedded message: {}>", hex))
                }
                _ => {
                    if let Ok(s) = std::str::from_utf8(bytes) {
                        if s.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace()) {
                            JsonValue::String(s.to_string())
                        } else {
                            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                            JsonValue::String(format!("0x{}", hex))
                        }
                    } else {
                        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                        JsonValue::String(format!("0x{}", hex))
                    }
                }
            };

            Ok((json_value, len_bytes + len))
        }
        5 => {
            if data.len() < 4 {
                return Err("数据不完整: 需要 4 字节".to_string());
            }
            let bytes: [u8; 4] = data[..4].try_into().unwrap();
            let json_value = match field.map(|f| &f.field_type) {
                Some(FieldType::Float) => {
                    let val = f32::from_le_bytes(bytes);
                    serde_json::Number::from_f64(val as f64)
                        .map(JsonValue::Number)
                        .unwrap_or(JsonValue::Null)
                }
                Some(FieldType::Fixed32) | Some(FieldType::UInt32) => {
                    JsonValue::Number(u32::from_le_bytes(bytes).into())
                }
                Some(FieldType::SFixed32) | Some(FieldType::Int32) => {
                    JsonValue::Number(i32::from_le_bytes(bytes).into())
                }
                _ => JsonValue::Number(u32::from_le_bytes(bytes).into()),
            };
            Ok((json_value, 4))
        }
        _ => Err(format!("未知的 wire type: {}", wire_type)),
    }
}

fn decode_varint(data: &[u8]) -> Result<(u64, usize), String> {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = 0;

    loop {
        if pos >= data.len() {
            return Err("数据不完整".to_string());
        }

        let byte = data[pos];
        pos += 1;

        result |= ((byte & 0x7F) as u64) << shift;

        if (byte & 0x80) == 0 {
            break;
        }

        shift += 7;
        if shift >= 64 {
            return Err("varint 过长".to_string());
        }
    }

    Ok((result, pos))
}

fn zigzag_decode(n: i64) -> i64 {
    (n >> 1) ^ -(n & 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = ProtoRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_import_content() {
        let mut registry = ProtoRegistry::new();
        let content = r#"
            package test;
            message Simple {
                int32 value = 1;
            }
        "#;

        let names = registry.import_content(content).unwrap();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "test.Simple");
    }

    #[test]
    fn test_decode_simple() {
        let mut registry = ProtoRegistry::new();
        let content = r#"
            package test;
            message Simple {
                int32 value = 1;
            }
        "#;

        registry.import_content(content).unwrap();

        let data: Vec<u8> = vec![0x08, 0x2A];
        let result = registry.decode_with_schema(&data, "test.Simple").unwrap();

        assert_eq!(result["value"], JsonValue::Number(42.into()));
    }
}
