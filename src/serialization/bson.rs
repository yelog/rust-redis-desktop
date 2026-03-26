use serde_json::Value as JsonValue;

pub fn is_bson_serialization(data: &[u8]) -> bool {
    if data.len() < 5 {
        return false;
    }

    let len = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if len < 5 || len as usize > data.len() + 4 {
        return false;
    }

    let last_byte = *data.last().unwrap_or(&0);
    if last_byte != 0x00 {
        return false;
    }

    if data.len() > 4 {
        let doc_type = data[4];
        matches!(
            doc_type,
            0x01 | 0x02
                | 0x03
                | 0x04
                | 0x05
                | 0x07
                | 0x08
                | 0x09
                | 0x0A
                | 0x0B
                | 0x10
                | 0x11
                | 0x12
                | 0x13
                | 0x14
                | 0x63
        )
    } else {
        false
    }
}

pub fn parse_bson_to_json(data: &[u8]) -> Result<String, String> {
    let bson_doc = bson::Document::from_reader(&mut std::io::Cursor::new(data))
        .map_err(|e| format!("BSON 解析失败: {}", e))?;

    let json = bson_to_json(&bson_doc);
    serde_json::to_string_pretty(&json).map_err(|e| format!("JSON 序列化失败: {}", e))
}

fn bson_to_json(doc: &bson::Document) -> JsonValue {
    let mut map = serde_json::Map::new();

    for (key, value) in doc.iter() {
        let json_value = bson_value_to_json(value);
        map.insert(key.clone(), json_value);
    }

    JsonValue::Object(map)
}

fn bson_value_to_json(value: &bson::Bson) -> JsonValue {
    match value {
        bson::Bson::Null => JsonValue::Null,
        bson::Bson::Boolean(b) => JsonValue::Bool(*b),
        bson::Bson::Int32(i) => JsonValue::Number((*i).into()),
        bson::Bson::Int64(i) => JsonValue::Number((*i).into()),
        bson::Bson::Double(d) => serde_json::Number::from_f64(*d)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        bson::Bson::String(s) => JsonValue::String(s.clone()),
        bson::Bson::Array(arr) => JsonValue::Array(arr.iter().map(bson_value_to_json).collect()),
        bson::Bson::Document(doc) => bson_to_json(doc),
        bson::Bson::Binary(bin) => {
            let hex: String = bin.bytes.iter().map(|b| format!("{:02x}", b)).collect();
            JsonValue::String(format!("<Binary({:?}): {}>", bin.subtype, hex))
        }
        bson::Bson::ObjectId(oid) => JsonValue::String(format!("ObjectId(\"{}\")", oid)),
        bson::Bson::DateTime(dt) => JsonValue::String(dt.to_string()),
        bson::Bson::Timestamp(ts) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "time".to_string(),
                JsonValue::Number((ts.time as i64).into()),
            );
            map.insert(
                "increment".to_string(),
                JsonValue::Number((ts.increment as i64).into()),
            );
            JsonValue::Object(map)
        }
        bson::Bson::RegularExpression(regex) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "pattern".to_string(),
                JsonValue::String(regex.pattern.clone()),
            );
            map.insert(
                "options".to_string(),
                JsonValue::String(regex.options.clone()),
            );
            JsonValue::Object(map)
        }
        bson::Bson::JavaScriptCode(code) => JsonValue::String(format!("<JS: {}>", code)),
        bson::Bson::JavaScriptCodeWithScope(code_with_scope) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "code".to_string(),
                JsonValue::String(code_with_scope.code.clone()),
            );
            map.insert("scope".to_string(), bson_to_json(&code_with_scope.scope));
            JsonValue::Object(map)
        }
        bson::Bson::MaxKey => JsonValue::String("<MaxKey>".to_string()),
        bson::Bson::MinKey => JsonValue::String("<MinKey>".to_string()),
        bson::Bson::Undefined => JsonValue::String("<Undefined>".to_string()),
        bson::Bson::Decimal128(d) => JsonValue::String(d.to_string()),
        bson::Bson::Symbol(s) => JsonValue::String(format!("<Symbol: {}>", s)),
        bson::Bson::DbPointer(db_pointer) => {
            JsonValue::String(format!("<DbPointer: {:?}>", db_pointer))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_bson_empty() {
        assert!(!is_bson_serialization(&[]));
        assert!(!is_bson_serialization(&[0x01, 0x02, 0x03]));
    }

    #[test]
    fn test_detect_bson_valid() {
        let mut data = vec![0x05, 0x00, 0x00, 0x00];
        data.push(0x00);
        assert!(is_bson_serialization(&data));
    }
}
