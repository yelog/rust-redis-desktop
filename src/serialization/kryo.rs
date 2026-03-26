use serde_json::{Map, Value as JsonValue};

#[derive(Debug, Clone)]
pub enum KryoValue {
    Null,
    Byte(i8),
    Char(char),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Boolean(bool),
    String(Option<String>),
    ByteArray(Vec<u8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(Vec<KryoValue>),
    Map(Vec<(KryoValue, KryoValue)>),
    Unknown {
        type_id: u8,
        data: Vec<u8>,
        message: String,
    },
}

pub fn is_kryo_serialization(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let first = data[0];

    if matches!(
        first,
        0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x08 | 0x09
    ) {
        return true;
    }

    if first == 0x0A || first == 0x0B || first == 0x0C {
        if data.len() > 2 {
            return true;
        }
    }

    false
}

pub fn is_fst_serialization(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    matches!(data[0], 0xF0 | 0xF1 | 0xF2 | 0xF3)
}

pub fn detect_kryo_or_fst(data: &[u8]) -> Option<&'static str> {
    if is_fst_serialization(data) {
        return Some("FST");
    }
    if is_kryo_serialization(data) {
        return Some("Kryo");
    }
    None
}

struct KryoReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> KryoReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        if self.pos >= self.data.len() {
            return Err("数据不完整".to_string());
        }
        let val = self.data[self.pos];
        self.pos += 1;
        Ok(val)
    }

    fn read_i8(&mut self) -> Result<i8, String> {
        let val = self.read_u8()?;
        Ok(val as i8)
    }

    fn read_i16(&mut self) -> Result<i16, String> {
        if self.pos + 2 > self.data.len() {
            return Err("数据不完整".to_string());
        }
        let val = i16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(val)
    }

    fn read_i32(&mut self) -> Result<i32, String> {
        if self.pos + 4 > self.data.len() {
            return Err("数据不完整".to_string());
        }
        let val = i32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(val)
    }

    fn read_varint(&mut self) -> Result<i32, String> {
        let mut result: i32 = 0;
        let mut shift = 0;

        loop {
            if shift > 28 {
                return Err("varint 过长".to_string());
            }
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i32) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }

        Ok(result)
    }

    fn read_varlong(&mut self) -> Result<i64, String> {
        let mut result: i64 = 0;
        let mut shift = 0;

        loop {
            if shift > 56 {
                return Err("varlong 过长".to_string());
            }
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i64) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }

        Ok(result)
    }

    fn read_float(&mut self) -> Result<f32, String> {
        if self.pos + 4 > self.data.len() {
            return Err("数据不完整".to_string());
        }
        let val = f32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(val)
    }

    fn read_double(&mut self) -> Result<f64, String> {
        if self.pos + 8 > self.data.len() {
            return Err("数据不完整".to_string());
        }
        let val = f64::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ]);
        self.pos += 8;
        Ok(val)
    }

    fn read_string(&mut self) -> Result<Option<String>, String> {
        let len = self.read_varint()?;
        if len < 0 {
            return Ok(None);
        }
        let len = len as usize;
        if self.pos + len > self.data.len() {
            return Err(format!(
                "字符串长度超出数据范围: {} > {}",
                len,
                self.remaining()
            ));
        }
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        String::from_utf8(bytes.to_vec())
            .map(Some)
            .map_err(|_| "无效的 UTF-8 字符串".to_string())
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, String> {
        if self.pos + len > self.data.len() {
            return Err("数据不完整".to_string());
        }
        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(bytes)
    }
}

fn parse_fst(data: &[u8]) -> Result<KryoValue, String> {
    if data.len() < 10 {
        return Err("FST 数据太短".to_string());
    }

    let _version = data[0];

    let length = i64::from_be_bytes([
        data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
    ]);

    if length < 0 {
        return Err("FST 字符串长度为负数".to_string());
    }

    let length = length as usize;
    if 9 + length > data.len() {
        return Err(format!(
            "FST 数据不完整: 需要 {} 字节，实际 {} 字节",
            9 + length,
            data.len()
        ));
    }

    let string_bytes = &data[9..9 + length];
    match String::from_utf8(string_bytes.to_vec()) {
        Ok(s) => Ok(KryoValue::String(Some(s))),
        Err(_) => Ok(KryoValue::ByteArray(string_bytes.to_vec())),
    }
}

pub fn parse_kryo_basic(data: &[u8]) -> Result<KryoValue, String> {
    if is_fst_serialization(data) {
        return parse_fst(data);
    }

    let mut reader = KryoReader::new(data);

    let type_id = reader.read_u8()?;

    match type_id {
        0x00 => Ok(KryoValue::Null),
        0x01 => reader.read_i8().map(KryoValue::Byte),
        0x03 => reader.read_i16().map(KryoValue::Short),
        0x04 => reader.read_varint().map(KryoValue::Int),
        0x05 => reader.read_varlong().map(KryoValue::Long),
        0x06 => reader.read_float().map(KryoValue::Float),
        0x07 => reader.read_double().map(KryoValue::Double),
        0x08 => Ok(KryoValue::Boolean(true)),
        0x09 => Ok(KryoValue::Boolean(false)),
        0x0A => reader.read_string().map(KryoValue::String),
        0x0B => {
            let len = reader.read_varint()? as usize;
            let mut items = Vec::with_capacity(len);
            for _ in 0..len {
                let item = parse_kryo_basic(&data[reader.pos..])?;
                reader.pos += estimate_kryo_value_size(&item);
                items.push(item);
            }
            Ok(KryoValue::List(items))
        }
        0x0C => {
            let len = reader.read_varint()? as usize;
            let mut items = Vec::with_capacity(len);
            for _ in 0..len {
                let key = parse_kryo_basic(&data[reader.pos..])?;
                reader.pos += estimate_kryo_value_size(&key);
                let value = parse_kryo_basic(&data[reader.pos..])?;
                reader.pos += estimate_kryo_value_size(&value);
                items.push((key, value));
            }
            Ok(KryoValue::Map(items))
        }
        _ => {
            let remaining = data.len().min(64);
            Ok(KryoValue::Unknown {
                type_id,
                data: data[..remaining].to_vec(),
                message: format!("未知的 Kryo 类型 ID: 0x{:02X}", type_id),
            })
        }
    }
}

fn estimate_kryo_value_size(value: &KryoValue) -> usize {
    match value {
        KryoValue::Null => 1,
        KryoValue::Byte(_) => 2,
        KryoValue::Char(_) => 3,
        KryoValue::Short(_) => 3,
        KryoValue::Int(_) => 5,
        KryoValue::Long(_) => 9,
        KryoValue::Float(_) => 5,
        KryoValue::Double(_) => 9,
        KryoValue::Boolean(_) => 1,
        KryoValue::String(s) => 1 + s.as_ref().map(|s| s.len()).unwrap_or(0) + 1,
        KryoValue::ByteArray(v) => 1 + v.len(),
        KryoValue::Unknown { data, .. } => data.len(),
        _ => 1,
    }
}

pub fn kryo_to_json(kryo: KryoValue) -> JsonValue {
    match kryo {
        KryoValue::Null => JsonValue::Null,
        KryoValue::Byte(b) => JsonValue::Number(b.into()),
        KryoValue::Char(c) => JsonValue::String(c.to_string()),
        KryoValue::Short(s) => JsonValue::Number(s.into()),
        KryoValue::Int(i) => JsonValue::Number(i.into()),
        KryoValue::Long(l) => JsonValue::Number(l.into()),
        KryoValue::Float(f) => serde_json::Number::from_f64(f as f64)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        KryoValue::Double(d) => serde_json::Number::from_f64(d)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        KryoValue::Boolean(b) => JsonValue::Bool(b),
        KryoValue::String(s) => JsonValue::String(s.unwrap_or_else(|| "<null string>".to_string())),
        KryoValue::ByteArray(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            JsonValue::String(format!("[{} bytes] {}", bytes.len(), hex))
        }
        KryoValue::IntArray(arr) => JsonValue::Array(
            arr.into_iter()
                .map(|i| JsonValue::Number(i.into()))
                .collect(),
        ),
        KryoValue::LongArray(arr) => JsonValue::Array(
            arr.into_iter()
                .map(|l| JsonValue::Number(l.into()))
                .collect(),
        ),
        KryoValue::List(items) => JsonValue::Array(items.into_iter().map(kryo_to_json).collect()),
        KryoValue::Map(items) => JsonValue::Object(
            items
                .into_iter()
                .filter_map(|(k, v)| {
                    let key = match k {
                        KryoValue::String(Some(s)) => s,
                        KryoValue::Int(i) => i.to_string(),
                        _ => return None,
                    };
                    Some((key, kryo_to_json(v)))
                })
                .collect(),
        ),
        KryoValue::Unknown {
            type_id,
            data,
            message,
        } => {
            let mut map = Map::new();
            map.insert("__type_id__".to_string(), JsonValue::Number(type_id.into()));
            map.insert("__message__".to_string(), JsonValue::String(message));
            let hex: String = data
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            map.insert("__hex__".to_string(), JsonValue::String(hex));
            JsonValue::Object(map)
        }
    }
}

pub fn parse_kryo_to_json(data: &[u8]) -> Result<String, String> {
    let value = parse_kryo_basic(data)?;
    let json = kryo_to_json(value);
    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_kryo() {
        assert!(is_kryo_serialization(&[0x01, 0x42]));
        assert!(is_kryo_serialization(&[0x04, 0x7F]));
        assert!(is_kryo_serialization(&[0x08, 0x00]));
        assert!(is_kryo_serialization(&[0x09, 0x00]));
        assert!(!is_kryo_serialization(&[0xFF, 0xFF]));
    }

    #[test]
    fn test_detect_fst() {
        assert!(is_fst_serialization(&[0xF0, 0x00, 0x00, 0x00]));
        assert!(is_fst_serialization(&[0xF1, 0x00, 0x00, 0x00]));
        assert!(!is_fst_serialization(&[0x00, 0x00, 0x00, 0x00]));
    }

    #[test]
    fn test_parse_kryo_null() {
        let result = parse_kryo_basic(&[0x00]).unwrap();
        assert!(matches!(result, KryoValue::Null));
    }

    #[test]
    fn test_parse_kryo_boolean() {
        let result = parse_kryo_basic(&[0x08]).unwrap();
        assert!(matches!(result, KryoValue::Boolean(true)));

        let result = parse_kryo_basic(&[0x09]).unwrap();
        assert!(matches!(result, KryoValue::Boolean(false)));
    }

    #[test]
    fn test_parse_kryo_byte() {
        let result = parse_kryo_basic(&[0x01, 0x42]).unwrap();
        assert!(matches!(result, KryoValue::Byte(0x42)));
    }

    #[test]
    fn test_kryo_to_json() {
        let json = kryo_to_json(KryoValue::Int(42));
        assert_eq!(json, JsonValue::Number(42.into()));

        let json = kryo_to_json(KryoValue::String(Some("hello".to_string())));
        assert_eq!(json, JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_parse_fst_string() {
        let data: Vec<u8> = vec![
            0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, b't', b'e', b's', b't',
        ];
        let result = parse_kryo_basic(&data).unwrap();
        assert!(matches!(result, KryoValue::String(Some(ref s)) if s == "test"));
    }
}
