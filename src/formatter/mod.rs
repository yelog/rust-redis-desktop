pub mod custom;
pub mod preset;

pub use custom::{CustomFormatter, FormatterConfig, FormatterRegistry};
pub use preset::apply_preset_formatter;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatterType {
    Json,
    Hex,
    Base64,
    Base64Url,
    UrlEncode,
    Gzip,
    Zlib,
    Deflate,
    Brotli,
    Zstd,
    MsgPack,
    Protobuf,
    Yaml,
    Toml,
    Custom(String),
}

impl FormatterType {
    pub fn as_str(&self) -> &str {
        match self {
            FormatterType::Json => "json",
            FormatterType::Hex => "hex",
            FormatterType::Base64 => "base64",
            FormatterType::Base64Url => "base64url",
            FormatterType::UrlEncode => "urlencode",
            FormatterType::Gzip => "gzip",
            FormatterType::Zlib => "zlib",
            FormatterType::Deflate => "deflate",
            FormatterType::Brotli => "brotli",
            FormatterType::Zstd => "zstd",
            FormatterType::MsgPack => "msgpack",
            FormatterType::Protobuf => "protobuf",
            FormatterType::Yaml => "yaml",
            FormatterType::Toml => "toml",
            FormatterType::Custom(name) => name.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(FormatterType::Json),
            "hex" => Some(FormatterType::Hex),
            "base64" => Some(FormatterType::Base64),
            "base64url" => Some(FormatterType::Base64Url),
            "urlencode" => Some(FormatterType::UrlEncode),
            "gzip" => Some(FormatterType::Gzip),
            "zlib" => Some(FormatterType::Zlib),
            "deflate" => Some(FormatterType::Deflate),
            "brotli" => Some(FormatterType::Brotli),
            "zstd" => Some(FormatterType::Zstd),
            "msgpack" => Some(FormatterType::MsgPack),
            "protobuf" => Some(FormatterType::Protobuf),
            "yaml" => Some(FormatterType::Yaml),
            "toml" => Some(FormatterType::Toml),
            other => Some(FormatterType::Custom(other.to_string())),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            FormatterType::Json => "JSON",
            FormatterType::Hex => "Hex",
            FormatterType::Base64 => "Base64",
            FormatterType::Base64Url => "Base64 URL",
            FormatterType::UrlEncode => "URL Encode",
            FormatterType::Gzip => "Gzip",
            FormatterType::Zlib => "Zlib",
            FormatterType::Deflate => "Deflate",
            FormatterType::Brotli => "Brotli",
            FormatterType::Zstd => "Zstd",
            FormatterType::MsgPack => "MessagePack",
            FormatterType::Protobuf => "Protobuf",
            FormatterType::Yaml => "YAML",
            FormatterType::Toml => "TOML",
            FormatterType::Custom(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransformResult {
    Text(String),
    Binary(Vec<u8>),
    Error(String),
}

impl TransformResult {
    pub fn to_display_string(&self) -> String {
        match self {
            TransformResult::Text(s) => s.clone(),
            TransformResult::Binary(data) => {
                let hex: String = data.iter().map(|b| format!("{:02x}", b)).collect();
                format!(
                    "Binary({} bytes): {}",
                    data.len(),
                    hex.chars().take(100).collect::<String>()
                )
            }
            TransformResult::Error(e) => format!("Error: {}", e),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            TransformResult::Text(s) => s.as_bytes().to_vec(),
            TransformResult::Binary(data) => data.clone(),
            TransformResult::Error(e) => e.as_bytes().to_vec(),
        }
    }

    pub fn as_text(&self) -> String {
        match self {
            TransformResult::Text(s) => s.clone(),
            TransformResult::Binary(data) => String::from_utf8_lossy(data).to_string(),
            TransformResult::Error(e) => e.clone(),
        }
    }
}
