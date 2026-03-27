use crate::protobuf_schema::types::*;
use std::path::Path;

pub fn parse_proto_file(path: &Path) -> Result<ProtoFile, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;

    parse_proto_content(&content)
}

pub fn parse_proto_content(content: &str) -> Result<ProtoFile, String> {
    let mut proto_file = ProtoFile {
        package: String::new(),
        messages: Vec::new(),
        enums: Vec::new(),
        imports: Vec::new(),
    };

    let content = remove_comments(content);

    if let Some(pkg) = extract_package(&content) {
        proto_file.package = pkg;
    }

    proto_file.imports = extract_imports(&content);

    let (messages, enums) = extract_messages_and_enums(&content, &proto_file.package);
    proto_file.messages = messages;
    proto_file.enums = enums;

    Ok(proto_file)
}

fn remove_comments(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if c == '"' && !in_string {
            in_string = true;
            result.push(c);
            continue;
        }
        if c == '"' && in_string {
            in_string = false;
            result.push(c);
            continue;
        }
        if in_string {
            result.push(c);
            continue;
        }

        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if next == '*' {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}

fn extract_package(content: &str) -> Option<String> {
    let package_pattern = regex::Regex::new(r"package\s+([\w.]+)\s*;").ok()?;
    let cap = package_pattern.captures(content)?;
    Some(cap[1].to_string())
}

fn extract_imports(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let import_pattern =
        regex::Regex::new(r#"import\s+(?:public\s+)?["']([^"']+)["']\s*;"#).unwrap();

    for cap in import_pattern.captures_iter(content) {
        imports.push(cap[1].to_string());
    }

    imports
}

fn extract_messages_and_enums(content: &str, package: &str) -> (Vec<MessageDef>, Vec<EnumDef>) {
    let mut messages = Vec::new();
    let mut enums = Vec::new();

    extract_messages_recursive(content, package, &mut messages, &mut enums);

    (messages, enums)
}

fn extract_messages_recursive(
    content: &str,
    prefix: &str,
    messages: &mut Vec<MessageDef>,
    enums: &mut Vec<EnumDef>,
) {
    let mut pos = 0;
    let content_bytes = content.as_bytes();

    while pos < content.len() {
        let remaining = &content[pos..];

        if let Some(msg_start) = find_keyword(remaining, "message") {
            let msg_content = extract_block(&remaining[msg_start..], '{', '}');
            if let Some(block) = msg_content {
                let header_end = remaining[msg_start..].find('{').unwrap();
                let header = &remaining[msg_start..msg_start + header_end].trim();

                if let Some(name) = extract_identifier_after_keyword(header, "message") {
                    let full_name = if prefix.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}.{}", prefix, name)
                    };

                    let mut msg = MessageDef {
                        name: name.to_string(),
                        full_name: full_name.clone(),
                        fields: Vec::new(),
                        nested_messages: Vec::new(),
                    };

                    let inner_content = &block[1..block.len() - 1];
                    extract_fields(inner_content, &mut msg.fields);

                    let mut nested_messages = Vec::new();
                    let mut nested_enums = Vec::new();
                    extract_messages_recursive(
                        inner_content,
                        &full_name,
                        &mut nested_messages,
                        &mut nested_enums,
                    );
                    messages.extend(nested_messages.clone());
                    msg.nested_messages = nested_messages;
                    enums.extend(nested_enums);

                    messages.push(msg);
                }

                pos += msg_start + block.len();
                continue;
            }
        }

        if let Some(enum_start) = find_keyword(remaining, "enum") {
            let enum_content = extract_block(&remaining[enum_start..], '{', '}');
            if let Some(block) = enum_content {
                let header_end = remaining[enum_start..].find('{').unwrap();
                let header = &remaining[enum_start..enum_start + header_end].trim();

                if let Some(name) = extract_identifier_after_keyword(header, "enum") {
                    let full_name = if prefix.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}.{}", prefix, name)
                    };

                    let inner_content = &block[1..block.len() - 1];
                    let values = extract_enum_values(inner_content);

                    enums.push(EnumDef {
                        name: full_name,
                        values,
                    });
                }

                pos += enum_start + block.len();
                continue;
            }
        }

        pos += 1;
    }
}

fn find_keyword(content: &str, keyword: &str) -> Option<usize> {
    let pattern = format!(r"\b{}\s+", keyword);
    let re = regex::Regex::new(&pattern).ok()?;
    let m = re.find(content)?;
    Some(m.start())
}

fn extract_identifier_after_keyword(text: &str, keyword: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(r"\b{}\s+(\w+)", keyword)).ok()?;
    let cap = re.captures(text)?;
    Some(cap[1].to_string())
}

fn extract_block(content: &str, open: char, close: char) -> Option<String> {
    let start = content.find(open)?;
    let mut depth = 1;
    let mut end = start + 1;

    for (i, c) in content[start + 1..].char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                end = start + 1 + i + 1;
                break;
            }
        }
    }

    if depth == 0 {
        Some(content[start..end].to_string())
    } else {
        None
    }
}

fn extract_fields(content: &str, fields: &mut Vec<FieldDef>) {
    let field_pattern =
        regex::Regex::new(r"(optional|required|repeated)?\s*(\w+)\s+(\w+)\s*=\s*(\d+)\s*;")
            .unwrap();

    for cap in field_pattern.captures_iter(content) {
        let label = match cap.get(1).map(|m| m.as_str()) {
            Some("required") => FieldLabel::Required,
            Some("repeated") => FieldLabel::Repeated,
            _ => FieldLabel::Optional,
        };

        let field_type = FieldType::from_str(&cap[2]);
        let name = cap[3].to_string();
        let number: u32 = cap[4].parse().unwrap_or(0);

        fields.push(FieldDef {
            name,
            number,
            field_type,
            label,
        });
    }
}

fn extract_enum_values(content: &str) -> Vec<(String, i32)> {
    let mut values = Vec::new();
    let pattern = regex::Regex::new(r"(\w+)\s*=\s*(-?\d+)\s*;").unwrap();

    for cap in pattern.captures_iter(content) {
        let name = cap[1].to_string();
        let value: i32 = cap[2].parse().unwrap_or(0);
        values.push((name, value));
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_message() {
        let content = r#"
            syntax = "proto3";
            package test;
            
            message Person {
                string name = 1;
                int32 age = 2;
                repeated string emails = 3;
            }
        "#;

        let result = parse_proto_content(content).unwrap();
        assert_eq!(result.package, "test");
        assert_eq!(result.messages.len(), 1);

        let msg = &result.messages[0];
        assert_eq!(msg.name, "Person");
        assert_eq!(msg.fields.len(), 3);
        assert_eq!(msg.fields[0].name, "name");
        assert_eq!(msg.fields[0].number, 1);
    }

    #[test]
    fn test_parse_nested_message() {
        let content = r#"
            package outer;
            
            message Outer {
                message Inner {
                    int32 value = 1;
                }
                Inner inner = 1;
            }
        "#;

        let result = parse_proto_content(content).unwrap();
        assert_eq!(result.messages.len(), 2);
    }

    #[test]
    fn test_parse_enum() {
        let content = r#"
            package test;
            
            enum Status {
                UNKNOWN = 0;
                ACTIVE = 1;
                INACTIVE = 2;
            }
        "#;

        let result = parse_proto_content(content).unwrap();
        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.enums[0].values.len(), 3);
    }
}
