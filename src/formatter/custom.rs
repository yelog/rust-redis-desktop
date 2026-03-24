use crate::formatter::{FormatterType, TransformResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormatterConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub formatter_type: String,
    pub enabled: bool,
}

impl FormatterConfig {
    pub fn new(name: String, formatter_type: FormatterType) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            name,
            description: None,
            formatter_type: formatter_type.as_str().to_string(),
            enabled: true,
        }
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomFormatter {
    pub configs: Vec<FormatterConfig>,
}

impl CustomFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_config(&mut self, config: FormatterConfig) {
        if !self.configs.iter().any(|c| c.id == config.id) {
            self.configs.push(config);
        }
    }

    pub fn remove_config(&mut self, id: &str) {
        self.configs.retain(|c| c.id != id);
    }

    pub fn update_config(&mut self, config: FormatterConfig) {
        if let Some(existing) = self.configs.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
        }
    }

    pub fn get_config(&self, id: &str) -> Option<&FormatterConfig> {
        self.configs.iter().find(|c| c.id == id)
    }

    pub fn enabled_configs(&self) -> Vec<&FormatterConfig> {
        self.configs.iter().filter(|c| c.enabled).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatterRegistry {
    pub custom: CustomFormatter,
    pub recent_used: Vec<String>,
}

impl FormatterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_formatter(&self, name: &str) -> Option<FormatterType> {
        FormatterType::from_str(name)
    }

    pub fn record_usage(&mut self, formatter_name: &str) {
        self.recent_used.retain(|n| n != formatter_name);
        self.recent_used.insert(0, formatter_name.to_string());
        if self.recent_used.len() > 10 {
            self.recent_used.truncate(10);
        }
    }

    pub fn get_recent_used(&self) -> &[String] {
        &self.recent_used
    }
}

pub fn get_builtin_formatters() -> Vec<FormatterConfig> {
    vec![
        FormatterConfig::new("JSON 格式化".to_string(), FormatterType::Json)
            .with_description("解析并格式化 JSON 数据".to_string()),
        FormatterConfig::new("Hex 编码".to_string(), FormatterType::Hex)
            .with_description("将数据转换为十六进制".to_string()),
        FormatterConfig::new("Base64 解码".to_string(), FormatterType::Base64)
            .with_description("解码 Base64 编码数据".to_string()),
        FormatterConfig::new("Base64 URL 解码".to_string(), FormatterType::Base64Url)
            .with_description("解码 Base64 URL 安全编码".to_string()),
        FormatterConfig::new("URL 解码".to_string(), FormatterType::UrlEncode)
            .with_description("解码 URL 编码数据".to_string()),
        FormatterConfig::new("Gzip 解压".to_string(), FormatterType::Gzip)
            .with_description("解压 Gzip 压缩数据".to_string()),
        FormatterConfig::new("Zlib 解压".to_string(), FormatterType::Zlib)
            .with_description("解压 Zlib 压缩数据".to_string()),
        FormatterConfig::new("Deflate 解压".to_string(), FormatterType::Deflate)
            .with_description("解压 Deflate 压缩数据".to_string()),
        FormatterConfig::new("Brotli 解压".to_string(), FormatterType::Brotli)
            .with_description("解压 Brotli 压缩数据".to_string()),
        FormatterConfig::new("MsgPack 解码".to_string(), FormatterType::MsgPack)
            .with_description("解码 MessagePack 格式".to_string()),
        FormatterConfig::new("Protobuf 解析".to_string(), FormatterType::Protobuf)
            .with_description("解析 Protobuf 原始数据".to_string()),
    ]
}

pub fn apply_formatter_chain(input: &[u8], formatters: &[FormatterType]) -> TransformResult {
    let mut current_data = input.to_vec();
    let mut is_binary = false;

    for formatter in formatters {
        let result = crate::formatter::apply_preset_formatter(
            formatter,
            if is_binary {
                &current_data
            } else {
                if let Ok(s) = std::str::from_utf8(&current_data) {
                    s.as_bytes()
                } else {
                    &current_data
                }
            },
        );

        match result {
            TransformResult::Text(t) => {
                current_data = t.into_bytes();
                is_binary = false;
            }
            TransformResult::Binary(b) => {
                current_data = b;
                is_binary = true;
            }
            TransformResult::Error(e) => {
                return TransformResult::Error(format!(
                    "Formatter {} failed: {}",
                    formatter.display_name(),
                    e
                ));
            }
        }
    }

    if is_binary {
        if let Ok(s) = String::from_utf8(current_data) {
            TransformResult::Text(s)
        } else {
            TransformResult::Binary(current_data)
        }
    } else {
        TransformResult::Text(String::from_utf8_lossy(&current_data).to_string())
    }
}
