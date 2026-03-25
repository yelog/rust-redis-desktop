use crate::theme::ThemeColors;
use crate::ui::icons::IconX;
use dioxus::prelude::*;
use std::time::Duration;

const EXIT_ANIMATION_DURATION_MS: u64 = 200;

#[derive(Clone, Copy, PartialEq, Default)]
enum VisibilityState {
    #[default]
    Hidden,
    Visible,
    Exiting,
}

#[component]
pub fn AnimatedDialog(
    is_open: bool,
    on_close: EventHandler<()>,
    colors: ThemeColors,
    width: Option<String>,
    max_height: Option<String>,
    show_close_button: Option<bool>,
    children: Element,
) -> Element {
    let width_val = width.unwrap_or_else(|| "450px".to_string());
    let max_height_val = max_height.unwrap_or_else(|| "90vh".to_string());
    let show_close = show_close_button.unwrap_or(true);

    let mut visibility = use_signal(VisibilityState::default);
    let backdrop_color = colors.overlay_backdrop;
    let dialog_id = use_signal(|| format!("dialog-{}", uuid::Uuid::new_v4()));

    let should_show = is_open || *visibility.read() == VisibilityState::Exiting;

    {
        let current_visibility = *visibility.read();

        if is_open && current_visibility == VisibilityState::Hidden {
            visibility.set(VisibilityState::Visible);
        } else if !is_open && current_visibility == VisibilityState::Visible {
            visibility.set(VisibilityState::Exiting);

            let mut vis = visibility.clone();
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                vis.set(VisibilityState::Hidden);
            });
        }
    }

    use_effect(move || {
        if *visibility.read() == VisibilityState::Visible {
            let id = dialog_id();
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let _ = document::eval(&format!("document.getElementById('{}')?.focus()", id));
            });
        }
    });

    if !should_show {
        return rsx! {};
    }

    let state = *visibility.read();
    let is_exiting = state == VisibilityState::Exiting;
    let animation_name = if is_exiting {
        "modalFadeOut"
    } else {
        "modalFadeIn"
    };

    rsx! {
        div {
            id: "{dialog_id}",
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            background: "{backdrop_color}",
            display: "flex",
            align_items: "center",
            justify_content: "center",
            z_index: "1000",
            tabindex: "0",
            "data-dialog": "true",
            onclick: {
                let mut visibility = visibility.clone();
                let on_close = on_close.clone();
                move |_| {
                    let current = *visibility.read();
                    if current == VisibilityState::Visible {
                        visibility.set(VisibilityState::Exiting);
                        let mut vis = visibility.clone();
                        let on_close = on_close.clone();
                        spawn(async move {
                            tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                            vis.set(VisibilityState::Hidden);
                            on_close.call(());
                        });
                    }
                }
            },
            onkeydown: {
                let mut visibility = visibility.clone();
                let on_close = on_close.clone();
                move |e: Event<KeyboardData>| {
                    if e.data().key() == Key::Escape {
                        e.prevent_default();
                        e.stop_propagation();
                        let current = *visibility.read();
                        if current == VisibilityState::Visible {
                            visibility.set(VisibilityState::Exiting);
                            let mut vis = visibility.clone();
                            let on_close = on_close.clone();
                            spawn(async move {
                                tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                                vis.set(VisibilityState::Hidden);
                                on_close.call(());
                            });
                        }
                    }
                }
            },

            div {
                width: "{width_val}",
                max_height: "{max_height_val}",
                padding: "24px",
                background: "{colors.background}",
                border_radius: "8px",
                box_shadow: "0 4px 24px rgba(0, 0, 0, 0.5)",
                overflow_y: "auto",
                overflow_x: "hidden",
                animation: "{animation_name} 0.2s ease-out forwards",
                position: "relative",
                onclick: move |evt| evt.stop_propagation(),

                style {
                    r#"
                    @keyframes modalFadeIn {{
                        from {{
                            opacity: 0;
                            transform: scale(0.8);
                        }}
                        to {{
                            opacity: 1;
                            transform: scale(1);
                        }}
                    }}
                    @keyframes modalFadeOut {{
                        from {{
                            opacity: 1;
                            transform: scale(1);
                        }}
                        to {{
                            opacity: 0;
                            transform: scale(0.8);
                        }}
                    }}
                    "#
                }

                if show_close {
                    button {
                        position: "absolute",
                        top: "14px",
                        right: "14px",
                        z_index: "10",
                        width: "30px",
                        height: "30px",
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        padding: "0",
                        background: "{colors.background_secondary}",
                        border: "1px solid {colors.border}",
                        border_radius: "8px",
                        box_shadow: "0 6px 18px rgba(0, 0, 0, 0.12)",
                        cursor: "pointer",
                        onclick: {
                            let mut visibility = visibility.clone();
                            let on_close = on_close.clone();
                            move |_| {
                                let current = *visibility.read();
                                if current == VisibilityState::Visible {
                                    visibility.set(VisibilityState::Exiting);
                                    let mut vis = visibility.clone();
                                    let on_close = on_close.clone();
                                    spawn(async move {
                                        tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                                        vis.set(VisibilityState::Hidden);
                                        on_close.call(());
                                    });
                                }
                            }
                        },

                        IconX { size: Some(16), color: Some(colors.text_secondary.to_string()) }
                    }
                }

                {children}
            }
        }
    }
}

#[component]
pub fn DialogHeader(title: String, colors: ThemeColors, icon: Option<String>) -> Element {
    rsx! {
        h2 {
            color: "{colors.text}",
            margin_bottom: "20px",
            font_size: "20px",
            display: "flex",
            align_items: "center",
            gap: "8px",

            if let Some(icon_str) = icon {
                "{icon_str} "
            }
            "{title}"
        }
    }
}

#[component]
pub fn DialogFooter(
    colors: ThemeColors,
    cancel_text: Option<String>,
    confirm_text: Option<String>,
    on_cancel: EventHandler<()>,
    on_confirm: Option<EventHandler<()>>,
    confirm_disabled: Option<bool>,
    is_processing: Option<bool>,
    confirm_style: Option<String>,
) -> Element {
    let cancel = cancel_text.unwrap_or_else(|| "取消".to_string());
    let confirm = confirm_text.unwrap_or_else(|| "确定".to_string());
    let disabled = confirm_disabled.unwrap_or(false);
    let processing = is_processing.unwrap_or(false);
    let style = confirm_style.unwrap_or_else(|| colors.primary.to_string());

    rsx! {
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
                onclick: move |_| on_cancel.call(()),

                "{cancel}"
            }

            if let Some(handler) = on_confirm {
                button {
                    padding: "8px 16px",
                    background: "{style}",
                    color: "{colors.primary_text}",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    disabled: disabled || processing,
                    onclick: move |_| handler.call(()),

                    if processing {
                        "处理中..."
                    } else {
                        "{confirm}"
                    }
                }
            }
        }
    }
}
