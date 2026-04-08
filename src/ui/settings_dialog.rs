use crate::config::AppSettings;
use crate::theme::{ThemeColors, ThemeId, ThemeMode, ThemePreference};
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::icons::{
    IconCheck, IconDownload, IconExternalLink, IconGitHub, IconGlobe, IconHelpCircle, IconRefresh,
    IconStar, IconX,
};
use crate::updater::{get_current_version, trigger_manual_check, UPDATE_STATUS};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use dioxus::prelude::*;
use once_cell::sync::Lazy;

const GITHUB_REPO_URL: &str = "https://github.com/yelog/rust-redis-desktop";
const GITHUB_RELEASES_URL: &str = "https://github.com/yelog/rust-redis-desktop/releases";
const GITHUB_ISSUES_URL: &str = "https://github.com/yelog/rust-redis-desktop/issues";

static ABOUT_ICON_DATA_URI: Lazy<String> = Lazy::new(|| {
    format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(include_bytes!("../../icons/icon.png"))
    )
});

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
                            let pending_version = update_status
                                .pending_update
                                .as_ref()
                                .map(|info| info.version.clone());
                            let channel_label = about_release_channel(&version);

                            rsx! {
                                div {
                                    display: "flex",
                                    flex_direction: "column",
                                    gap: "16px",

                                    AboutHero {
                                        version: version.clone(),
                                        channel_label,
                                        colors,
                                    }

                                    AboutStarCard { colors }

                                    AboutSectionCard {
                                        title: "项目资源",
                                        description: "查看源码、版本发布与问题反馈入口。",
                                        colors,

                                        AboutLinkRow {
                                            title: "项目主页",
                                            description: "查看源码、路线图和项目动态",
                                            href: GITHUB_REPO_URL,
                                            colors,
                                            icon: rsx! {
                                                IconGlobe { size: Some(18), color: Some(colors.accent.to_string()) }
                                            },
                                        }

                                        AboutLinkRow {
                                            title: "版本发布",
                                            description: "下载最新版本并查看发布说明",
                                            href: GITHUB_RELEASES_URL,
                                            colors,
                                            icon: rsx! {
                                                IconDownload { size: Some(18), color: Some(colors.accent.to_string()) }
                                            },
                                        }

                                        AboutLinkRow {
                                            title: "问题反馈",
                                            description: "提交 Bug、建议或跟进已知问题",
                                            href: GITHUB_ISSUES_URL,
                                            colors,
                                            icon: rsx! {
                                                IconHelpCircle { size: Some(18), color: Some(colors.warning.to_string()) }
                                            },
                                        }
                                    }

                                    AboutUpdateCard {
                                        colors,
                                        auto_check_updates: auto_check_updates(),
                                        checking,
                                        pending_version,
                                        on_set_auto_check: {
                                            let apply = apply_settings.clone();
                                            move |value| {
                                                auto_check_updates.set(value);
                                                apply();
                                            }
                                        },
                                        on_manual_check: move |_| trigger_manual_check(),
                                    }

                                    div {
                                        display: "flex",
                                        flex_direction: "column",
                                        align_items: "center",
                                        gap: "4px",
                                        padding: "4px 0 2px 0",
                                        text_align: "center",

                                        div {
                                            color: "{colors.text_subtle}",
                                            font_size: "12px",

                                            "MIT License | yelog"
                                        }

                                        div {
                                            color: "{colors.text_subtle}",
                                            font_size: "11px",

                                            "Built with Rust, Dioxus, and Freya"
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
}

fn about_release_channel(version: &str) -> &'static str {
    if version.contains("beta") {
        "Beta"
    } else {
        "Stable"
    }
}

#[component]
fn AboutHero(version: String, channel_label: &'static str, colors: ThemeColors) -> Element {
    rsx! {
        div {
            padding: "22px",
            background: "{colors.background_secondary}",
            border: "1px solid {colors.border}",
            border_radius: "18px",
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            gap: "14px",
            box_shadow: "0 6px 18px rgba(0, 0, 0, 0.10)",

            div {
                width: "80px",
                height: "80px",
                padding: "12px",
                background: "{colors.background_tertiary}",
                border: "1px solid {colors.outline_variant}",
                border_radius: "20px",
                display: "flex",
                align_items: "center",
                justify_content: "center",

                img {
                    src: "{ABOUT_ICON_DATA_URI.as_str()}",
                    width: "56px",
                    height: "56px",
                    border_radius: "14px",
                }
            }

            div {
                display: "flex",
                flex_direction: "column",
                align_items: "center",
                gap: "6px",
                text_align: "center",

                h2 {
                    margin: "0",
                    color: "{colors.text}",
                    font_size: "26px",
                    font_weight: "700",
                    line_height: "1.1",

                    "Rust Redis Desktop"
                }

                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    font_weight: "500",

                    "连接、浏览、编辑和分析 Redis 数据的一体化桌面工具"
                }
            }

            div {
                display: "flex",
                flex_wrap: "wrap",
                justify_content: "center",
                gap: "8px",

                AboutBadge {
                    label: format!("v{version}"),
                    colors,
                    emphasized: true,
                }

                AboutBadge {
                    label: channel_label.to_string(),
                    colors,
                    emphasized: false,
                }

                AboutBadge {
                    label: "Open Source".to_string(),
                    colors,
                    emphasized: false,
                }
            }

            div {
                max_width: "420px",
                color: "{colors.text_secondary}",
                font_size: "13px",
                line_height: "1.6",
                text_align: "center",

                "支持多连接管理、数据查看与编辑、命令执行、监控分析等 Redis 日常工作流。"
            }
        }
    }
}

#[component]
fn AboutStarCard(colors: ThemeColors) -> Element {
    rsx! {
        div {
            padding: "20px",
            background: "{colors.background_secondary}",
            border: "1px solid {colors.outline_variant}",
            border_radius: "18px",
            display: "flex",
            flex_direction: "column",
            gap: "14px",
            box_shadow: "0 10px 24px rgba(0, 0, 0, 0.12)",

            div {
                display: "flex",
                align_items: "center",
                gap: "10px",

                div {
                    width: "34px",
                    height: "34px",
                    background: "{colors.primary}",
                    border_radius: "17px",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",

                    IconStar { size: Some(18), color: Some(colors.primary_text.to_string()) }
                }

                div {
                    color: "{colors.text_secondary}",
                    font_size: "12px",
                    font_weight: "600",

                    "支持项目"
                }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "6px",

                div {
                    color: "{colors.text}",
                    font_size: "18px",
                    font_weight: "700",
                    line_height: "1.3",

                    "这个项目对你有帮助？欢迎点个 Star"
                }

                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    line_height: "1.6",

                    "你的支持会直接帮助项目持续迭代，也能让更多开发者更快发现并使用它。"
                }
            }

            AboutActionButton {
                label: "前往 GitHub 点 Star".to_string(),
                colors,
                icon: rsx! {
                    IconGitHub { size: Some(16), color: Some(colors.primary_text.to_string()) }
                },
                primary: true,
                disabled: false,
                on_click: move |_| {
                    let _ = open::that(GITHUB_REPO_URL);
                },
            }
        }
    }
}

#[component]
fn AboutUpdateCard(
    colors: ThemeColors,
    auto_check_updates: bool,
    checking: bool,
    pending_version: Option<String>,
    on_set_auto_check: EventHandler<bool>,
    on_manual_check: EventHandler<()>,
) -> Element {
    let enable_auto_check = on_set_auto_check.clone();
    let disable_auto_check = on_set_auto_check.clone();

    rsx! {
        AboutSectionCard {
            title: "版本更新",
            description: "保留当前版本策略，按需检查新版本。",
            colors,

            div {
                display: "flex",
                flex_direction: "column",
                gap: "14px",

                div {
                    display: "flex",
                    align_items: "center",
                    justify_content: "space_between",
                    flex_wrap: "wrap",
                    gap: "12px",

                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: "4px",

                        div {
                            color: "{colors.text}",
                            font_size: "14px",
                            font_weight: "600",

                            "自动检查更新"
                        }

                        div {
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            line_height: "1.5",

                            "应用启动后自动检查是否有新版本可用。"
                        }
                    }

                    div {
                        display: "flex",
                        flex_wrap: "wrap",
                        gap: "8px",

                        ChoiceChip {
                            label: "开启",
                            selected: auto_check_updates,
                            colors,
                            compact: true,
                            inactive_bg: Some(colors.background_tertiary),
                            on_click: move |_| enable_auto_check.call(true),
                        }

                        ChoiceChip {
                            label: "关闭",
                            selected: !auto_check_updates,
                            colors,
                            compact: true,
                            inactive_bg: Some(colors.background_tertiary),
                            on_click: move |_| disable_auto_check.call(false),
                        }
                    }
                }

                div {
                    height: "1px",
                    background: "{colors.border}",
                }

                div {
                    display: "flex",
                    align_items: "center",
                    justify_content: "space_between",
                    flex_wrap: "wrap",
                    gap: "12px",

                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: "4px",

                        div {
                            color: "{colors.text}",
                            font_size: "14px",
                            font_weight: "600",

                            "手动检查更新"
                        }

                        div {
                            color: "{colors.text_secondary}",
                            font_size: "12px",
                            line_height: "1.5",

                            "需要时立即拉取最新版本信息。"
                        }
                    }

                    AboutActionButton {
                        label: if checking {
                            "检查中...".to_string()
                        } else {
                            "检查更新".to_string()
                        },
                        colors,
                        icon: rsx! {
                            IconRefresh {
                                size: Some(16),
                                color: Some(if checking {
                                    colors.text_secondary.to_string()
                                } else {
                                    colors.text.to_string()
                                }),
                            }
                        },
                        primary: false,
                        disabled: checking,
                        on_click: move |_| on_manual_check.call(()),
                    }
                }

                if let Some(version) = pending_version {
                    div {
                        padding: "10px 12px",
                        background: "{colors.success_bg}",
                        border: "1px solid {colors.success}",
                        border_radius: "12px",
                        display: "flex",
                        align_items: "center",
                        gap: "8px",

                        IconCheck { size: Some(16), color: Some(colors.success.to_string()) }

                        div {
                            color: "{colors.success}",
                            font_size: "13px",
                            font_weight: "600",

                            "发现新版本 v{version}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AboutSectionCard(
    title: &'static str,
    description: &'static str,
    colors: ThemeColors,
    children: Element,
) -> Element {
    rsx! {
        div {
            padding: "18px",
            background: "{colors.background_secondary}",
            border: "1px solid {colors.border}",
            border_radius: "16px",
            display: "flex",
            flex_direction: "column",
            gap: "14px",

            div {
                display: "flex",
                flex_direction: "column",
                gap: "4px",

                div {
                    color: "{colors.text}",
                    font_size: "16px",
                    font_weight: "700",

                    "{title}"
                }

                div {
                    color: "{colors.text_secondary}",
                    font_size: "12px",
                    line_height: "1.5",

                    "{description}"
                }
            }

            {children}
        }
    }
}

#[component]
fn AboutLinkRow(
    title: &'static str,
    description: &'static str,
    href: &'static str,
    colors: ThemeColors,
    icon: Element,
) -> Element {
    let mut hover = use_signal(|| false);
    let border_color = if hover() {
        colors.outline_variant
    } else {
        colors.border
    };

    rsx! {
        button {
            width: "100%",
            padding: "12px 14px",
            background: if hover() {
                colors.background_tertiary
            } else {
                colors.background_secondary
            },
            border: "1px solid {border_color}",
            border_radius: "12px",
            display: "flex",
            align_items: "center",
            justify_content: "space_between",
            gap: "12px",
            cursor: "pointer",
            text_align: "left",
            transition: "background 0.2s, border 0.2s, box-shadow 0.2s",
            box_shadow: if hover() {
                "0 6px 18px rgba(0, 0, 0, 0.08)"
            } else {
                "none"
            },
            onmouseenter: move |_| hover.set(true),
            onmouseleave: move |_| hover.set(false),
            onclick: move |_| {
                let _ = open::that(href);
            },

            div {
                display: "flex",
                align_items: "center",
                gap: "12px",
                min_width: "0",

                div {
                    width: "36px",
                    height: "36px",
                    flex_shrink: "0",
                    background: "{colors.background_tertiary}",
                    border: "1px solid {colors.border}",
                    border_radius: "10px",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",

                    {icon}
                }

                div {
                    display: "flex",
                    flex_direction: "column",
                    gap: "4px",
                    min_width: "0",

                    div {
                        color: "{colors.text}",
                        font_size: "14px",
                        font_weight: "600",
                        line_height: "1.3",

                        "{title}"
                    }

                    div {
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        line_height: "1.5",

                        "{description}"
                    }
                }
            }

            IconExternalLink {
                size: Some(16),
                color: Some(if hover() {
                    colors.text.to_string()
                } else {
                    colors.text_subtle.to_string()
                }),
            }
        }
    }
}

#[component]
fn AboutBadge(label: String, colors: ThemeColors, emphasized: bool) -> Element {
    let background = if emphasized {
        colors.primary
    } else {
        colors.background_tertiary
    };
    let border_color = if emphasized {
        colors.primary
    } else {
        colors.border
    };
    let text_color = if emphasized {
        colors.primary_text
    } else {
        colors.text_secondary
    };

    rsx! {
        div {
            padding: "5px 10px",
            background: "{background}",
            border: "1px solid {border_color}",
            border_radius: "999px",
            color: "{text_color}",
            font_size: "11px",
            font_weight: "600",
            line_height: "1",
            white_space: "nowrap",

            "{label}"
        }
    }
}

#[component]
fn AboutActionButton(
    label: String,
    colors: ThemeColors,
    icon: Element,
    primary: bool,
    disabled: bool,
    on_click: EventHandler<()>,
) -> Element {
    let mut hover = use_signal(|| false);
    let background = if primary {
        colors.primary
    } else if hover() && !disabled {
        colors.background_tertiary
    } else {
        colors.background_secondary
    };
    let border_color = if primary {
        colors.primary
    } else if hover() && !disabled {
        colors.outline_variant
    } else {
        colors.border
    };
    let text_color = if primary {
        colors.primary_text
    } else if disabled {
        colors.text_secondary
    } else {
        colors.text
    };

    rsx! {
        button {
            padding: "10px 16px",
            min_height: "38px",
            background: "{background}",
            border: "1px solid {border_color}",
            border_radius: "10px",
            display: "flex",
            align_items: "center",
            justify_content: "center",
            gap: "8px",
            cursor: if disabled { "not_allowed" } else { "pointer" },
            color: "{text_color}",
            font_size: "13px",
            font_weight: "600",
            opacity: if disabled { "0.7" } else { "1" },
            transition: "background 0.2s, border 0.2s, box-shadow 0.2s",
            box_shadow: if hover() && !disabled {
                if primary {
                    "0 8px 18px rgba(0, 0, 0, 0.16)"
                } else {
                    "0 4px 12px rgba(0, 0, 0, 0.08)"
                }
            } else {
                "none"
            },
            disabled: disabled,
            onmouseenter: move |_| hover.set(true),
            onmouseleave: move |_| hover.set(false),
            onclick: move |_| {
                if !disabled {
                    on_click.call(());
                }
            },

            {icon}
            div { "{label}" }
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
