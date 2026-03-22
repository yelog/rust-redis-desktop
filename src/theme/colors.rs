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
            background: "#131313",
            background_secondary: "#1c1b1b",
            background_tertiary: "#2a2a2a",
            border: "#353535",
            text: "#e5e2e1",
            text_secondary: "#e2bfb8",
            primary: "#ffb4a6",
            accent: "#00daf3",
            success: "#30d158",
            warning: "#ff9f0a",
            error: "#ffb4ab",
        }
    }

    pub fn light() -> Self {
        Self {
            background: "#f5f5f5",
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
