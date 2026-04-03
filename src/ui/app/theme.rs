use crate::config::{AppSettings, ConfigStorage};
use crate::theme::ThemeSpec;
use serde_json::{Map, Value};

pub(super) fn system_theme_is_dark() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains("Dark"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub(super) fn load_initial_settings() -> AppSettings {
    ConfigStorage::new()
        .ok()
        .and_then(|storage| storage.load_settings().ok())
        .unwrap_or_default()
}

pub(super) fn build_theme_palette(theme: ThemeSpec) -> Value {
    let colors = theme.colors;
    let derived = theme.derived;
    let syntax = theme.syntax;
    let mut palette = Map::new();

    fn insert_str(palette: &mut Map<String, Value>, key: &str, value: &str) {
        palette.insert(key.to_string(), Value::String(value.to_string()));
    }

    insert_str(&mut palette, "id", theme.id.as_str());
    insert_str(&mut palette, "label", theme.label);
    palette.insert("isDark".to_string(), Value::Bool(theme.is_dark()));
    insert_str(&mut palette, "surfaceBase", colors.background);
    insert_str(
        &mut palette,
        "surfaceSecondary",
        colors.background_secondary,
    );
    insert_str(&mut palette, "surfaceTertiary", colors.background_tertiary);
    insert_str(&mut palette, "surfaceLowest", colors.surface_lowest);
    insert_str(&mut palette, "surfaceLow", colors.surface_low);
    insert_str(&mut palette, "surfaceHigh", colors.surface_high);
    insert_str(&mut palette, "surfaceHighest", colors.surface_highest);
    insert_str(&mut palette, "border", colors.border);
    insert_str(&mut palette, "outlineVariant", colors.outline_variant);
    insert_str(&mut palette, "overlayBackdrop", derived.overlay_backdrop);
    insert_str(&mut palette, "controlBg", derived.control_bg);
    insert_str(&mut palette, "controlBorder", derived.control_border);
    insert_str(&mut palette, "buttonSecondary", derived.button_secondary);
    insert_str(
        &mut palette,
        "buttonSecondaryBorder",
        derived.button_secondary_border,
    );
    insert_str(&mut palette, "textPrimary", colors.text);
    insert_str(&mut palette, "textSecondary", colors.text_secondary);
    insert_str(&mut palette, "textSubtle", colors.text_subtle);
    insert_str(&mut palette, "textSoft", derived.text_soft);
    insert_str(&mut palette, "textContrast", derived.text_contrast);
    insert_str(&mut palette, "primary", colors.primary);
    insert_str(&mut palette, "accent", colors.accent);
    insert_str(&mut palette, "success", colors.success);
    insert_str(&mut palette, "warning", colors.warning);
    insert_str(&mut palette, "error", colors.error);
    insert_str(&mut palette, "info", derived.info);
    insert_str(&mut palette, "outline", derived.outline);
    insert_str(&mut palette, "secondaryAction", derived.secondary_action);
    insert_str(&mut palette, "infoBg", derived.info_bg);
    insert_str(&mut palette, "infoBgAlt", derived.info_bg_alt);
    insert_str(&mut palette, "successBg", derived.success_bg);
    insert_str(&mut palette, "successBgAlt", derived.success_bg_alt);
    insert_str(&mut palette, "warningBg", derived.warning_bg);
    insert_str(&mut palette, "errorBg", derived.error_bg);
    insert_str(&mut palette, "selectionBg", derived.selection_bg);
    insert_str(&mut palette, "selectionBgAlt", derived.selection_bg_alt);
    insert_str(&mut palette, "rowCreateBg", derived.row_create_bg);
    insert_str(&mut palette, "rowEditBg", derived.row_edit_bg);
    insert_str(&mut palette, "toneStringBg", derived.tone_string_bg);
    insert_str(&mut palette, "toneStringBorder", derived.tone_string_border);
    insert_str(&mut palette, "toneHashBg", derived.tone_hash_bg);
    insert_str(&mut palette, "toneHashBorder", derived.tone_hash_border);
    insert_str(&mut palette, "toneListBg", derived.tone_list_bg);
    insert_str(&mut palette, "toneListBorder", derived.tone_list_border);
    insert_str(&mut palette, "toneSetBg", derived.tone_set_bg);
    insert_str(&mut palette, "toneSetBorder", derived.tone_set_border);
    insert_str(&mut palette, "toneZsetBg", derived.tone_zset_bg);
    insert_str(&mut palette, "toneZsetBorder", derived.tone_zset_border);
    insert_str(&mut palette, "toneStreamBg", derived.tone_stream_bg);
    insert_str(&mut palette, "toneStreamBorder", derived.tone_stream_border);
    insert_str(&mut palette, "syntaxKey", syntax.key);
    insert_str(&mut palette, "syntaxString", syntax.string);
    insert_str(&mut palette, "syntaxNumber", syntax.number);
    insert_str(&mut palette, "syntaxBoolean", syntax.boolean);
    insert_str(&mut palette, "syntaxNull", syntax.null);
    insert_str(&mut palette, "syntaxBracket", syntax.bracket);
    insert_str(&mut palette, "syntaxKeyword", syntax.keyword);
    insert_str(&mut palette, "syntaxType", syntax.type_name);
    insert_str(&mut palette, "syntaxFunction", syntax.function);
    insert_str(&mut palette, "syntaxComment", syntax.comment);
    insert_str(&mut palette, "syntaxOperator", syntax.operator);
    insert_str(&mut palette, "syntaxConstant", syntax.constant);

    Value::Object(palette)
}
