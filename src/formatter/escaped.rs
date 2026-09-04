#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscapeError {
    pub offset: usize,
    pub message: String,
}

impl std::fmt::Display for EscapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for EscapeError {}

pub fn escape_bytes(bytes: &[u8]) -> String {
    let mut output = String::new();
    for &byte in bytes {
        match byte {
            b'\n' => output.push_str("\\n"),
            b'\r' => output.push_str("\\r"),
            b'\t' => output.push_str("\\t"),
            0 => output.push_str("\\0"),
            b'\\' => output.push_str("\\\\"),
            b'"' => output.push_str("\\\""),
            0x20..=0x7e => output.push(byte as char),
            _ => output.push_str(&format!("\\x{byte:02x}")),
        }
    }
    output
}

pub fn unescape_text(text: &str) -> Result<Vec<u8>, EscapeError> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'\\' {
            let ch = text[index..].chars().next().ok_or_else(|| EscapeError {
                offset: index,
                message: "invalid UTF-8 boundary".to_string(),
            })?;
            let mut encoded = [0; 4];
            output.extend_from_slice(ch.encode_utf8(&mut encoded).as_bytes());
            index += ch.len_utf8();
            continue;
        }

        let offset = index;
        index += 1;
        let escaped = bytes.get(index).copied().ok_or_else(|| EscapeError {
            offset,
            message: "incomplete escape".to_string(),
        })?;
        match escaped {
            b'n' => output.push(b'\n'),
            b'r' => output.push(b'\r'),
            b't' => output.push(b'\t'),
            b'0' => output.push(0),
            b'\\' => output.push(b'\\'),
            b'"' => output.push(b'"'),
            b'x' => {
                let first = bytes.get(index + 1).copied();
                let second = bytes.get(index + 2).copied();
                let value = match (first.and_then(hex_value), second.and_then(hex_value)) {
                    (Some(first), Some(second)) => (first << 4) | second,
                    _ => {
                        return Err(EscapeError {
                            offset,
                            message: "\\x escape requires two hexadecimal digits".to_string(),
                        })
                    }
                };
                output.push(value);
                index += 2;
            }
            _ => {
                return Err(EscapeError {
                    offset,
                    message: "unknown escape".to_string(),
                })
            }
        }
        index += 1;
    }
    Ok(output)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_byte() {
        let input: Vec<u8> = (0..=u8::MAX).collect();
        assert_eq!(unescape_text(&escape_bytes(&input)).unwrap(), input);
    }

    #[test]
    fn rejects_invalid_escapes() {
        assert!(unescape_text("\\q").is_err());
        assert!(unescape_text("\\x0").is_err());
        assert!(unescape_text("\\").is_err());
    }
}
