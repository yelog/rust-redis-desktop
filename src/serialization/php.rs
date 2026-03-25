use serde_json::{Map, Value as JsonValue};

#[derive(Debug, Clone, PartialEq)]
pub enum PhpValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<(PhpValue, PhpValue)>),
    Object {
        class: String,
        properties: Vec<(String, PhpValue)>,
    },
    Reference(i32),
}

pub fn is_php_serialization(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let first_char = data[0] as char;

    matches!(
        first_char,
        'a' | 'O' | 's' | 'i' | 'b' | 'd' | 'N' | 'r' | 'R' | 'C'
    ) && data.len() > 1
        && (data[1] as char == ':' || first_char == 'N')
}

pub fn parse_php_serialization(data: &[u8]) -> Result<PhpValue, String> {
    let input = std::str::from_utf8(data).map_err(|_| "无效的 UTF-8 数据".to_string())?;
    let (value, remaining) = parse_php_value(input)?;
    if !remaining.trim().is_empty() {
        tracing::warn!("PHP 解析后剩余数据: {}", remaining);
    }
    Ok(value)
}

fn parse_php_value(input: &str) -> Result<(PhpValue, &str), String> {
    let input = input.trim_start();

    if input.is_empty() {
        return Err("空输入".to_string());
    }

    let type_char = input.chars().next().unwrap();
    let rest = &input[1..];

    match type_char {
        'N' => {
            if rest.starts_with(';') {
                Ok((PhpValue::Null, &rest[1..]))
            } else {
                Err("无效的 null 格式".to_string())
            }
        }
        'b' => parse_php_bool(rest),
        'i' => parse_php_int(rest),
        'd' => parse_php_float(rest),
        's' => parse_php_string(rest),
        'a' => parse_php_array(rest),
        'O' => parse_php_object(rest),
        'r' | 'R' => parse_php_reference(rest),
        'C' => parse_php_custom_object(rest),
        _ => Err(format!("未知的类型标识符: '{}'", type_char)),
    }
}

fn parse_php_bool(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("布尔值缺少冒号".to_string());
    }
    let rest = &rest[1..];

    if rest.starts_with("0;") {
        Ok((PhpValue::Bool(false), &rest[2..]))
    } else if rest.starts_with("1;") {
        Ok((PhpValue::Bool(true), &rest[2..]))
    } else {
        Err("无效的布尔值".to_string())
    }
}

fn parse_php_int(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("整数缺少冒号".to_string());
    }
    let rest = &rest[1..];

    let end_pos = rest.find(';').ok_or("整数缺少分号".to_string())?;
    let num_str = &rest[..end_pos];
    let num: i64 = num_str
        .parse()
        .map_err(|_| format!("无效的整数: {}", num_str))?;

    Ok((PhpValue::Int(num), &rest[end_pos + 1..]))
}

fn parse_php_float(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("浮点数缺少冒号".to_string());
    }
    let rest = &rest[1..];

    let end_pos = rest.find(';').ok_or("浮点数缺少分号".to_string())?;
    let num_str = &rest[..end_pos];
    let num: f64 = num_str
        .parse()
        .map_err(|_| format!("无效的浮点数: {}", num_str))?;

    Ok((PhpValue::Float(num), &rest[end_pos + 1..]))
}

fn parse_php_string(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("字符串缺少冒号".to_string());
    }
    let rest = &rest[1..];

    let colon_pos = rest.find(':').ok_or("字符串缺少第二个冒号".to_string())?;
    let len: usize = rest[..colon_pos]
        .parse()
        .map_err(|_| format!("无效的字符串长度: {}", &rest[..colon_pos]))?;

    let rest = &rest[colon_pos + 1..];

    if !rest.starts_with('"') {
        return Err("字符串缺少起始引号".to_string());
    }
    let rest = &rest[1..];

    let byte_slice = rest.as_bytes();
    if byte_slice.len() < len + 1 {
        return Err(format!(
            "字符串数据不完整: 需要 {} 字节, 剩余 {} 字节",
            len,
            byte_slice.len()
        ));
    }

    let value =
        std::str::from_utf8(&byte_slice[..len]).map_err(|_| "无效的 UTF-8 字符串".to_string())?;
    let rest = std::str::from_utf8(&byte_slice[len..]).map_err(|_| "剩余数据无效".to_string())?;

    if !rest.starts_with("\";") {
        return Err("字符串缺少结束引号和分号".to_string());
    }

    Ok((PhpValue::String(value.to_string()), &rest[2..]))
}

fn parse_php_array(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("数组缺少冒号".to_string());
    }
    let rest = &rest[1..];

    let colon_pos = rest.find(':').ok_or("数组缺少第二个冒号".to_string())?;
    let count: usize = rest[..colon_pos]
        .parse()
        .map_err(|_| format!("无效的数组长度: {}", &rest[..colon_pos]))?;

    let rest = &rest[colon_pos + 1..];

    if !rest.starts_with('{') {
        return Err("数组缺少起始大括号".to_string());
    }
    let mut rest = &rest[1..];

    let mut items = Vec::with_capacity(count);

    for _ in 0..count {
        if rest.is_empty() {
            return Err("数组解析提前结束".to_string());
        }
        let (key, new_rest) = parse_php_value(rest)?;
        rest = new_rest;
        let (value, new_rest) = parse_php_value(rest)?;
        rest = new_rest;
        items.push((key, value));
    }

    if !rest.starts_with('}') {
        return Err("数组缺少结束大括号".to_string());
    }

    Ok((PhpValue::Array(items), &rest[1..]))
}

fn parse_php_object(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("对象缺少冒号".to_string());
    }
    let rest = &rest[1..];

    let colon_pos = rest.find(':').ok_or("对象缺少类名长度分隔符".to_string())?;
    let class_len: usize = rest[..colon_pos]
        .parse()
        .map_err(|_| format!("无效的类名长度: {}", &rest[..colon_pos]))?;

    let rest = &rest[colon_pos + 1..];

    if !rest.starts_with('"') {
        return Err("对象类名缺少起始引号".to_string());
    }
    let rest = &rest[1..];

    if rest.len() < class_len {
        return Err("类名数据不完整".to_string());
    }

    let class_name = &rest[..class_len];
    let rest = &rest[class_len..];

    if !rest.starts_with("\":") {
        return Err("对象类名缺少结束引号和冒号".to_string());
    }
    let rest = &rest[2..];

    let colon_pos = rest.find(':').ok_or("对象缺少属性计数分隔符".to_string())?;
    let prop_count: usize = rest[..colon_pos]
        .parse()
        .map_err(|_| format!("无效的属性计数: {}", &rest[..colon_pos]))?;

    let rest = &rest[colon_pos + 1..];

    if !rest.starts_with('{') {
        return Err("对象缺少起始大括号".to_string());
    }
    let mut rest = &rest[1..];

    let mut properties = Vec::with_capacity(prop_count);

    for _ in 0..prop_count {
        if rest.is_empty() {
            return Err("对象属性解析提前结束".to_string());
        }

        let (name_val, new_rest) = parse_php_value(rest)?;
        rest = new_rest;

        let name = match name_val {
            PhpValue::String(s) => clean_php_property_name(&s),
            PhpValue::Int(i) => i.to_string(),
            _ => return Err("属性名必须是字符串或整数".to_string()),
        };

        let (value, new_rest) = parse_php_value(rest)?;
        rest = new_rest;

        properties.push((name, value));
    }

    if !rest.starts_with('}') {
        return Err("对象缺少结束大括号".to_string());
    }

    Ok((
        PhpValue::Object {
            class: class_name.to_string(),
            properties,
        },
        &rest[1..],
    ))
}

fn parse_php_reference(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("引用缺少冒号".to_string());
    }
    let rest = &rest[1..];

    let end_pos = rest.find(';').ok_or("引用缺少分号".to_string())?;
    let num_str = &rest[..end_pos];
    let num: i32 = num_str
        .parse()
        .map_err(|_| format!("无效的引用编号: {}", num_str))?;

    Ok((PhpValue::Reference(num), &rest[end_pos + 1..]))
}

fn parse_php_custom_object(rest: &str) -> Result<(PhpValue, &str), String> {
    if !rest.starts_with(':') {
        return Err("自定义对象缺少冒号".to_string());
    }
    let rest = &rest[1..];

    let colon_pos = rest
        .find(':')
        .ok_or("自定义对象缺少类名长度分隔符".to_string())?;
    let class_len: usize = rest[..colon_pos]
        .parse()
        .map_err(|_| format!("无效的类名长度: {}", &rest[..colon_pos]))?;

    let rest = &rest[colon_pos + 1..];

    if !rest.starts_with('"') {
        return Err("自定义对象类名缺少起始引号".to_string());
    }
    let rest = &rest[1..];

    if rest.len() < class_len {
        return Err("类名数据不完整".to_string());
    }

    let class_name = &rest[..class_len];
    let rest = &rest[class_len..];

    if !rest.starts_with("\":") {
        return Err("自定义对象类名缺少结束引号和冒号".to_string());
    }
    let rest = &rest[2..];

    let colon_pos = rest
        .find(':')
        .ok_or("自定义对象缺少数据长度分隔符".to_string())?;
    let data_len: usize = rest[..colon_pos]
        .parse()
        .map_err(|_| format!("无效的数据长度: {}", &rest[..colon_pos]))?;

    let rest = &rest[colon_pos + 1..];

    if !rest.starts_with('{') {
        return Err("自定义对象缺少起始大括号".to_string());
    }
    let rest = &rest[1..];

    if rest.len() < data_len {
        return Err("自定义对象数据不完整".to_string());
    }

    let data = &rest[..data_len];
    let rest = &rest[data_len..];

    if !rest.starts_with('}') {
        return Err("自定义对象缺少结束大括号".to_string());
    }

    Ok((
        PhpValue::Object {
            class: class_name.to_string(),
            properties: vec![(
                "__serialized_data__".to_string(),
                PhpValue::String(data.to_string()),
            )],
        },
        &rest[1..],
    ))
}

fn clean_php_property_name(name: &str) -> String {
    if name.starts_with('\0') {
        let parts: Vec<&str> = name.split('\0').collect();
        if parts.len() == 3 {
            return format!("{}::{}", parts[1], parts[2]);
        }
    }
    name.to_string()
}

pub fn php_to_json(php: PhpValue) -> JsonValue {
    match php {
        PhpValue::Null => JsonValue::Null,
        PhpValue::Bool(b) => JsonValue::Bool(b),
        PhpValue::Int(i) => JsonValue::Number(i.into()),
        PhpValue::Float(f) => serde_json::Number::from_f64(f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        PhpValue::String(s) => JsonValue::String(s),
        PhpValue::Reference(n) => {
            let mut map = Map::new();
            map.insert("__reference__".to_string(), JsonValue::Number(n.into()));
            JsonValue::Object(map)
        }
        PhpValue::Array(items) => {
            if items.is_empty() {
                return JsonValue::Array(vec![]);
            }

            let is_indexed = items
                .iter()
                .enumerate()
                .all(|(i, (k, _))| matches!(k, PhpValue::Int(n) if *n >= 0 && *n as usize == i));

            if is_indexed {
                JsonValue::Array(items.into_iter().map(|(_, v)| php_to_json(v)).collect())
            } else {
                JsonValue::Object(
                    items
                        .into_iter()
                        .filter_map(|(k, v)| {
                            let key = match k {
                                PhpValue::String(s) => s,
                                PhpValue::Int(i) => i.to_string(),
                                _ => return None,
                            };
                            Some((key, php_to_json(v)))
                        })
                        .collect(),
                )
            }
        }
        PhpValue::Object { class, properties } => {
            let mut map = Map::new();
            map.insert("__class__".to_string(), JsonValue::String(class));
            for (key, value) in properties {
                map.insert(key, php_to_json(value));
            }
            JsonValue::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_php() {
        assert!(is_php_serialization(b"a:0:{}"));
        assert!(is_php_serialization(b"O:8:\"stdClass\":0:{}"));
        assert!(is_php_serialization(b"s:5:\"hello\";"));
        assert!(is_php_serialization(b"i:42;"));
        assert!(is_php_serialization(b"N;"));
        assert!(!is_php_serialization(b"hello world"));
        assert!(!is_php_serialization(b"{\"json\": true}"));
    }

    #[test]
    fn test_parse_null() {
        let result = parse_php_serialization(b"N;").unwrap();
        assert_eq!(result, PhpValue::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(
            parse_php_serialization(b"b:1;").unwrap(),
            PhpValue::Bool(true)
        );
        assert_eq!(
            parse_php_serialization(b"b:0;").unwrap(),
            PhpValue::Bool(false)
        );
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(
            parse_php_serialization(b"i:42;").unwrap(),
            PhpValue::Int(42)
        );
        assert_eq!(
            parse_php_serialization(b"i:-123;").unwrap(),
            PhpValue::Int(-123)
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_php_serialization(b"s:5:\"hello\";").unwrap(),
            PhpValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_indexed_array() {
        let result =
            parse_php_serialization(b"a:3:{i:0;s:3:\"one\";i:1;s:3:\"two\";i:2;s:5:\"three\";}")
                .unwrap();
        let json = php_to_json(result);
        assert_eq!(json, serde_json::json!(["one", "two", "three"]));
    }

    #[test]
    fn test_parse_associative_array() {
        let result =
            parse_php_serialization(b"a:2:{s:3:\"foo\";s:3:\"bar\";s:3:\"baz\";i:42;}").unwrap();
        let json = php_to_json(result);
        assert_eq!(json, serde_json::json!({"foo": "bar", "baz": 42}));
    }

    #[test]
    fn test_parse_object() {
        let result =
            parse_php_serialization(b"O:8:\"stdClass\":1:{s:3:\"foo\";s:3:\"bar\";}").unwrap();
        let json = php_to_json(result);
        assert_eq!(json["__class__"], serde_json::json!("stdClass"));
        assert_eq!(json["foo"], serde_json::json!("bar"));
    }

    #[test]
    fn test_parse_nested_array() {
        let data = b"a:2:{s:4:\"user\";a:3:{s:2:\"id\";i:1;s:4:\"name\";s:4:\"John\";s:5:\"email\";s:13:\"john@test.com\";}s:6:\"status\";s:6:\"active\";}";
        let result = parse_php_serialization(data);
        match &result {
            Ok(val) => {
                let json = php_to_json(val.clone());
                println!(
                    "Parsed JSON: {}",
                    serde_json::to_string_pretty(&json).unwrap()
                );
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        assert!(result.is_ok());
    }
}
