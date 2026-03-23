use crate::config::AppSettings;
use crate::theme::{ThemeColors, ThemeId, ThemePreference};
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;

#[component]
pub fn SettingsDialog(
    settings: AppSettings,
    colors: ThemeColors,
    resolved_theme_id: ThemeId,
    on_save: EventHandler<AppSettings>,
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

    let current_theme_label = if use_system_theme() {
        format!("跟随系统，当前解析为 {}", resolved_theme_id.label())
    } else {
        format!("手动选择：{}", manual_theme().label())
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "400px".to_string(),

            h2 {
                color: "{colors.text}",
                margin_bottom: "24px",
                font_size: "20px",

                "设置"
            }

            div {
                margin_bottom: "24px",

                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    margin_bottom: "8px",

                    "主题模式"
                }

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "12px",

                    select {
                        flex: "1",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        onchange: move |e| {
                            use_system_theme.set(e.value() == "system");
                        },

                        option {
                            value: "system",
                            selected: use_system_theme(),
                            "跟随系统"
                        }
                        option {
                            value: "manual",
                            selected: !use_system_theme(),
                            "手动选择"
                        }
                    }

                    span {
                        color: "{colors.accent}",
                        font_size: "12px",

                        "{current_theme_label}"
                    }
                }

                if !use_system_theme() {
                    div {
                        margin_top: "12px",

                        div {
                            color: "{colors.text_secondary}",
                            font_size: "13px",
                            margin_bottom: "8px",

                            "手动主题"
                        }

                        select {
                            width: "100%",
                            padding: "8px 12px",
                            background: "{colors.background_tertiary}",
                            border: "1px solid {colors.border}",
                            border_radius: "4px",
                            color: "{colors.text}",
                            font_size: "13px",
                            onchange: move |e| {
                                if let Some(theme_id) = ThemeId::from_str(&e.value()) {
                                    manual_theme.set(theme_id);
                                }
                            },

                            for theme_id in ThemeId::MANUAL_OPTIONS {
                                option {
                                    value: "{theme_id.as_str()}",
                                    selected: manual_theme() == theme_id,
                                    "{theme_id.label()}"
                                }
                            }
                        }
                    }
                }
            }

            div {
                margin_bottom: "24px",

                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    margin_bottom: "8px",

                    "服务器信息自动刷新"
                }

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "12px",

                    select {
                        flex: "1",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse() {
                                auto_refresh_interval.set(v);
                            }
                        },

                        option { value: "0", selected: auto_refresh_interval() == 0, "关闭" }
                        option { value: "5", selected: auto_refresh_interval() == 5, "5 秒" }
                        option { value: "10", selected: auto_refresh_interval() == 10, "10 秒" }
                        option { value: "30", selected: auto_refresh_interval() == 30, "30 秒" }
                        option { value: "60", selected: auto_refresh_interval() == 60, "60 秒" }
                    }

                    if auto_refresh_interval() > 0 {
                        span {
                            color: "{colors.accent}",
                            font_size: "12px",

                            "每 {auto_refresh_interval} 秒刷新"
                        }
                    }
                }
            }

            div {
                    display: "flex",
                    justify_content: "flex_end",
                    gap: "12px",
                    margin_top: "20px",

                    button {
                        padding: "8px 16px",
                        background: "{colors.background_tertiary}",
                        color: "{colors.text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| on_close.call(()),

                        "取消"
                    }

                    button {
                        padding: "8px 16px",
                        background: "{colors.primary}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| {
                            on_save.call(AppSettings {
                                auto_refresh_interval: auto_refresh_interval(),
                                theme_preference: if use_system_theme() {
                                    ThemePreference::System
                                } else {
                                    ThemePreference::Manual(manual_theme())
                                },
                                window_width: settings.window_width,
                                window_height: settings.window_height,
                                window_x: settings.window_x,
                                window_y: settings.window_y,
                            });
                            on_close.call(());
                        },

                        "保存"
                    }
                }
        }
    }
}
