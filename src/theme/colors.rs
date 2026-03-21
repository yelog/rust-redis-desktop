use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
    pub background: &'static str,
    pub background_secondary: &'static str,
    pub background_tertiary: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub text_secondary: &'static str,
    pub primary: &'static str,
    pub accent: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: "#1a1a1a",
            background_secondary: "#2a2a2a",
            background_tertiary: "#363636",
            border: "#404040",
            text: "white",
            text_secondary: "#888888",
            primary: "#0a84ff",
            accent: "#0a84ff",
            success: "#30d158",
            warning: "#ff9f0a",
            error: "#ff453a",
        }
    }

    pub fn light() -> Self {
        Self {
            background: "#f0f0f0",
            background_secondary: "#ffffff",
            background_tertiary: "#e8e8e8",
            border: "#d0d0d0",
            text: "#1e1e1e",
            text_secondary: "#666666",
            primary: "#007aff",
            accent: "#007aff",
            success: "#28a745",
            warning: "#f59e0b",
            error: "#dc3545",
        }
    }
}
