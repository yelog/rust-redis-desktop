use serde_json::Value as JsonValue;

pub fn is_protobuf_data(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let first_byte = data[0];
    let field_number = first_byte >> 3;
    let wire_type = first_byte & 0x07;

    if field_number == 0 {
        return false;
    }

    if wire_type > 5 {
        return false;
    }

    true
}

pub fn try_parse_protobuf_as_any(data: &[u8]) -> Option<JsonValue> {
    if !is_protobuf_data(data) {
        return None;
    }

    let mut result = serde_json::Map::new();
    let mut pos = 0;

    while pos < data.len() {
        if let Some((field_number, wire_type, bytes_read)) =
            decode_varint_field_header(&data[pos..])
        {
            pos += bytes_read;

            let field_name = format!("field_{}", field_number);

            match wire_type {
                0 => {
                    if let Some((value, bytes_read)) = decode_varint(&data[pos..]) {
                        pos += bytes_read;
                        if let Some(num) = serde_json::Number::from_u128(value as u128) {
                            result.insert(field_name, JsonValue::Number(num));
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
                        }
                    } else {
                        break;
                    }
                }
                2 => {
                    if let Some((length, bytes_read)) = decode_varint(&data[pos..]) {
                        pos += bytes_read;
                        let length = length as usize;
                        if pos + length <= data.len() {
                            let bytes = &data[pos..pos + length];
                            pos += length;

                            if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                                result.insert(field_name, JsonValue::String(s));
                            } else {
                                let hex: String =
                                    bytes.iter().map(|b| format!("{:02x}", b)).collect();
                                result.insert(field_name, JsonValue::String(hex));
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
    }

    if result.is_empty() {
        None
    } else {
        Some(JsonValue::Object(result))
    }
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
