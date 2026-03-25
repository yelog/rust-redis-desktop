use crate::config::AppSettings;
use crate::theme::{ThemeColors, ThemeId, ThemeMode, ThemePreference};
use crate::ui::animated_dialog::AnimatedDialog;
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
    let mut theme_mode = use_signal(|| settings.theme_preference.mode());
    let mut light_theme = use_signal(|| settings.theme_preference.light_theme());
    let mut dark_theme = use_signal(|| settings.theme_preference.dark_theme());

    let apply_settings = {
        let on_change = on_change.clone();
        move || {
            let preference = match theme_mode() {
                ThemeMode::System => ThemePreference::System {
                    light: light_theme(),
                    dark: dark_theme(),
                },
                ThemeMode::Dark => ThemePreference::Dark(dark_theme()),
                ThemeMode::Light => ThemePreference::Light(light_theme()),
            };
            on_change.call(AppSettings {
                auto_refresh_interval: auto_refresh_interval(),
                theme_preference: preference,
            });
        }
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "520px".to_string(),
            title: "设置".to_string(),

            div {
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

                        for (mode, label) in [
                            (ThemeMode::System, "跟随系统"),
                            (ThemeMode::Dark, "暗色"),
                            (ThemeMode::Light, "亮色"),
                        ] {
                            {
                                let is_selected = theme_mode() == mode;
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
                                                theme_mode.set(mode);
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

                {
                    let mode = theme_mode();
                    let show_light = matches!(mode, ThemeMode::System | ThemeMode::Light);
                    let show_dark = matches!(mode, ThemeMode::System | ThemeMode::Dark);

                    rsx! {
                        if show_light {
                            ThemeSelector {
                                label: "亮色主题",
                                options: ThemeId::LIGHT_OPTIONS,
                                selected: light_theme(),
                                colors,
                                on_select: {
                                    let apply = apply_settings.clone();
                                    move |id: ThemeId| {
                                        light_theme.set(id);
                                        apply();
                                    }
                                },
                            }
                        }

                        if show_dark {
                            ThemeSelector {
                                label: "暗色主题",
                                options: ThemeId::DARK_OPTIONS,
                                selected: dark_theme(),
                                colors,
                                on_select: {
                                    let apply = apply_settings.clone();
                                    move |id: ThemeId| {
                                        dark_theme.set(id);
                                        apply();
                                    }
                                },
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

#[component]
fn ThemeSelector(
    label: &'static str,
    options: [ThemeId; 4],
    selected: ThemeId,
    colors: ThemeColors,
    on_select: EventHandler<ThemeId>,
) -> Element {
    rsx! {
        div {
            margin_bottom: "20px",

            label {
                display: "block",
                color: "{colors.text_secondary}",
                font_size: "12px",
                margin_bottom: "8px",

                "{label}"
            }

            div {
                display: "flex",
                flex_wrap: "wrap",
                gap: "8px",

                for theme_id in options {
                    {
                        let is_selected = selected == theme_id;
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
                                onclick: move |_| on_select.call(theme_id),

                                "{theme_id.label()}"
                            }
                        }
                    }
                }
            }
        }
    }
}
