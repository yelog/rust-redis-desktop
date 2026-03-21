use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JavaTypeCode {
    Byte,
    Char,
    Double,
    Float,
    Integer,
    Long,
    Short,
    Boolean,
    Array,
    Object,
}

impl JavaTypeCode {
    pub fn from_code(code: char) -> Option<Self> {
        match code {
            'B' => Some(JavaTypeCode::Byte),
            'C' => Some(JavaTypeCode::Char),
            'D' => Some(JavaTypeCode::Double),
            'F' => Some(JavaTypeCode::Float),
            'I' => Some(JavaTypeCode::Integer),
            'J' => Some(JavaTypeCode::Long),
            'S' => Some(JavaTypeCode::Short),
            'Z' => Some(JavaTypeCode::Boolean),
            '[' => Some(JavaTypeCode::Array),
            'L' => Some(JavaTypeCode::Object),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            JavaTypeCode::Byte => "byte",
            JavaTypeCode::Char => "char",
            JavaTypeCode::Double => "double",
            JavaTypeCode::Float => "float",
            JavaTypeCode::Integer => "int",
            JavaTypeCode::Long => "long",
            JavaTypeCode::Short => "short",
            JavaTypeCode::Boolean => "boolean",
            JavaTypeCode::Array => "array",
            JavaTypeCode::Object => "object",
        }
    }
}

impl fmt::Display for JavaTypeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaFieldInfo {
    pub name: String,
    pub type_code: JavaTypeCode,
    pub type_string: String,
    pub class_name: Option<String>,
}

impl JavaFieldInfo {
    pub fn display_type(&self) -> String {
        if let Some(ref class_name) = self.class_name {
            simplify_class_name(class_name)
        } else {
            match self.type_code {
                JavaTypeCode::Array => {
                    if let Some(ref class_name) = self.class_name {
                        format!("{}[]", simplify_class_name(class_name))
                    } else {
                        self.type_string.clone()
                    }
                }
                _ => self.type_code.type_name().to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaClassInfo {
    pub class_name: String,
    pub serial_version_uid: u64,
    pub fields: Vec<JavaFieldInfo>,
    pub super_class: Option<Box<JavaClassInfo>>,
}

impl JavaClassInfo {
    pub fn simple_class_name(&self) -> String {
        simplify_class_name(&self.class_name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaSerializationInfo {
    pub root_class: JavaClassInfo,
    pub object_type: JavaObjectType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JavaObjectType {
    Object,
    Array,
    String,
    Enum,
}

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

pub fn format_class_name_with_package(full_name: &str) -> String {
    let simplified = simplify_class_name(full_name);
    if simplified == full_name {
        simplified
    } else {
        format!("{} ({})", simplified, full_name)
    }
}
