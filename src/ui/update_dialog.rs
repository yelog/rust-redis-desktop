use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use crate::updater::UpdateInfo;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum UpdateDialogState {
    #[default]
    Ready,
    Downloading,
    Completed,
    Error,
}

#[component]
pub fn UpdateDialog(
    update_info: UpdateInfo,
    colors: ThemeColors,
    on_update: EventHandler<()>,
    on_skip: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut state = use_signal(UpdateDialogState::default);
    let mut progress = use_signal(|| (0u64, 0u64));
    let mut error_msg = use_signal(|| String::new());

    let format_size = |bytes: u64| {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{} KB", bytes / 1024)
        } else {
            format!("{} MB", bytes / (1024 * 1024))
        }
    };

    let progress_percent = move || {
        let (downloaded, total) = progress();
        if total > 0 {
            (downloaded * 100 / total) as u8
        } else {
            0
        }
    };

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "420px".to_string(),
            title: "发现新版本".to_string(),

            div {
                div {
                    margin_bottom: "16px",
                    display: "flex",
                    align_items: "center",
                    gap: "12px",

                    div {
                        font_size: "24px",
                        color: "{colors.success}",
                        "🎉"
                    }

                    div {
                        h3 {
                            margin: "0",
                            color: "{colors.text}",
                            font_size: "18px",
                            font_weight: "600",

                            "v{update_info.version}"
                        }

                        if update_info.is_beta {
                            span {
                                margin_left: "8px",
                                padding: "2px 8px",
                                background: "{colors.accent}",
                                color: "{colors.primary_text}",
                                font_size: "11px",
                                border_radius: "4px",

                                "Beta"
                            }
                        }
                    }
                }

                div {
                    margin_bottom: "16px",
                    padding: "12px",
                    background: "{colors.background_tertiary}",
                    border_radius: "6px",
                    max_height: "150px",
                    overflow_y: "auto",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "11px",
                        margin_bottom: "8px",

                        "更新内容"
                    }

                    div {
                        color: "{colors.text}",
                        font_size: "13px",
                        line_height: "1.5",
                        white_space: "pre_wrap",
                        word_break: "break_word",

                        "{update_info.release_notes}"
                    }
                }

                if state() == UpdateDialogState::Downloading {
                    div {
                        margin_bottom: "16px",

                        div {
                            display: "flex",
                            justify_content: "space_between",
                            margin_bottom: "8px",

                            span {
                                color: "{colors.text_secondary}",
                                font_size: "13px",

                                "正在下载..."
                            }

                            span {
                                color: "{colors.accent}",
                                font_size: "13px",

                                "{format_size(progress().0)} / {format_size(progress().1)}"
                            }
                        }

                        div {
                            width: "100%",
                            height: "8px",
                            background: "{colors.background_tertiary}",
                            border_radius: "4px",
                            overflow: "hidden",

                            div {
                                width: "{progress_percent()}%",
                                height: "100%",
                                background: "{colors.accent}",
                                border_radius: "4px",
                                transition: "width 0.2s ease",
                            }
                        }
                    }
                }

                if state() == UpdateDialogState::Error {
                    div {
                        margin_bottom: "16px",
                        padding: "12px",
                        background: "{colors.error_bg}",
                        border: "1px solid {colors.error}",
                        border_radius: "6px",
                        color: "{colors.error}",
                        font_size: "13px",

                        "下载失败: {error_msg()}"
                    }
                }

                if state() == UpdateDialogState::Completed {
                    div {
                        margin_bottom: "16px",
                        padding: "12px",
                        background: "{colors.success_bg}",
                        border_radius: "6px",
                        color: "{colors.success}",
                        font_size: "13px",

                        "下载完成，即将开始安装..."
                    }
                }

                div {
                    display: "flex",
                    gap: "12px",
                    justify_content: "flex_end",

                    if state() == UpdateDialogState::Ready {
                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let on_close = on_close.clone();
                                move |_| on_close.call(())
                            },

                            "稍后提醒"
                        }

                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text_secondary}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let version = update_info.version.clone();
                                let on_skip = on_skip.clone();
                                let on_close = on_close.clone();
                                move |_| {
                                    on_skip.call(version.clone());
                                    on_close.call(())
                                }
                            },

                            "跳过此版本"
                        }

                        button {
                            padding: "8px 16px",
                            background: "{colors.primary}",
                            color: "{colors.primary_text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let on_update = on_update.clone();
                                move |_| {
                                    state.set(UpdateDialogState::Downloading);
                                    on_update.call(())
                                }
                            },

                            "立即更新"
                        }
                    } else if state() == UpdateDialogState::Downloading {
                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "not_allowed",
                            font_size: "13px",
                            opacity: "0.6",
                            disabled: true,

                            "下载中..."
                        }
                    } else if state() == UpdateDialogState::Error {
                        button {
                            padding: "8px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: move |_| {
                                state.set(UpdateDialogState::Ready);
                                error_msg.set(String::new());
                            },

                            "重试"
                        }
                    }
                }
            }
        }
    }
}

pub fn use_update_progress() -> (
    Signal<(u64, u64)>,
    Signal<UpdateDialogState>,
    Signal<String>,
) {
    let progress = use_signal(|| (0u64, 0u64));
    let state = use_signal(UpdateDialogState::default);
    let error_msg = use_signal(|| String::new());
    (progress, state, error_msg)
}
