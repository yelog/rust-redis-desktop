use crate::config::AppSettings;
use crate::theme::{ThemeColors, ThemeId, ThemeMode, ThemePreference};
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::icons::IconX;
use crate::updater::{get_current_version, trigger_manual_check, UPDATE_STATUS};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
enum SettingsTab {
    #[default]
    General,
    Appearance,
    About,
}

impl SettingsTab {
    fn label(self) -> &'static str {
        match self {
            Self::General => "通用",
            Self::Appearance => "外观",
            Self::About => "关于",
        }
    }
}

#[component]
pub fn SettingsDialog(
    settings: AppSettings,
    colors: ThemeColors,
    resolved_theme_id: ThemeId,
    on_change: EventHandler<AppSettings>,
    on_close: EventHandler<()>,
) -> Element {
    let _ = resolved_theme_id;
    let mut current_tab = use_signal(SettingsTab::default);
    let mut auto_refresh_interval = use_signal(|| settings.auto_refresh_interval);
    let mut theme_mode = use_signal(|| settings.theme_preference.mode());
    let mut light_theme = use_signal(|| settings.theme_preference.light_theme());
    let mut dark_theme = use_signal(|| settings.theme_preference.dark_theme());
    let mut auto_check_updates = use_signal(|| settings.auto_check_updates);
    let close_button = on_close.clone();

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
                auto_check_updates: auto_check_updates(),
            });
        }
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "560px".to_string(),
            title: "".to_string(),
            show_close_button: false,

            div {
                display: "flex",
                flex_direction: "column",
                gap: "18px",

                {
                    rsx! {
                        div {
                            display: "flex",
                            align_items: "center",
                            justify_content: "space-between",
                            gap: "12px",

                            div {
                                display: "flex",
                                align_items: "center",
                                flex_wrap: "wrap",
                                gap: "12px",
                                min_width: "0",

                                div {
                                    color: "{colors.text_secondary}",
                                    font_size: "13px",
                                    font_weight: "600",
                                    line_height: "1",
                                    white_space: "nowrap",

                                    "设置"
                                }

                                div {
                                    display: "flex",
                                    align_items: "center",
                                    flex_wrap: "wrap",
                                    gap: "4px",
                                    padding: "4px",
                                    background: "{colors.background_secondary}",
                                    border: "1px solid {colors.border}",
                                    border_radius: "10px",

                                    for tab in [SettingsTab::General, SettingsTab::Appearance, SettingsTab::About] {
                                        SettingsTabButton {
                                            label: tab.label(),
                                            active: current_tab() == tab,
                                            colors,
                                            on_click: move |_| current_tab.set(tab),
                                        }
                                    }
                                }
                            }

                            button {
                                width: "28px",
                                height: "28px",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                flex_shrink: "0",
                                padding: "0",
                                background: "{colors.background_secondary}",
                                border: "1px solid {colors.border}",
                                border_radius: "8px",
                                cursor: "pointer",
                                title: "关闭",
                                onclick: move |_| close_button.call(()),

                                IconX { size: Some(14), color: Some(colors.text_secondary.to_string()) }
                            }
                        }
                    }
                }

                div {
                    height: "1px",
                    background: "{colors.border}",
                }

                {
                    match current_tab() {
                        SettingsTab::General => rsx! {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "16px",

                                SettingsGroup {
                                    label: "服务器信息自动刷新",
                                    colors,

                                    div {
                                        display: "flex",
                                        flex_wrap: "wrap",
                                        gap: "8px",

                                        for (value, label) in [(0, "关闭"), (5, "5秒"), (10, "10秒"), (30, "30秒"), (60, "60秒")] {
                                            ChoiceChip {
                                                label,
                                                selected: auto_refresh_interval() == value,
                                                colors,
                                                on_click: {
                                                    let apply = apply_settings.clone();
                                                    move |_| {
                                                        auto_refresh_interval.set(value);
                                                        apply();
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }

                                SettingsGroup {
                                    label: "自动检查更新",
                                    colors,

                                    div {
                                        display: "flex",
                                        flex_wrap: "wrap",
                                        gap: "8px",

                                        for (value, label) in [(true, "开启"), (false, "关闭")] {
                                            ChoiceChip {
                                                label,
                                                selected: auto_check_updates() == value,
                                                colors,
                                                on_click: {
                                                    let apply = apply_settings.clone();
                                                    move |_| {
                                                        auto_check_updates.set(value);
                                                        apply();
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        SettingsTab::Appearance => rsx! {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "16px",

                                SettingsGroup {
                                    label: "主题模式",
                                    colors,

                                    div {
                                        display: "flex",
                                        flex_wrap: "wrap",
                                        gap: "8px",

                                        for (mode, label) in [
                                            (ThemeMode::System, "跟随系统"),
                                            (ThemeMode::Dark, "暗色"),
                                            (ThemeMode::Light, "亮色"),
                                        ] {
                                            ChoiceChip {
                                                label,
                                                selected: theme_mode() == mode,
                                                colors,
                                                on_click: {
                                                    let apply = apply_settings.clone();
                                                    move |_| {
                                                        theme_mode.set(mode);
                                                        apply();
                                                    }
                                                },
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
                            }
                        },
                        SettingsTab::About => {
                            let update_status = UPDATE_STATUS();
                            let version = get_current_version();
                            let checking = update_status.checking;

                            rsx! {
                                div {
                                    display: "flex",
                                    flex_direction: "column",
                                    gap: "20px",

                                    div {
                                        display: "flex",
                                        align_items: "center",
                                        gap: "12px",

                                        div {
                                            font_size: "34px",
                                            line_height: "1",
                                            "🚀"
                                        }

                                        div {
                                            h2 {
                                                margin: "0",
                                                color: "{colors.text}",
                                                font_size: "20px",
                                                font_weight: "600",

                                                "Redis Desktop"
                                            }

                                            div {
                                                color: "{colors.text_secondary}",
                                                font_size: "13px",
                                                margin_top: "4px",

                                                "版本 {version}"
                                            }
                                        }
                                    }

                                    div {
                                        color: "{colors.text_secondary}",
                                        font_size: "13px",
                                        line_height: "1.6",

                                        "一个用 Rust 编写的 Redis 桌面管理工具。支持多数据库管理、数据可视化、命令执行等功能。"
                                    }

                                    div {
                                        padding: "14px",
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "10px",
                                        display: "flex",
                                        flex_direction: "column",
                                        gap: "14px",

                                        div {
                                            display: "flex",
                                            align_items: "center",
                                            justify_content: "space_between",
                                            gap: "12px",

                                            label {
                                                color: "{colors.text}",
                                                font_size: "14px",
                                                font_weight: "500",

                                                "自动更新"
                                            }

                                            div {
                                                display: "flex",
                                                flex_wrap: "wrap",
                                                gap: "8px",

                                                for (value, label) in [(true, "开启"), (false, "关闭")] {
                                                    ChoiceChip {
                                                        label,
                                                        selected: auto_check_updates() == value,
                                                        colors,
                                                        compact: true,
                                                        inactive_bg: Some(colors.background_secondary),
                                                        on_click: {
                                                            let apply = apply_settings.clone();
                                                            move |_| {
                                                                auto_check_updates.set(value);
                                                                apply();
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        }

                                        div {
                                            display: "flex",
                                            align_items: "center",
                                            gap: "12px",

                                            button {
                                                padding: "8px 16px",
                                                background: "{colors.primary}",
                                                color: "{colors.primary_text}",
                                                border: "none",
                                                border_radius: "8px",
                                                cursor: if checking { "not_allowed" } else { "pointer" },
                                                font_size: "13px",
                                                opacity: if checking { "0.6" } else { "1" },
                                                disabled: checking,
                                                onclick: move |_| trigger_manual_check(),

                                                if checking {
                                                    "检查中..."
                                                } else {
                                                    "检查更新"
                                                }
                                            }

                                            if let Some(info) = update_status.pending_update {
                                                div {
                                                    color: "{colors.success}",
                                                    font_size: "13px",

                                                    "发现新版本 v{info.version}"
                                                }
                                            }
                                        }
                                    }

                                    div {
                                        display: "flex",
                                        flex_wrap: "wrap",
                                        gap: "16px",

                                        a {
                                            href: "https://github.com/yelog/rust-redis-desktop",
                                            target: "_blank",
                                            color: "{colors.accent}",
                                            font_size: "13px",
                                            text_decoration: "none",

                                            "GitHub"
                                        }

                                        a {
                                            href: "https://github.com/yelog/rust-redis-desktop/releases",
                                            target: "_blank",
                                            color: "{colors.accent}",
                                            font_size: "13px",
                                            text_decoration: "none",

                                            "下载"
                                        }

                                        a {
                                            href: "https://github.com/yelog/rust-redis-desktop/issues",
                                            target: "_blank",
                                            color: "{colors.accent}",
                                            font_size: "13px",
                                            text_decoration: "none",

                                            "反馈"
                                        }
                                    }

                                    div {
                                        color: "{colors.text_subtle}",
                                        font_size: "12px",

                                        "MIT License © 2024 yelog"
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
fn SettingsTabButton(
    label: &'static str,
    active: bool,
    colors: ThemeColors,
    on_click: EventHandler<()>,
) -> Element {
    let background = if active {
        colors.background_tertiary
    } else {
        "transparent"
    };
    let border_color = if active {
        colors.outline_variant
    } else {
        "transparent"
    };
    let text_color = if active {
        colors.text
    } else {
        colors.text_secondary
    };

    rsx! {
        button {
            padding: "6px 12px",
            min_height: "30px",
            background: "{background}",
            border: "1px solid {border_color}",
            border_radius: "8px",
            color: "{text_color}",
            font_size: "13px",
            font_weight: if active { "600" } else { "500" },
            line_height: "1",
            cursor: "pointer",
            user_select: "none",
            white_space: "nowrap",
            onclick: move |_| on_click.call(()),

            "{label}"
        }
    }
}

#[component]
fn SettingsGroup(label: &'static str, colors: ThemeColors, children: Element) -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "8px",

            label {
                display: "block",
                color: "{colors.text_secondary}",
                font_size: "12px",
                font_weight: "500",
                line_height: "1.4",

                "{label}"
            }

            {children}
        }
    }
}

#[component]
fn ChoiceChip(
    label: &'static str,
    selected: bool,
    colors: ThemeColors,
    on_click: EventHandler<()>,
    compact: Option<bool>,
    inactive_bg: Option<&'static str>,
) -> Element {
    let compact = compact.unwrap_or(false);
    let background = if selected {
        colors.primary
    } else {
        inactive_bg.unwrap_or(colors.background_tertiary)
    };
    let border_color = if selected {
        colors.primary
    } else {
        colors.border
    };
    let text_color = if selected {
        colors.primary_text
    } else {
        colors.text
    };
    let padding = if compact { "5px 12px" } else { "6px 12px" };
    let min_height = if compact { "28px" } else { "30px" };
    let border_radius = if compact { "12px" } else { "14px" };
    let font_size = if compact { "12px" } else { "13px" };

    rsx! {
        button {
            padding: "{padding}",
            min_height: "{min_height}",
            background: "{background}",
            border: "1px solid {border_color}",
            border_radius: "{border_radius}",
            color: "{text_color}",
            font_size: "{font_size}",
            line_height: "1",
            cursor: "pointer",
            user_select: "none",
            white_space: "nowrap",
            onclick: move |_| on_click.call(()),

            "{label}"
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
        SettingsGroup {
            label,
            colors,
            div {
                display: "flex",
                flex_wrap: "wrap",
                gap: "8px",

                for theme_id in options {
                    ChoiceChip {
                        label: theme_id.label(),
                        selected: selected == theme_id,
                        colors,
                        on_click: move |_| on_select.call(theme_id),
                    }
                }
            }
        }
    }
}
