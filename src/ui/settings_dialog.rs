use crate::config::AppSettings;
use crate::theme::{ThemeColors, ThemeId, ThemePreference};
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::icons::IconX;
use dioxus::prelude::*;

#[component]
pub fn SettingsDialog(
    settings: AppSettings,
    colors: ThemeColors,
    resolved_theme_id: ThemeId,
    on_change: EventHandler<AppSettings>,
    on_close: EventHandler<()>,
) -> Element {
    let mut auto_refresh_interval = use_signal(|| settings.auto_refresh_interval);
    let mut use_system_theme = use_signal(|| settings.theme_preference.is_system());
    let mut manual_theme = use_signal(|| {
        settings
            .theme_preference
            .manual_theme()
            .unwrap_or(resolved_theme_id)
    });

    let apply_settings = {
        let on_change = on_change.clone();
        move || {
            on_change.call(AppSettings {
                auto_refresh_interval: auto_refresh_interval(),
                theme_preference: if use_system_theme() {
                    ThemePreference::System
                } else {
                    ThemePreference::Manual(manual_theme())
                },
            });
        }
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "420px".to_string(),

            div {
                position: "relative",

                button {
                    position: "absolute",
                    top: "-8px",
                    right: "-8px",
                    z_index: "10",
                    padding: "4px",
                    background: "transparent",
                    border: "none",
                    cursor: "pointer",
                    color: "{colors.text_secondary}",
                    onclick: move |_| on_close.call(()),

                    IconX { size: Some(18) }
                }

                h2 {
                    color: "{colors.text}",
                    margin_bottom: "24px",
                    font_size: "20px",

                    "设置"
                }

                div {
                    margin_bottom: "20px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "8px",

                        "主题模式"
                    }

                    div {
                        display: "flex",
                        flex_wrap: "wrap",
                        gap: "8px",

                        for (value, label) in [(true, "跟随系统"), (false, "手动选择")] {
                            {
                                let is_selected = use_system_theme() == value;
                                let bg = if is_selected { colors.primary.clone() } else { colors.background_tertiary.clone() };
                                let border_color = if is_selected { colors.primary.clone() } else { colors.border.clone() };
                                let text_color = if is_selected { colors.primary_text.clone() } else { colors.text.clone() };
                                rsx! {
                                    div {
                                        key: "{label}",
                                        padding: "6px 14px",
                                        background: "{bg}",
                                        border: "1px solid {border_color}",
                                        border_radius: "16px",
                                        color: "{text_color}",
                                        font_size: "13px",
                                        cursor: "pointer",
                                        user_select: "none",
                                        onclick: {
                                            let apply = apply_settings.clone();
                                            move |_| {
                                                use_system_theme.set(value);
                                                apply();
                                            }
                                        },

                                        "{label}"
                                    }
                                }
                            }
                        }
                    }

                    if !use_system_theme() {
                        div {
                            margin_top: "16px",

                            label {
                                display: "block",
                                color: "{colors.text_secondary}",
                                font_size: "12px",
                                margin_bottom: "8px",

                                "手动主题"
                            }

                            div {
                                display: "flex",
                                flex_wrap: "wrap",
                                gap: "8px",

                                for theme_id in ThemeId::MANUAL_OPTIONS {
                                    {
                                        let is_selected = manual_theme() == theme_id;
                                        let bg = if is_selected { colors.primary.clone() } else { colors.background_tertiary.clone() };
                                        let border_color = if is_selected { colors.primary.clone() } else { colors.border.clone() };
                                        let text_color = if is_selected { colors.primary_text.clone() } else { colors.text.clone() };
                                        rsx! {
                                            div {
                                                key: "{theme_id.as_str()}",
                                                padding: "6px 14px",
                                                background: "{bg}",
                                                border: "1px solid {border_color}",
                                                border_radius: "16px",
                                                color: "{text_color}",
                                                font_size: "13px",
                                                cursor: "pointer",
                                                user_select: "none",
                                                onclick: {
                                                    let apply = apply_settings.clone();
                                                    move |_| {
                                                        manual_theme.set(theme_id);
                                                        apply();
                                                    }
                                                },

                                                "{theme_id.label()}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    margin_bottom: "20px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "8px",

                        "服务器信息自动刷新"
                    }

                    div {
                        display: "flex",
                        flex_wrap: "wrap",
                        gap: "8px",

                        for (value, label) in [(0, "关闭"), (5, "5秒"), (10, "10秒"), (30, "30秒"), (60, "60秒")] {
                            {
                                let is_selected = auto_refresh_interval() == value;
                                let bg = if is_selected { colors.primary.clone() } else { colors.background_tertiary.clone() };
                                let border_color = if is_selected { colors.primary.clone() } else { colors.border.clone() };
                                let text_color = if is_selected { colors.primary_text.clone() } else { colors.text.clone() };
                                rsx! {
                                    div {
                                        key: "{value}",
                                        padding: "6px 14px",
                                        background: "{bg}",
                                        border: "1px solid {border_color}",
                                        border_radius: "16px",
                                        color: "{text_color}",
                                        font_size: "13px",
                                        cursor: "pointer",
                                        user_select: "none",
                                        onclick: {
                                            let apply = apply_settings.clone();
                                            move |_| {
                                                auto_refresh_interval.set(value);
                                                apply();
                                            }
                                        },

                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
