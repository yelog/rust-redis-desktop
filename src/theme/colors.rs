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
            background: "#1e1e1e",
            background_secondary: "#252526",
            background_tertiary: "#2d2d2d",
            border: "#3c3c3c",
            text: "white",
            text_secondary: "#888888",
            primary: "#007acc",
            accent: "#4ec9b0",
            success: "#4ec9b0",
            warning: "#f59e0b",
            error: "#ef4444",
        }
    }

    pub fn light() -> Self {
        Self {
            background: "#ffffff",
            background_secondary: "#f3f3f3",
            background_tertiary: "#e8e8e8",
            border: "#d4d4d4",
            text: "#1e1e1e",
            text_secondary: "#666666",
            primary: "#007acc",
            accent: "#007acc",
            success: "#107c10",
            warning: "#ca5010",
            error: "#d13438",
        }
    }
}
