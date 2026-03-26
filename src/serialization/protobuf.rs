use serde_json::Value as JsonValue;

pub fn is_protobuf_data(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let first_byte = data[0];
    let field_number = first_byte >> 3;
    let wire_type = first_byte & 0x07;

    if field_number == 0 || field_number > 100 {
        return false;
    }

    if wire_type > 5 {
        return false;
    }

    let parsed = parse_protobuf_fields(data);
    !parsed.is_empty()
}

pub fn try_parse_protobuf_as_any(data: &[u8]) -> Option<JsonValue> {
    let fields = parse_protobuf_fields(data);
    if fields.is_empty() {
        None
    } else {
        Some(JsonValue::Object(fields))
    }
}

pub fn try_parse_with_schema(data: &[u8], message_name: &str) -> Option<JsonValue> {
    let registry = crate::protobuf_schema::PROTO_REGISTRY();
    registry.decode_with_schema(data, message_name).ok()
}

pub fn get_available_messages() -> Vec<String> {
    let registry = crate::protobuf_schema::PROTO_REGISTRY();
    registry
        .list_messages()
        .iter()
        .map(|m| m.full_name.clone())
        .collect()
}

pub fn has_schema() -> bool {
    let registry = crate::protobuf_schema::PROTO_REGISTRY();
    !registry.is_empty()
}

fn parse_protobuf_fields(data: &[u8]) -> serde_json::Map<String, JsonValue> {
    let mut result = serde_json::Map::new();

    if data.len() < 2 {
        return result;
    }

    let first_byte = data[0];
    let field_number = first_byte >> 3;
    let wire_type = first_byte & 0x07;

    if field_number == 0 || field_number > 100 {
        return result;
    }

    if wire_type > 5 {
        return result;
    }

    let mut pos = 0;
    let mut valid_fields = 0;

    while pos < data.len() {
        if let Some((field_num, wire_ty, bytes_read)) = decode_varint_field_header(&data[pos..]) {
            if field_num == 0 || field_num > 100 {
                break;
            }
            pos += bytes_read;

            let field_name = format!("field_{}", field_num);

            match wire_ty {
                0 => {
                    if let Some((value, bytes_read)) = decode_varint(&data[pos..]) {
                        pos += bytes_read;
                        if let Some(num) = serde_json::Number::from_u128(value as u128) {
                            result.insert(field_name, JsonValue::Number(num));
                            valid_fields += 1;
                        }
                    } else {
                        break;
                    }
                }
                1 => {
                    if pos + 8 <= data.len() {
                        let value = u64::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                            data[pos + 4],
                            data[pos + 5],
                            data[pos + 6],
                            data[pos + 7],
                        ]);
                        pos += 8;
                        if let Some(num) = serde_json::Number::from_u128(value as u128) {
                            result.insert(field_name, JsonValue::Number(num));
                            valid_fields += 1;
                        }
                    } else {
                        break;
                    }
                }
                2 => {
                    if let Some((length, bytes_read)) = decode_varint(&data[pos..]) {
                        pos += bytes_read;
                        let length = length as usize;
                        if length > 0 && pos + length <= data.len() {
                            let bytes = &data[pos..pos + length];
                            pos += length;

                            if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                                if s.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace()) {
                                    result.insert(field_name, JsonValue::String(s));
                                    valid_fields += 1;
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                5 => {
                    if pos + 4 <= data.len() {
                        let value = u32::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                        if let Some(num) = serde_json::Number::from_u128(value as u128) {
                            result.insert(field_name, JsonValue::Number(num));
                            valid_fields += 1;
                        }
                    } else {
                        break;
                    }
                }
                _ => {
                    break;
                }
            }
        } else {
            break;
        }

        if valid_fields > 20 {
            break;
        }
    }

    if valid_fields == 0 {
        result.clear();
    }

    result
}

fn decode_varint_field_header(data: &[u8]) -> Option<(u32, u32, usize)> {
    if data.is_empty() {
        return None;
    }

    let (tag, bytes_read) = decode_varint(data)?;
    let field_number = (tag >> 3) as u32;
    let wire_type = (tag & 0x07) as u32;

    Some((field_number, wire_type, bytes_read))
}

fn decode_varint(data: &[u8]) -> Option<(u64, usize)> {
    if data.is_empty() {
        return None;
    }

    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = 0;

    loop {
        if pos >= data.len() {
            return None;
        }

        let byte = data[pos];
        pos += 1;

        result |= ((byte & 0x7F) as u64) << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;
        if shift >= 64 {
            return None;
        }
    }

    Some((result, pos))
}
