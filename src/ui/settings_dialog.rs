use crate::config::AppSettings;
use crate::theme::{ThemeColors, ThemeMode};
use dioxus::prelude::*;

#[component]
pub fn SettingsDialog(
    settings: AppSettings,
    colors: ThemeColors,
    on_save: EventHandler<AppSettings>,
    on_close: EventHandler<()>,
) -> Element {
    let mut auto_refresh_interval = use_signal(|| settings.auto_refresh_interval);
    let mut theme_mode = use_signal(|| settings.theme_mode);

    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            background: "rgba(0, 0, 0, 0.7)",
            display: "flex",
            align_items: "center",
            justify_content: "center",
            z_index: "1000",
            onclick: move |_| on_close.call(()),

            div {
                width: "400px",
                padding: "24px",
                background: "{colors.background}",
                border_radius: "8px",
                box_shadow: "0 4px 24px rgba(0, 0, 0, 0.5)",

                onclick: move |evt| {
                    evt.stop_propagation();
                },

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
                                let mode = match e.value().as_str() {
                                    "light" => ThemeMode::Light,
                                    "dark" => ThemeMode::Dark,
                                    _ => ThemeMode::System,
                                };
                                theme_mode.set(mode);
                            },

                            option {
                                value: "system",
                                selected: theme_mode() == ThemeMode::System,
                                "跟随系统"
                            }
                            option {
                                value: "light",
                                selected: theme_mode() == ThemeMode::Light,
                                "亮色"
                            }
                            option {
                                value: "dark",
                                selected: theme_mode() == ThemeMode::Dark,
                                "暗色"
                            }
                        }

                        span {
                            color: "{colors.accent}",
                            font_size: "12px",

                            match theme_mode() {
                                ThemeMode::Light => "🌞 亮色模式",
                                ThemeMode::Dark => "🌙 暗色模式",
                                ThemeMode::System => "💻 跟随系统",
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
                                theme_mode: theme_mode(),
                            });
                            on_close.call(());
                        },

                        "保存"
                    }
                }
            }
        }
    }
}
