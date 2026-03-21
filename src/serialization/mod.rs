pub mod java_converters;

pub use jaded::{Content, Parser, PrimitiveType, Value};

/// Check if data is Java serialization format
pub fn is_java_serialization(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0xAC && data[1] == 0xED
}

/// Simplify class name for display
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

/// Format class name with package info
pub fn format_class_name_with_package(full_name: &str) -> String {
    let simplified = simplify_class_name(full_name);
    if simplified == full_name {
        simplified
    } else {
        format!("{} ({})", simplified, full_name)
    }
}
