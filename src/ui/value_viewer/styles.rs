use crate::theme::{
    COLOR_BG_SECONDARY, COLOR_BORDER, COLOR_BUTTON_SECONDARY, COLOR_BUTTON_SECONDARY_BORDER,
    COLOR_ERROR, COLOR_ERROR_BG, COLOR_OVERLAY_BACKDROP, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_TEXT,
    COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
};

pub(super) fn secondary_action_button_style() -> String {
    format!(
        "height: 32px; padding: 0 10px; background: {}; color: {}; border: 1px solid {}; border-radius: 6px; cursor: pointer; display: flex; align-items: center; gap: 4px; font-size: 12px; font-weight: 500;",
        COLOR_BUTTON_SECONDARY, COLOR_TEXT, COLOR_BUTTON_SECONDARY_BORDER
    )
}

pub(super) fn primary_action_button_style(disabled: bool) -> String {
    let cursor = if disabled { "default" } else { "pointer" };
    let opacity = if disabled { "0.55" } else { "1" };

    format!(
        "height: 32px; padding: 0 12px; background: {}; color: {}; border: 1px solid {}; border-radius: 6px; cursor: {}; opacity: {}; display: flex; align-items: center; justify-content: center; gap: 4px; font-size: 12px; font-weight: 500;",
        COLOR_PRIMARY, COLOR_TEXT_CONTRAST, COLOR_PRIMARY, cursor, opacity
    )
}

pub(super) fn destructive_action_button_style(disabled: bool) -> String {
    let cursor = if disabled { "default" } else { "pointer" };
    let opacity = if disabled { "0.55" } else { "1" };

    format!(
        "height: 32px; padding: 0 12px; background: {}; color: {}; border: 1px solid {}; border-radius: 6px; cursor: {}; opacity: {}; display: flex; align-items: center; justify-content: center; gap: 4px; font-size: 12px; font-weight: 500;",
        COLOR_ERROR_BG, COLOR_ERROR, COLOR_BORDER, cursor, opacity
    )
}

pub(super) fn data_section_toolbar_style() -> &'static str {
    "display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; margin-bottom: 12px;"
}

pub(super) fn data_section_controls_style() -> &'static str {
    "display: flex; gap: 8px; align-items: center; flex-wrap: wrap;"
}

pub(super) fn data_section_count_style() -> String {
    format!(
        "color: {}; font-size: 12px; font-weight: 500;",
        COLOR_TEXT_SECONDARY
    )
}

pub(super) fn status_banner_style(is_error: bool) -> String {
    let background = if is_error {
        COLOR_ERROR_BG
    } else {
        COLOR_SUCCESS
    };
    let color = if is_error { COLOR_ERROR } else { COLOR_SUCCESS };

    format!(
        "margin-bottom: 12px; padding: 8px 12px; background: {}; border: 1px solid {}; border-radius: 8px; color: {}; font-size: 13px; line-height: 1.45;",
        background, COLOR_BORDER, color
    )
}

pub(super) fn data_table_header_row_style() -> String {
    format!(
        "background: {}; border-bottom: 1px solid {}; position: sticky; top: 0; z-index: 1;",
        COLOR_BG_SECONDARY, COLOR_BORDER
    )
}

pub(super) fn data_table_header_cell_style(width: Option<&str>, align: &str) -> String {
    let mut style = format!(
        "padding: 12px; color: {}; font-size: 12px; font-weight: 600; text-align: {};",
        COLOR_TEXT_SECONDARY, align
    );

    if let Some(width) = width {
        style.push_str(&format!(" width: {};", width));
    }

    style
}

pub(super) fn compact_icon_action_button_style(danger: bool, disabled: bool) -> String {
    let (background, color, border) = if danger {
        (COLOR_ERROR_BG, COLOR_ERROR, COLOR_BORDER)
    } else {
        (
            COLOR_BUTTON_SECONDARY,
            COLOR_TEXT_SECONDARY,
            COLOR_BUTTON_SECONDARY_BORDER,
        )
    };
    let cursor = if disabled { "default" } else { "pointer" };
    let opacity = if disabled { "0.55" } else { "1" };

    format!(
        "width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; background: {}; color: {}; border: 1px solid {}; border-radius: 6px; cursor: {}; opacity: {};",
        background, color, border, cursor, opacity
    )
}

pub(super) fn image_preview_button_style() -> String {
    format!(
        "height: 40px; padding: 0 14px; background: {}; color: {}; border: 1px solid {}; border-radius: 8px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 500;",
        COLOR_BUTTON_SECONDARY, COLOR_TEXT, COLOR_BUTTON_SECONDARY_BORDER
    )
}

pub(super) fn image_preview_info_chip_style() -> String {
    format!(
        "padding: 8px 14px; background: {}; color: {}; border: 1px solid {}; border-radius: 999px; font-size: 13px; font-weight: 500;",
        COLOR_BG_SECONDARY, COLOR_TEXT, COLOR_BORDER
    )
}

pub(super) fn overlay_modal_keyframes() -> &'static str {
    r#"
    @keyframes backdropFadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }
    @keyframes backdropFadeOut {
        from { opacity: 1; }
        to { opacity: 0; }
    }
    @keyframes modalFadeIn {
        from { opacity: 0; transform: scale(0.95); }
        to { opacity: 1; transform: scale(1); }
    }
    @keyframes modalFadeOut {
        from { opacity: 1; transform: scale(1); }
        to { opacity: 0; transform: scale(0.95); }
    }
    "#
}

pub(super) fn overlay_modal_backdrop_style(exiting: bool) -> String {
    let animation = if exiting {
        "backdropFadeOut 0.2s ease-out forwards"
    } else {
        "backdropFadeIn 0.2s ease-out"
    };

    format!(
        "position: fixed; inset: 0; background: {}; display: flex; align-items: center; justify-content: center; z-index: 1000; animation: {};",
        COLOR_OVERLAY_BACKDROP, animation
    )
}

pub(super) fn overlay_modal_surface_style(max_width: &str, exiting: bool) -> String {
    let animation = if exiting {
        "modalFadeOut 0.2s ease-out forwards"
    } else {
        "modalFadeIn 0.2s ease-out"
    };

    format!(
        "width: 90%; max-width: {}; padding: 24px; background: {}; border: 1px solid {}; border-radius: 12px; animation: {};",
        max_width, COLOR_BG_SECONDARY, COLOR_BORDER, animation
    )
}

pub(super) fn overlay_modal_title_style() -> &'static str {
    "margin: 0 0 16px 0; color: var(--theme-text); font-size: 16px; font-weight: 600;"
}

pub(super) fn overlay_modal_body_style() -> &'static str {
    "margin: 0 0 24px 0; color: var(--theme-text-secondary); font-size: 14px; line-height: 1.55; word-break: break-all;"
}

pub(super) fn overlay_modal_actions_style() -> &'static str {
    "display: flex; justify-content: flex-end; gap: 12px;"
}
