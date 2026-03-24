use crate::formatter::{FormatterType, TransformResult};
use base64::{engine::general_purpose, Engine as _};
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use serde_json::Value as JsonValue;
use std::io::Read;

pub fn apply_preset_formatter(formatter_type: &FormatterType, input: &[u8]) -> TransformResult {
    match formatter_type {
        FormatterType::Json => format_json(input),
        FormatterType::Hex => to_hex(input),
        FormatterType::Base64 => from_base64(input),
        FormatterType::Base64Url => from_base64url(input),
        FormatterType::UrlEncode => url_decode(input),
        FormatterType::Gzip => decompress_gzip(input),
        FormatterType::Zlib => decompress_zlib(input),
        FormatterType::Deflate => decompress_deflate(input),
        FormatterType::Brotli => decompress_brotli(input),
        FormatterType::MsgPack => decode_msgpack(input),
        FormatterType::Protobuf => decode_protobuf(input),
        FormatterType::Custom(_) => {
            TransformResult::Error("Custom formatters are not supported yet".to_string())
        }
    }
}

fn format_json(input: &[u8]) -> TransformResult {
    match serde_json::from_slice::<JsonValue>(input) {
        Ok(json) => match serde_json::to_string_pretty(&json) {
            Ok(formatted) => TransformResult::Text(formatted),
            Err(e) => TransformResult::Error(format!("Failed to format JSON: {}", e)),
        },
        Err(e) => TransformResult::Error(format!("Invalid JSON: {}", e)),
    }
}

fn to_hex(input: &[u8]) -> TransformResult {
    let hex: String = input.iter().map(|b| format!("{:02x}", b)).collect();
    TransformResult::Text(hex)
}

fn from_base64(input: &[u8]) -> TransformResult {
    let input_str = String::from_utf8_lossy(input);
    match general_purpose::STANDARD.decode(input_str.trim()) {
        Ok(decoded) => TransformResult::Binary(decoded),
        Err(e) => TransformResult::Error(format!("Invalid Base64: {}", e)),
    }
}

fn from_base64url(input: &[u8]) -> TransformResult {
    let input_str = String::from_utf8_lossy(input);
    match general_purpose::URL_SAFE.decode(input_str.trim()) {
        Ok(decoded) => TransformResult::Binary(decoded),
        Err(e) => TransformResult::Error(format!("Invalid Base64URL: {}", e)),
    }
}

fn url_decode(input: &[u8]) -> TransformResult {
    let input_str = String::from_utf8_lossy(input);
    match urlencoding::decode(&input_str) {
        Ok(decoded) => TransformResult::Text(decoded.into_owned()),
        Err(e) => TransformResult::Error(format!("URL decode error: {}", e)),
    }
}

fn decompress_gzip(input: &[u8]) -> TransformResult {
    let mut decoder = GzDecoder::new(input);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => TransformResult::Binary(decompressed),
        Err(e) => TransformResult::Error(format!("Gzip decompression error: {}", e)),
    }
}

fn decompress_zlib(input: &[u8]) -> TransformResult {
    let mut decoder = ZlibDecoder::new(input);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => TransformResult::Binary(decompressed),
        Err(e) => TransformResult::Error(format!("Zlib decompression error: {}", e)),
    }
}

fn decompress_deflate(input: &[u8]) -> TransformResult {
    let mut decoder = DeflateDecoder::new(input);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => TransformResult::Binary(decompressed),
        Err(e) => TransformResult::Error(format!("Deflate decompression error: {}", e)),
    }
}

fn decompress_brotli(input: &[u8]) -> TransformResult {
    use brotli::Decompressor;
    let mut decoder = Decompressor::new(input, 4096);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => TransformResult::Binary(decompressed),
        Err(e) => TransformResult::Error(format!("Brotli decompression error: {}", e)),
    }
}

fn decode_msgpack(input: &[u8]) -> TransformResult {
    match rmpv::decode::read_value(&mut &input[..]) {
        Ok(value) => {
            let json = msgpack_to_json(&value);
            TransformResult::Text(json)
        }
        Err(e) => TransformResult::Error(format!("MsgPack decode error: {}", e)),
    }
}

fn msgpack_to_json(value: &rmpv::Value) -> String {
    let json = match value {
        rmpv::Value::Nil => JsonValue::Null,
        rmpv::Value::Boolean(b) => JsonValue::Bool(*b),
        rmpv::Value::Integer(i) => JsonValue::Number(i.as_i64().unwrap_or(0).into()),
        rmpv::Value::F32(f) => {
            JsonValue::Number(serde_json::Number::from_f64(*f as f64).unwrap_or_else(|| 0.into()))
        }
        rmpv::Value::F64(f) => {
            JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into()))
        }
        rmpv::Value::String(s) => JsonValue::String(s.as_str().unwrap_or("").to_string()),
        rmpv::Value::Binary(b) => {
            let hex: String = b.iter().map(|byte| format!("{:02x}", byte)).collect();
            JsonValue::String(format!("0x{}", hex))
        }
        rmpv::Value::Array(arr) => JsonValue::Array(
            arr.iter()
                .map(|v| serde_json::from_str(&msgpack_to_json(v)).unwrap_or(JsonValue::Null))
                .collect(),
        ),
        rmpv::Value::Map(map) => {
            let obj: serde_json::Map<String, JsonValue> = map
                .iter()
                .filter_map(|(k, v)| {
                    let key = match k {
                        rmpv::Value::String(s) => s.as_str().unwrap_or("").to_string(),
                        rmpv::Value::Integer(i) => i.as_i64().unwrap_or(0).to_string(),
                        _ => return None,
                    };
                    let val = serde_json::from_str(&msgpack_to_json(v)).unwrap_or(JsonValue::Null);
                    Some((key, val))
                })
                .collect();
            JsonValue::Object(obj)
        }
        rmpv::Value::Ext(_ty, data) => {
            let hex: String = data.iter().map(|b| format!("{:02x}", b)).collect();
            JsonValue::String(format!("Extension({})", hex))
        }
    };
    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
}

fn decode_protobuf(input: &[u8]) -> TransformResult {
    let mut result = String::new();
    result.push_str("Protobuf Raw Decode:\n");
    result.push_str(&format!("Length: {} bytes\n\n", input.len()));

    let mut pos = 0;
    let mut field_num = 1;

    while pos < input.len() {
        if pos >= input.len() {
            break;
        }

        let b = input[pos];
        let wire_type = b & 0x07;
        let field_number = b >> 3;

        result.push_str(&format!("Field {}: ", field_number));
        pos += 1;

        match wire_type {
            0 => {
                let mut value: u64 = 0;
                let mut shift = 0;
                while pos < input.len() {
                    let byte = input[pos];
                    pos += 1;
                    value |= ((byte & 0x7F) as u64) << shift;
                    if byte & 0x80 == 0 {
                        break;
                    }
                    shift += 7;
                }
                result.push_str(&format!("varint = {}\n", value));
            }
            1 => {
                if pos + 8 <= input.len() {
                    let bytes: [u8; 8] = input[pos..pos + 8].try_into().unwrap_or([0; 8]);
                    let value = f64::from_le_bytes(bytes);
                    result.push_str(&format!("64-bit = {} (f64)\n", value));
                    pos += 8;
                } else {
                    result.push_str("64-bit (truncated)\n");
                    break;
                }
            }
            2 => {
                if pos >= input.len() {
                    break;
                }
                let len_byte = input[pos];
                pos += 1;

                let mut len = 0usize;
                let mut shift = 0;
                let mut tmp_pos = pos - 1;
                loop {
                    if tmp_pos >= input.len() {
                        break;
                    }
                    let byte = input[tmp_pos];
                    tmp_pos += 1;
                    len |= ((byte & 0x7F) as usize) << shift;
                    if byte & 0x80 == 0 {
                        pos = tmp_pos;
                        break;
                    }
                    shift += 7;
                }

                if len > input.len() - pos {
                    len = input.len() - pos;
                }

                let data = &input[pos..pos + len];
                pos += len;

                if let Ok(s) = std::str::from_utf8(data) {
                    result.push_str(&format!("string = \"{}\"\n", s));
                } else {
                    let hex: String = data.iter().map(|b| format!("{:02x}", b)).collect();
                    result.push_str(&format!("bytes = {}\n", hex));
                }
            }
            5 => {
                if pos + 4 <= input.len() {
                    let bytes: [u8; 4] = input[pos..pos + 4].try_into().unwrap_or([0; 4]);
                    let value = f32::from_le_bytes(bytes);
                    result.push_str(&format!("32-bit = {} (f32)\n", value));
                    pos += 4;
                } else {
                    result.push_str("32-bit (truncated)\n");
                    break;
                }
            }
            wt => {
                result.push_str(&format!("unknown wire type {}\n", wt));
                break;
            }
        }

        field_num += 1;
        if field_num > 100 {
            result.push_str("\n... (truncated, too many fields)");
            break;
        }
    }

    TransformResult::Text(result)
}
