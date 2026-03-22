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
            background: "#f7f2f0",
            background_secondary: "#fffaf8",
            background_tertiary: "#f2e7e3",
            surface_lowest: "#ffffff",
            surface_low: "#fcf4f1",
            surface_high: "#f3e6e1",
            surface_highest: "#e8d8d2",
            border: "#dcc8c2",
            outline_variant: "#c7b0aa",
            text: "#241917",
            text_secondary: "#5f4a45",
            text_subtle: "#8c6f68",
            primary: "#b12c19",
            accent: "#007f8e",
            success: "#2d7a4b",
            warning: "#9b5c00",
            error: "#ba1a1a",
        }
    }
}
