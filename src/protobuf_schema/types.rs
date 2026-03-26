#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Double,
    Float,
    Int32,
    Int64,
    UInt32,
    UInt64,
    SInt32,
    SInt64,
    Fixed32,
    Fixed64,
    SFixed32,
    SFixed64,
    Bool,
    String,
    Bytes,
    Message(String),
    Enum(String),
}

impl FieldType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "double" => FieldType::Double,
            "float" => FieldType::Float,
            "int32" => FieldType::Int32,
            "int64" => FieldType::Int64,
            "uint32" => FieldType::UInt32,
            "uint64" => FieldType::UInt64,
            "sint32" => FieldType::SInt32,
            "sint64" => FieldType::SInt64,
            "fixed32" => FieldType::Fixed32,
            "fixed64" => FieldType::Fixed64,
            "sfixed32" => FieldType::SFixed32,
            "sfixed64" => FieldType::SFixed64,
            "bool" => FieldType::Bool,
            "string" => FieldType::String,
            "bytes" => FieldType::Bytes,
            other => {
                if other.starts_with(char::is_uppercase) {
                    FieldType::Message(other.to_string())
                } else {
                    FieldType::Enum(other.to_string())
                }
            }
        }
    }

    pub fn wire_type(&self) -> u32 {
        match self {
            FieldType::Double | FieldType::Fixed64 | FieldType::SFixed64 => 1,
            FieldType::Float | FieldType::Fixed32 | FieldType::SFixed32 => 5,
            FieldType::Int32
            | FieldType::Int64
            | FieldType::UInt32
            | FieldType::UInt64
            | FieldType::SInt32
            | FieldType::SInt64
            | FieldType::Bool
            | FieldType::Enum(_) => 0,
            FieldType::String | FieldType::Bytes | FieldType::Message(_) => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldLabel {
    Optional,
    Required,
    Repeated,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub number: u32,
    pub field_type: FieldType,
    pub label: FieldLabel,
}

#[derive(Debug, Clone)]
pub struct MessageDef {
    pub name: String,
    pub full_name: String,
    pub fields: Vec<FieldDef>,
    pub nested_messages: Vec<MessageDef>,
}

impl MessageDef {
    pub fn get_field(&self, field_number: u32) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.number == field_number)
    }
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub values: Vec<(String, i32)>,
}

#[derive(Debug, Clone)]
pub struct ProtoFile {
    pub package: String,
    pub messages: Vec<MessageDef>,
    pub enums: Vec<EnumDef>,
    pub imports: Vec<String>,
}
