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
    TokyoNightLight,
    AtomOneLight,
    GitHubLight,
    OneDarkPro,
    Dracula,
}

impl ThemeId {
    pub const MANUAL_OPTIONS: [Self; 8] = [
        Self::ClassicDark,
        Self::ClassicLight,
        Self::TokyoNight,
        Self::TokyoNightLight,
        Self::AtomOneLight,
        Self::GitHubLight,
        Self::OneDarkPro,
        Self::Dracula,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClassicLight => "classic_light",
            Self::ClassicDark => "classic_dark",
            Self::TokyoNight => "tokyo_night",
            Self::TokyoNightLight => "tokyo_night_light",
            Self::AtomOneLight => "atom_one_light",
            Self::GitHubLight => "github_light",
            Self::OneDarkPro => "one_dark_pro",
            Self::Dracula => "dracula",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ClassicLight => "经典亮色",
            Self::ClassicDark => "经典暗色",
            Self::TokyoNight => "Tokyo Night",
            Self::TokyoNightLight => "Tokyo Night Light",
            Self::AtomOneLight => "Atom One Light",
            Self::GitHubLight => "GitHub Light",
            Self::OneDarkPro => "One Dark Pro",
            Self::Dracula => "Dracula",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "classic_light" | "classiclight" | "light" => Some(Self::ClassicLight),
            "classic_dark" | "classicdark" | "dark" => Some(Self::ClassicDark),
            "tokyo_night" | "tokyonight" | "tokyo-night" | "tokyo night" => Some(Self::TokyoNight),
            "tokyo_night_light" | "tokyonightlight" | "tokyo-night-light" => {
                Some(Self::TokyoNightLight)
            }
            "atom_one_light" | "atomonelight" | "atom-one-light" | "one light" => {
                Some(Self::AtomOneLight)
            }
            "github_light" | "githublight" | "github-light" => Some(Self::GitHubLight),
            "one_dark_pro" | "onedarkpro" | "one-dark-pro" | "one dark" => Some(Self::OneDarkPro),
            "dracula" => Some(Self::Dracula),
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
    pub state_connected: &'static str,
    pub state_connecting: &'static str,
    pub state_error: &'static str,
    pub state_disconnected: &'static str,
    pub error_bg: &'static str,
    pub success_bg: &'static str,
    pub overlay_backdrop: &'static str,
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
        state_connected: "#30d158",
        state_connecting: "#ff9f0a",
        state_error: "#ffb4ab",
        state_disconnected: "#a98a84",
        error_bg: "rgba(209, 52, 56, 0.12)",
        success_bg: "rgba(48, 209, 88, 0.08)",
        overlay_backdrop: "rgba(0, 0, 0, 0.7)",
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
        state_connected: "#2d7a4b",
        state_connecting: "#9b5c00",
        state_error: "#ba1a1a",
        state_disconnected: "#8c6f68",
        error_bg: "#fff1ef",
        success_bg: "#edf7f0",
        overlay_backdrop: "rgba(20, 17, 16, 0.54)",
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
        state_connected: "#9ece6a",
        state_connecting: "#e0af68",
        state_error: "#f7768e",
        state_disconnected: "#565f89",
        error_bg: "rgba(247, 118, 142, 0.16)",
        success_bg: "rgba(158, 206, 106, 0.14)",
        overlay_backdrop: "rgba(9, 11, 17, 0.78)",
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

const TOKYO_NIGHT_LIGHT: ThemeSpec = ThemeSpec {
    id: ThemeId::TokyoNightLight,
    label: "Tokyo Night Light",
    kind: ThemeKind::Light,
    colors: ThemeColors {
        background: "#e6e7ed",
        background_secondary: "#d5d6db",
        background_tertiary: "#cbccd2",
        surface_lowest: "#f3f4f8",
        surface_low: "#e6e7ed",
        surface_high: "#d5d6db",
        surface_highest: "#cbccd2",
        border: "#b4b5bd",
        outline_variant: "#9495a1",
        text: "#343b58",
        text_secondary: "#4f5779",
        text_subtle: "#6c6e75",
        primary_text: "#ffffff",
        primary: "#2959aa",
        accent: "#006c86",
        success: "#385f0d",
        warning: "#8f5e15",
        error: "#8c4351",
        state_connected: "#385f0d",
        state_connecting: "#8f5e15",
        state_error: "#8c4351",
        state_disconnected: "#6c6e75",
        error_bg: "rgba(140, 67, 81, 0.12)",
        success_bg: "rgba(56, 95, 13, 0.10)",
        overlay_backdrop: "rgba(52, 59, 88, 0.54)",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(52, 59, 88, 0.54)",
        control_bg: "#d5d6db",
        control_border: "#b4b5bd",
        button_secondary: "#cbccd2",
        button_secondary_border: "#b4b5bd",
        text_soft: "#4f5779",
        text_contrast: "#ffffff",
        info: "#006c86",
        outline: "#6c6e75",
        secondary_action: "#5a3e8e",
        info_bg: "rgba(0, 108, 134, 0.10)",
        info_bg_alt: "rgba(0, 108, 134, 0.18)",
        success_bg: "rgba(56, 95, 13, 0.10)",
        success_bg_alt: "rgba(56, 95, 13, 0.18)",
        warning_bg: "rgba(143, 94, 21, 0.12)",
        error_bg: "rgba(140, 67, 81, 0.12)",
        selection_bg: "rgba(41, 89, 170, 0.14)",
        selection_bg_alt: "rgba(0, 108, 134, 0.12)",
        row_create_bg: "rgba(0, 108, 134, 0.08)",
        row_edit_bg: "rgba(41, 89, 170, 0.12)",
        tone_string_bg: "rgba(41, 89, 170, 0.10)",
        tone_string_border: "rgba(41, 89, 170, 0.20)",
        tone_hash_bg: "rgba(0, 108, 134, 0.10)",
        tone_hash_border: "rgba(0, 108, 134, 0.20)",
        tone_list_bg: "rgba(79, 87, 121, 0.08)",
        tone_list_border: "rgba(79, 87, 121, 0.16)",
        tone_set_bg: "rgba(41, 89, 170, 0.10)",
        tone_set_border: "rgba(41, 89, 170, 0.20)",
        tone_zset_bg: "rgba(0, 108, 134, 0.10)",
        tone_zset_border: "rgba(0, 108, 134, 0.20)",
        tone_stream_bg: "rgba(56, 95, 13, 0.10)",
        tone_stream_border: "rgba(56, 95, 13, 0.20)",
    },
    syntax: ThemeSyntaxColors {
        key: "#2959aa",
        string: "#385f0d",
        number: "#965027",
        boolean: "#965027",
        null: "#6c6e75",
        bracket: "#343b58",
        keyword: "#5a3e8e",
        type_name: "#5a3e8e",
        function: "#2959aa",
        comment: "#6c6e75",
        operator: "#0f4b6e",
        constant: "#965027",
    },
};

const ATOM_ONE_LIGHT: ThemeSpec = ThemeSpec {
    id: ThemeId::AtomOneLight,
    label: "Atom One Light",
    kind: ThemeKind::Light,
    colors: ThemeColors {
        background: "#fafafa",
        background_secondary: "#f5f5f5",
        background_tertiary: "#e5e5e5",
        surface_lowest: "#ffffff",
        surface_low: "#fafafa",
        surface_high: "#f0f0f0",
        surface_highest: "#e5e5e5",
        border: "#e5e5e5",
        outline_variant: "#d4d4d4",
        text: "#383a42",
        text_secondary: "#5f6368",
        text_subtle: "#a0a1a7",
        primary_text: "#ffffff",
        primary: "#4078f2",
        accent: "#0184bc",
        success: "#50a14f",
        warning: "#c18401",
        error: "#e45649",
        state_connected: "#50a14f",
        state_connecting: "#c18401",
        state_error: "#e45649",
        state_disconnected: "#a0a1a7",
        error_bg: "rgba(228, 86, 73, 0.12)",
        success_bg: "rgba(80, 161, 79, 0.10)",
        overlay_backdrop: "rgba(56, 58, 66, 0.54)",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(56, 58, 66, 0.54)",
        control_bg: "#f0f0f0",
        control_border: "#d4d4d4",
        button_secondary: "#e5e5e5",
        button_secondary_border: "#d4d4d4",
        text_soft: "#5f6368",
        text_contrast: "#ffffff",
        info: "#0184bc",
        outline: "#a0a1a7",
        secondary_action: "#a626a4",
        info_bg: "rgba(1, 132, 188, 0.10)",
        info_bg_alt: "rgba(1, 132, 188, 0.18)",
        success_bg: "rgba(80, 161, 79, 0.10)",
        success_bg_alt: "rgba(80, 161, 79, 0.18)",
        warning_bg: "rgba(193, 132, 1, 0.12)",
        error_bg: "rgba(228, 86, 73, 0.12)",
        selection_bg: "rgba(64, 120, 242, 0.14)",
        selection_bg_alt: "rgba(1, 132, 188, 0.12)",
        row_create_bg: "rgba(1, 132, 188, 0.08)",
        row_edit_bg: "rgba(64, 120, 242, 0.12)",
        tone_string_bg: "rgba(64, 120, 242, 0.10)",
        tone_string_border: "rgba(64, 120, 242, 0.20)",
        tone_hash_bg: "rgba(1, 132, 188, 0.10)",
        tone_hash_border: "rgba(1, 132, 188, 0.20)",
        tone_list_bg: "rgba(95, 99, 104, 0.08)",
        tone_list_border: "rgba(95, 99, 104, 0.16)",
        tone_set_bg: "rgba(64, 120, 242, 0.10)",
        tone_set_border: "rgba(64, 120, 242, 0.20)",
        tone_zset_bg: "rgba(1, 132, 188, 0.10)",
        tone_zset_border: "rgba(1, 132, 188, 0.20)",
        tone_stream_bg: "rgba(80, 161, 79, 0.10)",
        tone_stream_border: "rgba(80, 161, 79, 0.20)",
    },
    syntax: ThemeSyntaxColors {
        key: "#e45649",
        string: "#50a14f",
        number: "#986801",
        boolean: "#0184bc",
        null: "#a0a1a7",
        bracket: "#383a42",
        keyword: "#a626a4",
        type_name: "#c18401",
        function: "#4078f2",
        comment: "#a0a1a7",
        operator: "#0184bc",
        constant: "#986801",
    },
};

const GITHUB_LIGHT: ThemeSpec = ThemeSpec {
    id: ThemeId::GitHubLight,
    label: "GitHub Light",
    kind: ThemeKind::Light,
    colors: ThemeColors {
        background: "#ffffff",
        background_secondary: "#f6f8fa",
        background_tertiary: "#eaeef2",
        surface_lowest: "#ffffff",
        surface_low: "#f6f8fa",
        surface_high: "#eaeef2",
        surface_highest: "#d8dee4",
        border: "#d0d7de",
        outline_variant: "#8c959f",
        text: "#24292f",
        text_secondary: "#57606a",
        text_subtle: "#656d76",
        primary_text: "#ffffff",
        primary: "#0969da",
        accent: "#0550ae",
        success: "#1a7f37",
        warning: "#9a6700",
        error: "#cf222e",
        state_connected: "#1a7f37",
        state_connecting: "#9a6700",
        state_error: "#cf222e",
        state_disconnected: "#656d76",
        error_bg: "rgba(207, 34, 46, 0.12)",
        success_bg: "rgba(26, 127, 55, 0.10)",
        overlay_backdrop: "rgba(36, 41, 47, 0.54)",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(36, 41, 47, 0.54)",
        control_bg: "#f6f8fa",
        control_border: "#d0d7de",
        button_secondary: "#eaeef2",
        button_secondary_border: "#d0d7de",
        text_soft: "#57606a",
        text_contrast: "#ffffff",
        info: "#0550ae",
        outline: "#656d76",
        secondary_action: "#8250df",
        info_bg: "rgba(5, 80, 174, 0.10)",
        info_bg_alt: "rgba(5, 80, 174, 0.18)",
        success_bg: "rgba(26, 127, 55, 0.10)",
        success_bg_alt: "rgba(26, 127, 55, 0.18)",
        warning_bg: "rgba(154, 103, 0, 0.12)",
        error_bg: "rgba(207, 34, 46, 0.12)",
        selection_bg: "rgba(9, 105, 218, 0.14)",
        selection_bg_alt: "rgba(5, 80, 174, 0.12)",
        row_create_bg: "rgba(5, 80, 174, 0.08)",
        row_edit_bg: "rgba(9, 105, 218, 0.12)",
        tone_string_bg: "rgba(9, 105, 218, 0.10)",
        tone_string_border: "rgba(9, 105, 218, 0.20)",
        tone_hash_bg: "rgba(5, 80, 174, 0.10)",
        tone_hash_border: "rgba(5, 80, 174, 0.20)",
        tone_list_bg: "rgba(87, 96, 106, 0.08)",
        tone_list_border: "rgba(87, 96, 106, 0.16)",
        tone_set_bg: "rgba(9, 105, 218, 0.10)",
        tone_set_border: "rgba(9, 105, 218, 0.20)",
        tone_zset_bg: "rgba(5, 80, 174, 0.10)",
        tone_zset_border: "rgba(5, 80, 174, 0.20)",
        tone_stream_bg: "rgba(26, 127, 55, 0.10)",
        tone_stream_border: "rgba(26, 127, 55, 0.20)",
    },
    syntax: ThemeSyntaxColors {
        key: "#cf222e",
        string: "#0a3069",
        number: "#0550ae",
        boolean: "#cf222e",
        null: "#656d76",
        bracket: "#24292f",
        keyword: "#cf222e",
        type_name: "#8250df",
        function: "#8250df",
        comment: "#656d76",
        operator: "#0550ae",
        constant: "#0550ae",
    },
};

const ONE_DARK_PRO: ThemeSpec = ThemeSpec {
    id: ThemeId::OneDarkPro,
    label: "One Dark Pro",
    kind: ThemeKind::Dark,
    colors: ThemeColors {
        background: "#282c34",
        background_secondary: "#21252b",
        background_tertiary: "#3e4451",
        surface_lowest: "#1e2127",
        surface_low: "#21252b",
        surface_high: "#3e4451",
        surface_highest: "#4b5263",
        border: "#3e4451",
        outline_variant: "#5c6370",
        text: "#abb2bf",
        text_secondary: "#9da5b4",
        text_subtle: "#5c6370",
        primary_text: "#282c34",
        primary: "#61afef",
        accent: "#56b6c2",
        success: "#98c379",
        warning: "#e5c07b",
        error: "#e06c75",
        state_connected: "#98c379",
        state_connecting: "#e5c07b",
        state_error: "#e06c75",
        state_disconnected: "#5c6370",
        error_bg: "rgba(224, 108, 117, 0.16)",
        success_bg: "rgba(152, 195, 121, 0.14)",
        overlay_backdrop: "rgba(33, 37, 43, 0.78)",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(33, 37, 43, 0.78)",
        control_bg: "#3e4451",
        control_border: "#4b5263",
        button_secondary: "#3e4451",
        button_secondary_border: "#4b5263",
        text_soft: "#7a8290",
        text_contrast: "#ffffff",
        info: "#56b6c2",
        outline: "#5c6370",
        secondary_action: "#c678dd",
        info_bg: "rgba(86, 182, 194, 0.14)",
        info_bg_alt: "rgba(97, 175, 239, 0.18)",
        success_bg: "rgba(152, 195, 121, 0.14)",
        success_bg_alt: "rgba(152, 195, 121, 0.22)",
        warning_bg: "rgba(229, 192, 123, 0.16)",
        error_bg: "rgba(224, 108, 117, 0.16)",
        selection_bg: "#3e4451",
        selection_bg_alt: "rgba(97, 175, 239, 0.18)",
        row_create_bg: "rgba(86, 182, 194, 0.10)",
        row_edit_bg: "rgba(97, 175, 239, 0.14)",
        tone_string_bg: "rgba(97, 175, 239, 0.14)",
        tone_string_border: "rgba(97, 175, 239, 0.28)",
        tone_hash_bg: "rgba(86, 182, 194, 0.14)",
        tone_hash_border: "rgba(86, 182, 194, 0.28)",
        tone_list_bg: "rgba(157, 165, 180, 0.10)",
        tone_list_border: "rgba(157, 165, 180, 0.20)",
        tone_set_bg: "rgba(97, 175, 239, 0.12)",
        tone_set_border: "rgba(97, 175, 239, 0.24)",
        tone_zset_bg: "rgba(86, 182, 194, 0.14)",
        tone_zset_border: "rgba(86, 182, 194, 0.28)",
        tone_stream_bg: "rgba(152, 195, 121, 0.14)",
        tone_stream_border: "rgba(152, 195, 121, 0.28)",
    },
    syntax: ThemeSyntaxColors {
        key: "#e06c75",
        string: "#98c379",
        number: "#d19a66",
        boolean: "#d19a66",
        null: "#5c6370",
        bracket: "#abb2bf",
        keyword: "#c678dd",
        type_name: "#e5c07b",
        function: "#61afef",
        comment: "#5c6370",
        operator: "#56b6c2",
        constant: "#d19a66",
    },
};

const DRACULA: ThemeSpec = ThemeSpec {
    id: ThemeId::Dracula,
    label: "Dracula",
    kind: ThemeKind::Dark,
    colors: ThemeColors {
        background: "#282a36",
        background_secondary: "#21222c",
        background_tertiary: "#44475a",
        surface_lowest: "#1d1e26",
        surface_low: "#21222c",
        surface_high: "#44475a",
        surface_highest: "#4d5067",
        border: "#44475a",
        outline_variant: "#6272a4",
        text: "#f8f8f2",
        text_secondary: "#f0f0ec",
        text_subtle: "#6272a4",
        primary_text: "#282a36",
        primary: "#bd93f9",
        accent: "#8be9fd",
        success: "#50fa7b",
        warning: "#ffb86c",
        error: "#ff5555",
        state_connected: "#50fa7b",
        state_connecting: "#ffb86c",
        state_error: "#ff5555",
        state_disconnected: "#6272a4",
        error_bg: "rgba(255, 85, 85, 0.16)",
        success_bg: "rgba(80, 250, 123, 0.14)",
        overlay_backdrop: "rgba(33, 34, 44, 0.78)",
    },
    derived: ThemeDerivedColors {
        overlay_backdrop: "rgba(33, 34, 44, 0.78)",
        control_bg: "#44475a",
        control_border: "#4d5067",
        button_secondary: "#44475a",
        button_secondary_border: "#4d5067",
        text_soft: "#b0b0a8",
        text_contrast: "#ffffff",
        info: "#8be9fd",
        outline: "#6272a4",
        secondary_action: "#ff79c6",
        info_bg: "rgba(139, 233, 253, 0.14)",
        info_bg_alt: "rgba(189, 147, 249, 0.18)",
        success_bg: "rgba(80, 250, 123, 0.14)",
        success_bg_alt: "rgba(80, 250, 123, 0.22)",
        warning_bg: "rgba(255, 184, 108, 0.16)",
        error_bg: "rgba(255, 85, 85, 0.16)",
        selection_bg: "#44475a",
        selection_bg_alt: "rgba(189, 147, 249, 0.18)",
        row_create_bg: "rgba(139, 233, 253, 0.10)",
        row_edit_bg: "rgba(189, 147, 249, 0.14)",
        tone_string_bg: "rgba(189, 147, 249, 0.14)",
        tone_string_border: "rgba(189, 147, 249, 0.28)",
        tone_hash_bg: "rgba(139, 233, 253, 0.14)",
        tone_hash_border: "rgba(139, 233, 253, 0.28)",
        tone_list_bg: "rgba(240, 240, 236, 0.10)",
        tone_list_border: "rgba(240, 240, 236, 0.20)",
        tone_set_bg: "rgba(189, 147, 249, 0.12)",
        tone_set_border: "rgba(189, 147, 249, 0.24)",
        tone_zset_bg: "rgba(139, 233, 253, 0.14)",
        tone_zset_border: "rgba(139, 233, 253, 0.28)",
        tone_stream_bg: "rgba(80, 250, 123, 0.14)",
        tone_stream_border: "rgba(80, 250, 123, 0.28)",
    },
    syntax: ThemeSyntaxColors {
        key: "#ff79c6",
        string: "#f1fa8c",
        number: "#bd93f9",
        boolean: "#ff79c6",
        null: "#6272a4",
        bracket: "#f8f8f2",
        keyword: "#ff79c6",
        type_name: "#8be9fd",
        function: "#50fa7b",
        comment: "#6272a4",
        operator: "#ff79c6",
        constant: "#bd93f9",
    },
};

pub fn theme_spec(id: ThemeId) -> ThemeSpec {
    match id {
        ThemeId::ClassicLight => CLASSIC_LIGHT,
        ThemeId::ClassicDark => CLASSIC_DARK,
        ThemeId::TokyoNight => TOKYO_NIGHT,
        ThemeId::TokyoNightLight => TOKYO_NIGHT_LIGHT,
        ThemeId::AtomOneLight => ATOM_ONE_LIGHT,
        ThemeId::GitHubLight => GITHUB_LIGHT,
        ThemeId::OneDarkPro => ONE_DARK_PRO,
        ThemeId::Dracula => DRACULA,
    }
}

pub fn resolve_theme(preference: ThemePreference, system_is_dark: bool) -> ThemeSpec {
    theme_spec(preference.resolved_theme_id(system_is_dark))
}
