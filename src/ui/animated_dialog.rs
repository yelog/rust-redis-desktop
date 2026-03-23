use crate::theme::ThemeColors;
use crate::ui::animation_utils::prefers_reduced_motion;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum DialogAnimationState {
    Closed,
    Open,
}

#[component]
pub fn AnimatedDialog(
    is_open: bool,
    on_close: EventHandler<()>,
    colors: ThemeColors,
    width: Option<String>,
    max_height: Option<String>,
    children: Element,
) -> Element {
    let width_val = width.unwrap_or_else(|| "450px".to_string());
    let max_height_val = max_height.unwrap_or_else(|| "90vh".to_string());

    if !is_open {
        return rsx! {};
    }

    let transition_style = if prefers_reduced_motion() {
        "none".to_string()
    } else {
        "opacity 150ms ease-out, transform 200ms ease-out".to_string()
    };

    let backdrop_color = colors.overlay_backdrop;

    rsx! {
        div {
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
            onclick: move |_| on_close.call(()),

            div {
                width: "{width_val}",
                max_height: "{max_height_val}",
                padding: "24px",
                background: "{colors.background}",
                border_radius: "8px",
                box_shadow: "0 4px 24px rgba(0, 0, 0, 0.5)",
                overflow_y: "auto",
                transition: "{transition_style}",

                onclick: move |evt| {
                    evt.stop_propagation();
                },

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
