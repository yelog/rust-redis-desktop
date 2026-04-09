use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::icons::IconX;
use dioxus::prelude::*;
use std::time::Duration;

pub const EXIT_ANIMATION_DURATION_MS: u64 = 200;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum VisibilityState {
    #[default]
    Hidden,
    Visible,
    Exiting,
}

pub fn use_exit_animation(is_open: Signal<bool>) -> (bool, bool) {
    let mut visibility = use_signal(VisibilityState::default);

    {
        let current = *visibility.read();
        if is_open() && current == VisibilityState::Hidden {
            visibility.set(VisibilityState::Visible);
        } else if !is_open() && current == VisibilityState::Visible {
            visibility.set(VisibilityState::Exiting);
            let mut vis = visibility.clone();
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                vis.set(VisibilityState::Hidden);
            });
        }
    }

    let state = *visibility.read();
    let should_show = is_open() || state == VisibilityState::Exiting;
    let is_exiting = state == VisibilityState::Exiting;

    (should_show, is_exiting)
}

#[component]
pub fn AnimatedDialog(
    is_open: bool,
    on_close: EventHandler<()>,
    colors: ThemeColors,
    width: Option<String>,
    max_height: Option<String>,
    title: Option<String>,
    show_close_button: Option<bool>,
    scrollable_body: Option<bool>,
    children: Element,
) -> Element {
    let i18n = use_i18n();
    let width_val = width.unwrap_or_else(|| "450px".to_string());
    let max_height_val = max_height.unwrap_or_else(|| "90vh".to_string());
    let title_text = title.unwrap_or_default();
    let has_title = !title_text.is_empty();
    let show_close = show_close_button.unwrap_or(true);
    let body_scrollable = scrollable_body.unwrap_or(true);

    let mut visibility = use_signal(VisibilityState::default);
    let backdrop_color = colors.overlay_backdrop;
    let dialog_id = use_signal(|| format!("dialog-{}", uuid::Uuid::new_v4()));

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
            let on_close = on_close.clone();
            let mut visibility_clone = visibility.clone();
            spawn(async move {
                let mut eval = document::eval(
                    r#"
                    let handler = function(e) {
                        if (e.key === 'Escape') {
                            e.preventDefault();
                            e.stopPropagation();
                            dioxus.send('escape');
                        }
                    };
                    document.addEventListener('keydown', handler, true);
                    await new Promise(() => {});
                    "#,
                );
                tokio::time::sleep(Duration::from_millis(50)).await;
                let _ = document::eval(&format!("document.getElementById('{}')?.focus()", id));
                while let Ok(msg) = eval.recv::<String>().await {
                    if msg == "escape" {
                        let current = *visibility_clone.read();
                        if current == VisibilityState::Visible {
                            visibility_clone.set(VisibilityState::Exiting);
                            let mut vis = visibility_clone.clone();
                            let on_close = on_close.clone();
                            spawn(async move {
                                tokio::time::sleep(Duration::from_millis(
                                    EXIT_ANIMATION_DURATION_MS,
                                ))
                                .await;
                                vis.set(VisibilityState::Hidden);
                                on_close.call(());
                            });
                        }
                        break;
                    }
                }
            });
        }
    });

    let state = *visibility.read();
    if state == VisibilityState::Hidden {
        return rsx! {};
    }

    let is_exiting = state == VisibilityState::Exiting;
    let animation_name = if is_exiting {
        "modalFadeOut"
    } else {
        "modalFadeIn"
    };
    let backdrop_animation_name = if is_exiting {
        "backdropFadeOut"
    } else {
        "backdropFadeIn"
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
            animation: "{backdrop_animation_name} 0.2s ease-out forwards",
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
                background: "{colors.background}",
                border_radius: "8px",
                box_shadow: "0 4px 24px rgba(0, 0, 0, 0.5)",
                overflow: "hidden",
                animation: "{animation_name} 0.2s ease-out forwards",
                position: "relative",
                display: "flex",
                flex_direction: "column",
                onclick: move |evt| evt.stop_propagation(),

                style {
                    r#"
                    @keyframes backdropFadeIn {{
                        from {{
                            opacity: 0;
                        }}
                        to {{
                            opacity: 1;
                        }}
                    }}
                    @keyframes backdropFadeOut {{
                        from {{
                            opacity: 1;
                        }}
                        to {{
                            opacity: 0;
                        }}
                    }}
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

                if has_title {
                    div {
                        padding: "18px 24px 16px 24px",
                        display: "flex",
                        align_items: "center",
                        justify_content: "space-between",
                        gap: "16px",
                        border_bottom: "1px solid {colors.border}",

                        h2 {
                            flex: "1",
                            min_width: "0",
                            margin: "0",
                            color: "{colors.text}",
                            font_size: "22px",
                            font_weight: "700",
                            line_height: "1.2",
                            white_space: "nowrap",
                            text_overflow: "ellipsis",
                            overflow: "hidden",

                            "{title_text}"
                        }

                        if show_close {
                            button {
                                width: "30px",
                                height: "30px",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                flex_shrink: "0",
                                padding: "0",
                                background: "{colors.background_secondary}",
                                border: "1px solid {colors.border}",
                                border_radius: "8px",
                                box_shadow: "0 6px 18px rgba(0, 0, 0, 0.12)",
                                cursor: "pointer",
                                title: i18n.read().t("Close"),
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
                    }
                } else if show_close {
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
                        title: i18n.read().t("Close"),
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

                div {
                    padding: if has_title { "20px 24px 24px 24px" } else { "24px" },
                    display: "flex",
                    flex_direction: "column",
                    overflow_y: if body_scrollable { "auto" } else { "hidden" },
                    overflow_x: "hidden",
                    flex: "1",
                    min_height: "0",

                    {children}
                }
            }
        }
    }
}

#[component]
pub fn DialogHeader(title: String, colors: ThemeColors, icon: Option<String>) -> Element {
    rsx! {
        h2 {
            color: "{colors.text}",
            margin: "0",
            font_size: "22px",
            font_weight: "700",
            line_height: "1.2",
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
    let i18n = use_i18n();
    let cancel = cancel_text.unwrap_or_else(|| i18n.read().t("Cancel"));
    let confirm = confirm_text.unwrap_or_else(|| i18n.read().t("Confirm"));
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

#[component]
pub fn SimpleAnimatedModal(
    is_open: Signal<bool>,
    on_close: EventHandler<()>,
    width: Option<String>,
    children: Element,
) -> Element {
    let width_val = width.unwrap_or_else(|| "420px".to_string());
    let mut visibility = use_signal(VisibilityState::default);

    {
        let current = *visibility.read();
        if is_open() && current == VisibilityState::Hidden {
            visibility.set(VisibilityState::Visible);
        } else if !is_open() && current == VisibilityState::Visible {
            visibility.set(VisibilityState::Exiting);
            let mut vis = visibility.clone();
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                vis.set(VisibilityState::Hidden);
            });
        }
    }

    let state = *visibility.read();
    if state == VisibilityState::Hidden {
        return rsx! {};
    }

    let is_exiting = state == VisibilityState::Exiting;
    let backdrop_animation = if is_exiting {
        "backdropFadeOut"
    } else {
        "backdropFadeIn"
    };
    let modal_animation = if is_exiting {
        "modalFadeOut"
    } else {
        "modalFadeIn"
    };

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
            animation: "{backdrop_animation} 0.2s ease-out forwards",
            onclick: {
                let mut visibility = visibility.clone();
                move |_| {
                    if *visibility.read() == VisibilityState::Visible {
                        visibility.set(VisibilityState::Exiting);
                        let mut vis = visibility.clone();
                        let mut is_open_sig = is_open.clone();
                        spawn(async move {
                            tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                            vis.set(VisibilityState::Hidden);
                            is_open_sig.set(false);
                            on_close.call(());
                        });
                    }
                }
            },

            style {
                r#"
                @keyframes backdropFadeIn {{
                    from {{ opacity: 0; }}
                    to {{ opacity: 1; }}
                }}
                @keyframes backdropFadeOut {{
                    from {{ opacity: 1; }}
                    to {{ opacity: 0; }}
                }}
                @keyframes modalFadeIn {{
                    from {{ opacity: 0; transform: scale(0.9); }}
                    to {{ opacity: 1; transform: scale(1); }}
                }}
                @keyframes modalFadeOut {{
                    from {{ opacity: 1; transform: scale(1); }}
                    to {{ opacity: 0; transform: scale(0.9); }}
                }}
                "#
            }

            div {
                width: "{width_val}",
                background: "#1e1e1e",
                border_radius: "10px",
                padding: "20px",
                animation: "{modal_animation} 0.2s ease-out forwards",
                onclick: move |e| e.stop_propagation(),

                {children}
            }
        }
    }
}
