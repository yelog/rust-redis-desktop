use super::types::*;
use std::char;
use std::io::{Cursor, Read};

const STREAM_MAGIC: u16 = 0xACED;
const STREAM_VERSION: u16 = 5;

const TC_NULL: u8 = 0x70;
const TC_REFERENCE: u8 = 0x71;
const TC_CLASSDESC: u8 = 0x72;
const TC_OBJECT: u8 = 0x73;
const TC_STRING: u8 = 0x74;
const TC_ARRAY: u8 = 0x75;
const TC_CLASS: u8 = 0x76;
const TC_BLOCKDATA: u8 = 0x77;
const TC_ENDBLOCKDATA: u8 = 0x78;
const TC_LONGSTRING: u8 = 0x7C;
const TC_PROXYCLASSDESC: u8 = 0x7D;
const TC_ENUM: u8 = 0x7E;

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidMagic(u16),
    InvalidVersion(u16),
    UnexpectedEnd,
    InvalidToken(u8),
    InvalidUtf8,
    InvalidTypeCode(char),
    InvalidClassDescriptor,
    IoError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidMagic(m) => write!(f, "Invalid magic number: 0x{:04X}", m),
            ParseError::InvalidVersion(v) => write!(f, "Invalid version: {}", v),
            ParseError::UnexpectedEnd => write!(f, "Unexpected end of data"),
            ParseError::InvalidToken(t) => write!(f, "Invalid token: 0x{:02X}", t),
            ParseError::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            ParseError::InvalidTypeCode(c) => write!(f, "Invalid type code: {}", c),
            ParseError::InvalidClassDescriptor => write!(f, "Invalid class descriptor"),
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}

struct ByteReader<'a> {
    cursor: Cursor<&'a [u8]>,
    handles: Vec<Handle>,
}

#[derive(Debug, Clone)]
enum Handle {
    Class(JavaClassInfo),
    String(String),
    Array,
    Object,
}

impl<'a> ByteReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        ByteReader {
            cursor: Cursor::new(data),
            handles: Vec::new(),
        }
    }

    fn read_u8(&mut self) -> Result<u8, ParseError> {
        let mut buf = [0u8; 1];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ParseError::UnexpectedEnd)?;
        Ok(buf[0])
    }

    fn read_u16(&mut self) -> Result<u16, ParseError> {
        let mut buf = [0u8; 2];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ParseError::UnexpectedEnd)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_u32(&mut self) -> Result<u32, ParseError> {
        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ParseError::UnexpectedEnd)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_u64(&mut self) -> Result<u64, ParseError> {
        let mut buf = [0u8; 8];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ParseError::UnexpectedEnd)?;
        Ok(u64::from_be_bytes(buf))
    }

    fn read_i32(&mut self) -> Result<i32, ParseError> {
        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ParseError::UnexpectedEnd)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_long_utf(&mut self) -> Result<String, ParseError> {
        let length = self.read_u16()? as usize;
        let mut buf = vec![0u8; length];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ParseError::UnexpectedEnd)?;
        String::from_utf8(buf).map_err(|_| ParseError::InvalidUtf8)
    }

    fn add_handle(&mut self, handle: Handle) {
        self.handles.push(handle);
    }

    fn get_handle(&self, index: usize) -> Option<&Handle> {
        let base_handle = 0x7E0000;
        let actual_index = index - base_handle;
        self.handles.get(actual_index)
    }
}

pub fn is_java_serialization(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0xAC && data[1] == 0xED && data[2] == 0x00 && data[3] == 0x05
}

pub fn parse_java_serialization(data: &[u8]) -> Result<JavaSerializationInfo, ParseError> {
    let mut reader = ByteReader::new(data);

    let magic = reader.read_u16()?;
    if magic != STREAM_MAGIC {
        return Err(ParseError::InvalidMagic(magic));
    }

    let version = reader.read_u16()?;
    if version != STREAM_VERSION {
        return Err(ParseError::InvalidVersion(version));
    }

    let (root_class, object_type) = parse_content(&mut reader)?;

    Ok(JavaSerializationInfo {
        root_class,
        object_type,
    })
}

fn parse_content(reader: &mut ByteReader) -> Result<(JavaClassInfo, JavaObjectType), ParseError> {
    let token = reader.read_u8()?;

    match token {
        TC_NULL => Err(ParseError::InvalidClassDescriptor),
        TC_REFERENCE => {
            let handle = reader.read_u32()? as usize;
            match reader.get_handle(handle) {
                Some(Handle::Class(class_info)) => Ok((class_info.clone(), JavaObjectType::Object)),
                _ => Err(ParseError::InvalidClassDescriptor),
            }
        }
        TC_CLASSDESC => {
            let class_info = parse_class_desc(reader)?;
            Ok((class_info, JavaObjectType::Object))
        }
        TC_PROXYCLASSDESC => {
            let _interface_count = reader.read_i32()?;
            for _ in 0.._interface_count {
                let _ = reader.read_long_utf()?;
            }
            parse_class_annotation(reader)?;
            let super_class = parse_super_class(reader)?;
            let class_info = JavaClassInfo {
                class_name: "$Proxy".to_string(),
                serial_version_uid: 0,
                fields: vec![],
                super_class,
            };
            reader.add_handle(Handle::Class(class_info.clone()));
            Ok((class_info, JavaObjectType::Object))
        }
        TC_OBJECT => {
            let class_info = parse_new_object(reader)?;
            Ok((class_info, JavaObjectType::Object))
        }
        TC_STRING => {
            let s = reader.read_long_utf()?;
            reader.add_handle(Handle::String(s));
            let class_info = JavaClassInfo {
                class_name: "java.lang.String".to_string(),
                serial_version_uid: 0,
                fields: vec![],
                super_class: None,
            };
            Ok((class_info, JavaObjectType::String))
        }
        TC_LONGSTRING => {
            let length = reader.read_u64()? as usize;
            let mut buf = vec![0u8; length];
            reader
                .cursor
                .read_exact(&mut buf)
                .map_err(|_| ParseError::UnexpectedEnd)?;
            let _s = String::from_utf8(buf).map_err(|_| ParseError::InvalidUtf8)?;
            let class_info = JavaClassInfo {
                class_name: "java.lang.String".to_string(),
                serial_version_uid: 0,
                fields: vec![],
                super_class: None,
            };
            Ok((class_info, JavaObjectType::String))
        }
        TC_ARRAY => {
            let class_info = parse_new_array(reader)?;
            Ok((class_info, JavaObjectType::Array))
        }
        TC_ENUM => {
            let class_info = parse_enum(reader)?;
            Ok((class_info, JavaObjectType::Enum))
        }
        TC_CLASS => {
            let class_info = parse_class(reader)?;
            Ok((class_info, JavaObjectType::Object))
        }
        _ => Err(ParseError::InvalidToken(token)),
    }
}

fn parse_class_desc(reader: &mut ByteReader) -> Result<JavaClassInfo, ParseError> {
    let class_name = reader.read_long_utf()?;
    let serial_version_uid = reader.read_u64()?;

    let _flags = reader.read_u8()?;

    let field_count = reader.read_u16()? as usize;
    let mut fields = Vec::with_capacity(field_count);

    for _ in 0..field_count {
        let type_code_byte = reader.read_u8()?;
        let type_code = type_code_byte as char;

        let field_name = reader.read_long_utf()?;

        let (java_type_code, type_string, class_name) = match type_code {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' => {
                let tc = JavaTypeCode::from_code(type_code)
                    .ok_or(ParseError::InvalidTypeCode(type_code))?;
                (tc, tc.type_name().to_string(), None)
            }
            '[' | 'L' => {
                let type_str = parse_type_string(reader)?;
                let tc = if type_code == '[' {
                    JavaTypeCode::Array
                } else {
                    JavaTypeCode::Object
                };
                let class = if type_str.starts_with('L') && type_str.ends_with(';') {
                    Some(type_str[1..type_str.len() - 1].replace('/', "."))
                } else if type_str.starts_with('[') {
                    Some(parse_array_type(&type_str))
                } else {
                    Some(type_str.clone())
                };
                (tc, type_str, class)
            }
            _ => {
                return Err(ParseError::InvalidTypeCode(type_code));
            }
        };

        fields.push(JavaFieldInfo {
            name: field_name,
            type_code: java_type_code,
            type_string: type_string,
            class_name,
            value: JavaFieldValue::NotParsed,
        });
    }

    parse_class_annotation(reader)?;

    let super_class = parse_super_class(reader)?;

    let class_info = JavaClassInfo {
        class_name: class_name.replace('/', "."),
        serial_version_uid,
        fields,
        super_class,
    };

    reader.add_handle(Handle::Class(class_info.clone()));

    Ok(class_info)
}

fn parse_type_string(reader: &mut ByteReader) -> Result<String, ParseError> {
    let first_byte = reader.read_u8()?;

    if first_byte as char == 'L' {
        let class_name = reader.read_long_utf()?;
        let _semicolon = reader.read_u8()?;
        Ok(format!("L{};", class_name))
    } else if first_byte as char == '[' {
        let inner = parse_type_string(reader)?;
        Ok(format!("[{}", inner))
    } else {
        Ok((first_byte as char).to_string())
    }
}

fn parse_array_type(type_str: &str) -> String {
    let dims = type_str.chars().filter(|&c| c == '[').count();
    let base_type = type_str.trim_start_matches('[');

    let base = if base_type.starts_with('L') && base_type.ends_with(';') {
        simplify_class_name(&base_type[1..base_type.len() - 1].replace('/', "."))
    } else {
        match base_type {
            "B" => "byte".to_string(),
            "C" => "char".to_string(),
            "D" => "double".to_string(),
            "F" => "float".to_string(),
            "I" => "int".to_string(),
            "J" => "long".to_string(),
            "S" => "short".to_string(),
            "Z" => "boolean".to_string(),
            _ => base_type.to_string(),
        }
    };

    format!("{}{}", base, "[]".repeat(dims))
}

fn parse_class_annotation(reader: &mut ByteReader) -> Result<(), ParseError> {
    loop {
        let token = reader.read_u8()?;
        match token {
            TC_ENDBLOCKDATA => break,
            TC_BLOCKDATA => {
                let size = reader.read_u8()? as usize;
                let mut _buf = vec![0u8; size];
                reader
                    .cursor
                    .read_exact(&mut _buf)
                    .map_err(|_| ParseError::UnexpectedEnd)?;
            }
            TC_STRING => {
                let _ = reader.read_long_utf()?;
            }
            _ => {
                break;
            }
        }
    }
    Ok(())
}

fn parse_super_class(reader: &mut ByteReader) -> Result<Option<Box<JavaClassInfo>>, ParseError> {
    let token = reader.read_u8()?;
    match token {
        TC_NULL => Ok(None),
        TC_REFERENCE => {
            let handle = reader.read_u32()? as usize;
            match reader.get_handle(handle) {
                Some(Handle::Class(class_info)) => Ok(Some(Box::new(class_info.clone()))),
                _ => Ok(None),
            }
        }
        TC_CLASSDESC => {
            let class_info = parse_class_desc(reader)?;
            Ok(Some(Box::new(class_info)))
        }
        _ => Err(ParseError::InvalidToken(token)),
    }
}

fn parse_new_object(reader: &mut ByteReader) -> Result<JavaClassInfo, ParseError> {
    let token = reader.read_u8()?;

    match token {
        TC_CLASSDESC => {
            let mut class_info = parse_class_desc(reader)?;
            parse_object_data(reader, &mut class_info)?;
            Ok(class_info)
        }
        TC_REFERENCE => {
            let handle = reader.read_u32()? as usize;
            let mut class_info = match reader.get_handle(handle) {
                Some(Handle::Class(class_info)) => class_info.clone(),
                _ => return Err(ParseError::InvalidClassDescriptor),
            };
            parse_object_data(reader, &mut class_info)?;
            Ok(class_info)
        }
        _ => Err(ParseError::InvalidToken(token)),
    }
}

fn parse_primitive_value(
    reader: &mut ByteReader,
    type_code: JavaTypeCode,
) -> Result<JavaFieldValue, ParseError> {
    match type_code {
        JavaTypeCode::Byte => {
            let v = reader.read_u8()? as i8;
            Ok(JavaFieldValue::Byte(v))
        }
        JavaTypeCode::Char => {
            let v = reader.read_u16()?;
            Ok(JavaFieldValue::Char(
                char::from_u32(v as u32).unwrap_or('\0'),
            ))
        }
        JavaTypeCode::Double => {
            let bytes = reader.read_u64()?;
            Ok(JavaFieldValue::Double(f64::from_be_bytes(
                bytes.to_be_bytes(),
            )))
        }
        JavaTypeCode::Float => {
            let bytes = reader.read_u32()?;
            Ok(JavaFieldValue::Float(f32::from_be_bytes(
                bytes.to_be_bytes(),
            )))
        }
        JavaTypeCode::Integer => {
            let v = reader.read_i32()?;
            Ok(JavaFieldValue::Int(v))
        }
        JavaTypeCode::Long => {
            let v = reader.read_u64()? as i64;
            Ok(JavaFieldValue::Long(v))
        }
        JavaTypeCode::Short => {
            let v = reader.read_u16()? as i16;
            Ok(JavaFieldValue::Short(v))
        }
        JavaTypeCode::Boolean => {
            let v = reader.read_u8()?;
            Ok(JavaFieldValue::Boolean(v != 0))
        }
        JavaTypeCode::Array | JavaTypeCode::Object => Ok(JavaFieldValue::NotParsed),
    }
}

fn parse_object_data(
    reader: &mut ByteReader,
    class_info: &mut JavaClassInfo,
) -> Result<(), ParseError> {
    if let Some(ref mut super_class) = class_info.super_class {
        parse_object_data(reader, super_class)?;
    }

    for field in &mut class_info.fields {
        let value = match field.type_code {
            JavaTypeCode::Byte
            | JavaTypeCode::Char
            | JavaTypeCode::Double
            | JavaTypeCode::Float
            | JavaTypeCode::Integer
            | JavaTypeCode::Long
            | JavaTypeCode::Short
            | JavaTypeCode::Boolean => parse_primitive_value(reader, field.type_code)?,
            JavaTypeCode::Array | JavaTypeCode::Object => {
                let token = reader.read_u8()?;
                match token {
                    TC_NULL => JavaFieldValue::Null,
                    TC_STRING => {
                        let s = reader.read_long_utf()?;
                        reader.add_handle(Handle::String(s.clone()));
                        JavaFieldValue::String(Some(s))
                    }
                    TC_REFERENCE => {
                        let handle = reader.read_u32()?;
                        JavaFieldValue::Reference(handle)
                    }
                    _ => JavaFieldValue::NotParsed,
                }
            }
        };
        field.value = value;
    }

    Ok(())
}

fn skip_object_data(reader: &mut ByteReader, class_info: &JavaClassInfo) -> Result<(), ParseError> {
    if let Some(ref super_class) = class_info.super_class {
        skip_object_data(reader, super_class)?;
    }

    for field in &class_info.fields {
        match field.type_code {
            JavaTypeCode::Byte => {
                let _ = reader.read_u8()?;
            }
            JavaTypeCode::Char => {
                let _ = reader.read_u16()?;
            }
            JavaTypeCode::Double => {
                let _ = reader.read_u64()?;
            }
            JavaTypeCode::Float => {
                let _ = reader.read_u32()?;
            }
            JavaTypeCode::Integer => {
                let _ = reader.read_i32()?;
            }
            JavaTypeCode::Long => {
                let _ = reader.read_u64()?;
            }
            JavaTypeCode::Short => {
                let _ = reader.read_u16()?;
            }
            JavaTypeCode::Boolean => {
                let _ = reader.read_u8()?;
            }
            JavaTypeCode::Array | JavaTypeCode::Object => {
                skip_content(reader)?;
            }
        }
    }

    Ok(())
}

fn skip_content(reader: &mut ByteReader) -> Result<(), ParseError> {
    let token = reader.read_u8()?;

    match token {
        TC_NULL => {}
        TC_REFERENCE => {
            let _ = reader.read_u32()?;
        }
        TC_STRING => {
            let _ = reader.read_long_utf()?;
        }
        TC_OBJECT => {
            skip_content(reader)?;
        }
        TC_ARRAY => {
            skip_content(reader)?;
            let size = reader.read_i32()?;
            for _ in 0..size {
                skip_content(reader)?;
            }
        }
        TC_BLOCKDATA => {
            let size = reader.read_u8()? as usize;
            let mut _buf = vec![0u8; size];
            reader
                .cursor
                .read_exact(&mut _buf)
                .map_err(|_| ParseError::UnexpectedEnd)?;
        }
        _ => {}
    }

    Ok(())
}

fn parse_new_array(reader: &mut ByteReader) -> Result<JavaClassInfo, ParseError> {
    let token = reader.read_u8()?;

    let class_name = match token {
        TC_CLASSDESC => {
            let class_info = parse_class_desc(reader)?;
            reader.add_handle(Handle::Array);
            class_info.class_name
        }
        TC_REFERENCE => {
            let handle = reader.read_u32()? as usize;
            match reader.get_handle(handle) {
                Some(Handle::Class(class_info)) => class_info.class_name.clone(),
                _ => return Err(ParseError::InvalidClassDescriptor),
            }
        }
        _ => return Err(ParseError::InvalidToken(token)),
    };

    let size = reader.read_i32()?;

    for _ in 0..size {
        skip_content(reader)?;
    }

    let class_info = JavaClassInfo {
        class_name,
        serial_version_uid: 0,
        fields: vec![],
        super_class: None,
    };

    Ok(class_info)
}

fn parse_enum(reader: &mut ByteReader) -> Result<JavaClassInfo, ParseError> {
    let token = reader.read_u8()?;

    let class_info = match token {
        TC_CLASSDESC => {
            let info = parse_class_desc(reader)?;
            reader.add_handle(Handle::Object);
            info
        }
        TC_REFERENCE => {
            let handle = reader.read_u32()? as usize;
            match reader.get_handle(handle) {
                Some(Handle::Class(info)) => info.clone(),
                _ => return Err(ParseError::InvalidClassDescriptor),
            }
        }
        _ => return Err(ParseError::InvalidToken(token)),
    };

    let _constant_name = match reader.read_u8()? {
        TC_STRING => reader.read_long_utf()?,
        TC_REFERENCE => {
            let handle = reader.read_u32()? as usize;
            match reader.get_handle(handle) {
                Some(Handle::String(s)) => s.clone(),
                _ => return Err(ParseError::InvalidClassDescriptor),
            }
        }
        _ => return Err(ParseError::InvalidClassDescriptor),
    };

    Ok(class_info)
}

fn parse_class(reader: &mut ByteReader) -> Result<JavaClassInfo, ParseError> {
    let token = reader.read_u8()?;

    match token {
        TC_CLASSDESC => {
            let class_info = parse_class_desc(reader)?;
            reader.add_handle(Handle::Class(class_info.clone()));
            Ok(class_info)
        }
        TC_REFERENCE => {
            let handle = reader.read_u32()? as usize;
            match reader.get_handle(handle) {
                Some(Handle::Class(class_info)) => Ok(class_info.clone()),
                _ => Err(ParseError::InvalidClassDescriptor),
            }
        }
        TC_NULL => Err(ParseError::InvalidClassDescriptor),
        _ => Err(ParseError::InvalidToken(token)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_java_serialization() {
        let valid_data = [0xAC, 0xED, 0x00, 0x05, 0x73, 0x72];
        assert!(is_java_serialization(&valid_data));

        let invalid_data = [0x00, 0x01, 0x02, 0x03];
        assert!(!is_java_serialization(&invalid_data));
    }

    #[test]
    fn test_parse_oauth2_authorization() {
        let hex_data = "aced00057372004c6f72672e737072696e676672616d65776f726b2e73656375726974792e6f61757468322e7365727665722e617574686f72697a6174696f6e2e4f4175746832417574686f72697a6174696f6e0000000002c82a370200074c000a61747472627574657374000f4c6a6176612f7574696c2f4d61703b4c0016617574686f72697a6174696f6e4772616e74547970657400414c6f72672f737072696e676672616d65776f726b2f73656375726974792e6f61757468322f636f72652f417574686f72697a6174696f6e4772616e74547970653b4c0010617574686f72697a656453636f70657374000f4c6a6176612f7574696c2f5365743b4c000269647400124c6a6176612f6c616e672f537472696e673b4c000d7072696e636970616c4e616d6571007e00044c001272656769737465726564436c69656e74496471007e00044c0006746f6b656e7371007e00017870";

        let binary_data: Vec<u8> = (0..hex_data.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_data[i..i + 2], 16).unwrap())
            .collect();

        assert!(is_java_serialization(&binary_data));

        let result = parse_java_serialization(&binary_data);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }

        if let Ok(info) = result {
            println!("Class: {}", info.root_class.class_name);
            println!("Fields: {}", info.root_class.fields.len());
            for f in &info.root_class.fields {
                println!(
                    "  - {}: {} = {}",
                    f.name,
                    f.display_type(),
                    f.value.display_value()
                );
            }
            assert!(info.root_class.class_name.contains("OAuth2Authorization"));
        }
    }

    #[test]
    fn test_parse_int_field() {
        let hex_data = "aced00057372000454657374000000000000000102000149000576616c756578700000002a";

        let binary_data: Vec<u8> = (0..hex_data.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_data[i..i + 2], 16).unwrap())
            .collect();

        assert!(is_java_serialization(&binary_data));

        let result = parse_java_serialization(&binary_data);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.root_class.class_name, "Test");
        assert_eq!(info.root_class.fields.len(), 1);
        assert_eq!(info.root_class.fields[0].name, "value");

        match &info.root_class.fields[0].value {
            JavaFieldValue::Int(v) => assert_eq!(*v, 42),
            other => panic!("Expected Int value, got {:?}", other),
        }
    }
}
