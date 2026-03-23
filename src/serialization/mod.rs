pub mod java_converters;

pub use jaded::{Content, Parser};
use serde_json::Value as JsonValue;
use std::io::Cursor;

/// Check if data is Java serialization format
pub fn is_java_serialization(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0xAC && data[1] == 0xED
}

pub fn extract_inner_value(json: JsonValue) -> JsonValue {
    match json {
        JsonValue::Object(ref obj) => {
            if let Some(inner) = obj.get("Object") {
                extract_inner_value(inner.clone())
            } else if let Some(inner) = obj.get("JavaString") {
                inner.clone()
            } else if let Some(inner) = obj.get("Primitive") {
                extract_inner_value(inner.clone())
            } else if let Some(inner) = obj.get("Array") {
                extract_inner_value(inner.clone())
            } else if let Some(inner) = obj.get("Enum") {
                if let Some(arr) = inner.as_array() {
                    if arr.len() == 2 {
                        JsonValue::String(format!(
                            "{}::{}",
                            simplify_class_name(arr[0].as_str().unwrap_or("")),
                            arr[1].as_str().unwrap_or("")
                        ))
                    } else {
                        inner.clone()
                    }
                } else {
                    inner.clone()
                }
            } else if let Some(inner) = obj.get("Class") {
                JsonValue::String(format!("class {}", inner.as_str().unwrap_or("")))
            } else if obj.contains_key("Block") {
                JsonValue::String("<Block data>".to_string())
            } else if obj.contains_key("Loop") {
                JsonValue::String("<循环引用>".to_string())
            } else if obj.get("Null").map(|v| v.is_string()).unwrap_or(false) {
                JsonValue::Null
            } else if obj.contains_key("class") {
                let mut result = serde_json::Map::new();
                result.insert(
                    "class".to_string(),
                    obj.get("class").cloned().unwrap_or(JsonValue::Null),
                );

                if let Some(fields) = obj.get("fields") {
                    let extracted_fields = extract_fields(fields);
                    result.insert("fields".to_string(), JsonValue::Object(extracted_fields));
                }

                if let Some(annotations) = obj.get("annotations") {
                    result.insert("annotations".to_string(), annotations.clone());
                }

                JsonValue::Object(result)
            } else {
                let extracted: serde_json::Map<String, JsonValue> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), extract_inner_value(v.clone())))
                    .collect();
                JsonValue::Object(extracted)
            }
        }
        JsonValue::Array(arr) => {
            JsonValue::Array(arr.into_iter().map(extract_inner_value).collect())
        }
        other => other,
    }
}

fn extract_fields(fields: &JsonValue) -> serde_json::Map<String, JsonValue> {
    match fields {
        JsonValue::Object(obj) => obj
            .iter()
            .map(|(k, v)| (k.clone(), extract_inner_value(v.clone())))
            .collect(),
        _ => serde_json::Map::new(),
    }
}

pub fn parse_java_to_json(data: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(data);
    let mut parser = Parser::new(cursor).map_err(|e| e.to_string())?;
    let content = parser.read().map_err(|e| e.to_string())?;

    match content {
        Content::Object(value) => {
            let json = serde_json::to_value(value).map_err(|e| e.to_string())?;
            let extracted = extract_inner_value(json);
            serde_json::to_string_pretty(&extracted).map_err(|e| e.to_string())
        }
        Content::Block(bytes) => Ok(format!("<Block data: {} bytes>", bytes.len())),
    }
}

/// Simplify class name for display
pub fn simplify_class_name(full_name: &str) -> String {
    if full_name.starts_with("java.lang.") {
        return full_name.strip_prefix("java.lang.").unwrap().to_string();
    }
    let parts: Vec<&str> = full_name.split('.').collect();
    if parts.len() > 1 {
        parts.last().unwrap().to_string()
    } else {
        full_name.to_string()
    }
}

/// Format class name with package info
pub fn format_class_name_with_package(full_name: &str) -> String {
    let simplified = simplify_class_name(full_name);
    if simplified == full_name {
        simplified
    } else {
        format!("{} ({})", simplified, full_name)
    }
}
