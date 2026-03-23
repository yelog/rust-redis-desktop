use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    ClassicLight,
    #[default]
    ClassicDark,
    TokyoNight,
}

impl ThemeId {
    pub const MANUAL_OPTIONS: [Self; 3] = [Self::ClassicDark, Self::ClassicLight, Self::TokyoNight];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClassicLight => "classic_light",
            Self::ClassicDark => "classic_dark",
            Self::TokyoNight => "tokyo_night",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ClassicLight => "经典亮色",
            Self::ClassicDark => "经典暗色",
            Self::TokyoNight => "Tokyo Night",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "classic_light" | "classiclight" | "light" => Some(Self::ClassicLight),
            "classic_dark" | "classicdark" | "dark" => Some(Self::ClassicDark),
            "tokyo_night" | "tokyonight" | "tokyo-night" | "tokyo night" => Some(Self::TokyoNight),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    System,
    Manual(ThemeId),
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self::System
    }
}

impl ThemePreference {
    pub fn is_system(self) -> bool {
        matches!(self, Self::System)
    }

    pub fn manual_theme(self) -> Option<ThemeId> {
        match self {
            Self::System => None,
            Self::Manual(id) => Some(id),
        }
    }

    pub fn resolved_theme_id(self, system_is_dark: bool) -> ThemeId {
        match self {
            Self::System => {
                if system_is_dark {
                    ThemeId::ClassicDark
                } else {
                    ThemeId::ClassicLight
                }
            }
            Self::Manual(id) => id,
        }
    }

    fn storage_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Manual(id) => id.as_str(),
        }
    }

    fn from_storage_value(value: &str) -> Option<Self> {
        let normalized = value.trim();
        if normalized.eq_ignore_ascii_case("system") {
            return Some(Self::System);
        }

        ThemeId::from_str(normalized).map(Self::Manual)
    }
}

impl Serialize for ThemePreference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.storage_value())
    }
}

struct ThemePreferenceVisitor;

impl<'de> Visitor<'de> for ThemePreferenceVisitor {
    type Value = ThemePreference;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a theme preference string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ThemePreference::from_storage_value(value)
            .ok_or_else(|| E::custom(format!("unknown theme preference: {value}")))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

impl<'de> Deserialize<'de> for ThemePreference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ThemePreferenceVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub primary_text: &'static str,
    pub primary: &'static str,
    pub accent: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeDerivedColors {
    pub overlay_backdrop: &'static str,
    pub control_bg: &'static str,
    pub control_border: &'static str,
    pub button_secondary: &'static str,
    pub button_secondary_border: &'static str,
    pub text_soft: &'static str,
    pub text_contrast: &'static str,
    pub info: &'static str,
    pub outline: &'static str,
    pub secondary_action: &'static str,
    pub info_bg: &'static str,
    pub info_bg_alt: &'static str,
    pub success_bg: &'static str,
    pub success_bg_alt: &'static str,
    pub warning_bg: &'static str,
    pub error_bg: &'static str,
    pub selection_bg: &'static str,
    pub selection_bg_alt: &'static str,
    pub row_create_bg: &'static str,
    pub row_edit_bg: &'static str,
    pub tone_string_bg: &'static str,
    pub tone_string_border: &'static str,
    pub tone_hash_bg: &'static str,
    pub tone_hash_border: &'static str,
    pub tone_list_bg: &'static str,
    pub tone_list_border: &'static str,
    pub tone_set_bg: &'static str,
    pub tone_set_border: &'static str,
    pub tone_zset_bg: &'static str,
    pub tone_zset_border: &'static str,
    pub tone_stream_bg: &'static str,
    pub tone_stream_border: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeSyntaxColors {
    pub key: &'static str,
    pub string: &'static str,
    pub number: &'static str,
    pub boolean: &'static str,
    pub null: &'static str,
    pub bracket: &'static str,
    pub keyword: &'static str,
    pub type_name: &'static str,
    pub function: &'static str,
    pub comment: &'static str,
    pub operator: &'static str,
    pub constant: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeSpec {
    pub id: ThemeId,
    pub label: &'static str,
    pub kind: ThemeKind,
    pub colors: ThemeColors,
    pub derived: ThemeDerivedColors,
    pub syntax: ThemeSyntaxColors,
}

impl ThemeSpec {
    pub fn is_dark(self) -> bool {
        matches!(self.kind, ThemeKind::Dark)
    }
}

const CLASSIC_DARK: ThemeSpec = ThemeSpec {
    id: ThemeId::ClassicDark,
    label: "经典暗色",
    kind: ThemeKind::Dark,
    colors: ThemeColors {
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
        primary_text: "#3f0300",
        primary: "#ffb4a6",
        accent: "#00daf3",
        success: "#30d158",
        warning: "#ff9f0a",
        error: "#ffb4ab",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(0, 0, 0, 0.7)",
        control_bg: "#353535",
        control_border: "#5a413c",
        button_secondary: "#353535",
        button_secondary_border: "#5a413c",
        text_soft: "#e5e2e1",
        text_contrast: "#ffffff",
        info: "#00daf3",
        outline: "#a98a84",
        secondary_action: "#bb86fc",
        info_bg: "#1c1b1b",
        info_bg_alt: "#2a2a2a",
        success_bg: "rgba(48, 209, 88, 0.08)",
        success_bg_alt: "rgba(48, 209, 88, 0.16)",
        warning_bg: "rgba(255, 159, 10, 0.16)",
        error_bg: "rgba(209, 52, 56, 0.12)",
        selection_bg: "#2a2a2a",
        selection_bg_alt: "rgba(0, 218, 243, 0.10)",
        row_create_bg: "rgba(0, 218, 243, 0.08)",
        row_edit_bg: "rgba(0, 218, 243, 0.12)",
        tone_string_bg: "rgba(255, 180, 166, 0.12)",
        tone_string_border: "rgba(255, 180, 166, 0.24)",
        tone_hash_bg: "rgba(0, 218, 243, 0.10)",
        tone_hash_border: "rgba(0, 218, 243, 0.22)",
        tone_list_bg: "rgba(229, 226, 225, 0.08)",
        tone_list_border: "rgba(229, 226, 225, 0.18)",
        tone_set_bg: "rgba(255, 180, 166, 0.10)",
        tone_set_border: "rgba(255, 180, 166, 0.20)",
        tone_zset_bg: "rgba(0, 218, 243, 0.10)",
        tone_zset_border: "rgba(0, 218, 243, 0.22)",
        tone_stream_bg: "rgba(48, 209, 88, 0.10)",
        tone_stream_border: "rgba(48, 209, 88, 0.20)",
    },
    syntax: ThemeSyntaxColors {
        key: "#e2bfb8",
        string: "#00daf3",
        number: "#ffb4a6",
        boolean: "#a98a84",
        null: "#5a413c",
        bracket: "#e5e2e1",
        keyword: "#569cd6",
        type_name: "#dcdcaa",
        function: "#4ec9b0",
        comment: "#a98a84",
        operator: "#63b3ed",
        constant: "#ff9f0a",
    },
};

const CLASSIC_LIGHT: ThemeSpec = ThemeSpec {
    id: ThemeId::ClassicLight,
    label: "经典亮色",
    kind: ThemeKind::Light,
    colors: ThemeColors {
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
        primary_text: "#ffffff",
        primary: "#b12c19",
        accent: "#007f8e",
        success: "#2d7a4b",
        warning: "#9b5c00",
        error: "#ba1a1a",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(20, 17, 16, 0.54)",
        control_bg: "#fffaf8",
        control_border: "#c7b0aa",
        button_secondary: "#e8d8d2",
        button_secondary_border: "#c7b0aa",
        text_soft: "#6f5953",
        text_contrast: "#ffffff",
        info: "#007f8e",
        outline: "#8c6f68",
        secondary_action: "#7b4dd6",
        info_bg: "#e4f4f6",
        info_bg_alt: "#d8ecef",
        success_bg: "#edf7f0",
        success_bg_alt: "#e3f1e7",
        warning_bg: "#fff4e5",
        error_bg: "#fff1ef",
        selection_bg: "rgba(0, 127, 142, 0.12)",
        selection_bg_alt: "rgba(177, 44, 25, 0.08)",
        row_create_bg: "rgba(0, 127, 142, 0.08)",
        row_edit_bg: "rgba(0, 127, 142, 0.12)",
        tone_string_bg: "rgba(177, 44, 25, 0.10)",
        tone_string_border: "rgba(177, 44, 25, 0.18)",
        tone_hash_bg: "rgba(0, 127, 142, 0.10)",
        tone_hash_border: "rgba(0, 127, 142, 0.18)",
        tone_list_bg: "rgba(95, 74, 69, 0.08)",
        tone_list_border: "rgba(95, 74, 69, 0.16)",
        tone_set_bg: "rgba(177, 44, 25, 0.10)",
        tone_set_border: "rgba(177, 44, 25, 0.18)",
        tone_zset_bg: "rgba(0, 127, 142, 0.10)",
        tone_zset_border: "rgba(0, 127, 142, 0.18)",
        tone_stream_bg: "rgba(45, 122, 75, 0.10)",
        tone_stream_border: "rgba(45, 122, 75, 0.18)",
    },
    syntax: ThemeSyntaxColors {
        key: "#9f2d1f",
        string: "#006d79",
        number: "#b15d00",
        boolean: "#b42318",
        null: "#8c6f68",
        bracket: "#241917",
        keyword: "#005f73",
        type_name: "#7a5c1f",
        function: "#007f8e",
        comment: "#8c6f68",
        operator: "#0f6cbd",
        constant: "#9b5c00",
    },
};

const TOKYO_NIGHT: ThemeSpec = ThemeSpec {
    id: ThemeId::TokyoNight,
    label: "Tokyo Night",
    kind: ThemeKind::Dark,
    colors: ThemeColors {
        background: "#1a1b26",
        background_secondary: "#16161e",
        background_tertiary: "#24283b",
        surface_lowest: "#0c0e14",
        surface_low: "#16161e",
        surface_high: "#24283b",
        surface_highest: "#292e42",
        border: "#3b4261",
        outline_variant: "#545c7e",
        text: "#c0caf5",
        text_secondary: "#a9b1d6",
        text_subtle: "#565f89",
        primary_text: "#16161e",
        primary: "#7aa2f7",
        accent: "#7dcfff",
        success: "#9ece6a",
        warning: "#e0af68",
        error: "#f7768e",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(9, 11, 17, 0.78)",
        control_bg: "#24283b",
        control_border: "#3b4261",
        button_secondary: "#292e42",
        button_secondary_border: "#3b4261",
        text_soft: "#737aa2",
        text_contrast: "#ffffff",
        info: "#2ac3de",
        outline: "#565f89",
        secondary_action: "#bb9af7",
        info_bg: "rgba(42, 195, 222, 0.14)",
        info_bg_alt: "rgba(125, 207, 255, 0.18)",
        success_bg: "rgba(158, 206, 106, 0.14)",
        success_bg_alt: "rgba(115, 218, 202, 0.20)",
        warning_bg: "rgba(224, 175, 104, 0.16)",
        error_bg: "rgba(247, 118, 142, 0.16)",
        selection_bg: "#283457",
        selection_bg_alt: "rgba(122, 162, 247, 0.14)",
        row_create_bg: "rgba(42, 195, 222, 0.10)",
        row_edit_bg: "rgba(122, 162, 247, 0.14)",
        tone_string_bg: "rgba(122, 162, 247, 0.14)",
        tone_string_border: "rgba(122, 162, 247, 0.28)",
        tone_hash_bg: "rgba(125, 207, 255, 0.14)",
        tone_hash_border: "rgba(125, 207, 255, 0.28)",
        tone_list_bg: "rgba(169, 177, 214, 0.10)",
        tone_list_border: "rgba(169, 177, 214, 0.20)",
        tone_set_bg: "rgba(122, 162, 247, 0.12)",
        tone_set_border: "rgba(122, 162, 247, 0.24)",
        tone_zset_bg: "rgba(125, 207, 255, 0.14)",
        tone_zset_border: "rgba(125, 207, 255, 0.28)",
        tone_stream_bg: "rgba(158, 206, 106, 0.14)",
        tone_stream_border: "rgba(158, 206, 106, 0.28)",
    },
    syntax: ThemeSyntaxColors {
        key: "#7aa2f7",
        string: "#9ece6a",
        number: "#ff9e64",
        boolean: "#ff9e64",
        null: "#565f89",
        bracket: "#c0caf5",
        keyword: "#9d7cd8",
        type_name: "#bb9af7",
        function: "#7aa2f7",
        comment: "#565f89",
        operator: "#89ddff",
        constant: "#ff9e64",
    },
};

pub fn theme_spec(id: ThemeId) -> ThemeSpec {
    match id {
        ThemeId::ClassicLight => CLASSIC_LIGHT,
        ThemeId::ClassicDark => CLASSIC_DARK,
        ThemeId::TokyoNight => TOKYO_NIGHT,
    }
}

pub fn resolve_theme(preference: ThemePreference, system_is_dark: bool) -> ThemeSpec {
    theme_spec(preference.resolved_theme_id(system_is_dark))
}
