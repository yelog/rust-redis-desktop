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
    pub surface_lowest: &'static str,
    pub surface_low: &'static str,
    pub surface_high: &'static str,
    pub surface_highest: &'static str,
    pub border: &'static str,
    pub outline_variant: &'static str,
    pub text: &'static str,
    pub text_secondary: &'static str,
    pub text_subtle: &'static str,
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
            surface_lowest: "#0e0e0e",
            surface_low: "#1c1b1b",
            surface_high: "#2a2a2a",
            surface_highest: "#353535",
            border: "#353535",
            outline_variant: "#5a413c",
            text: "#e5e2e1",
            text_secondary: "#e2bfb8",
            text_subtle: "#a98a84",
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
            surface_lowest: "#ffffff",
            surface_low: "#f3f3f3",
            surface_high: "#e8e8e8",
            surface_highest: "#d9d9d9",
            border: "#d0d0d0",
            outline_variant: "#c7c7c7",
            text: "#1e1e1e",
            text_secondary: "#666666",
            text_subtle: "#808080",
            primary: "#007aff",
            accent: "#007aff",
            success: "#28a745",
            warning: "#f59e0b",
            error: "#dc3545",
        }
    }
}
