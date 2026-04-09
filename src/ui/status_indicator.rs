use crate::connection::ConnectionState;
use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use dioxus::prelude::*;

#[component]
pub fn StatusIndicator(state: ConnectionState, colors: ThemeColors, size: Option<f32>) -> Element {
    let size = size.unwrap_or(10.0);

    let (color, glow_animation) = match state {
        ConnectionState::Connected => (colors.state_connected, "none"),
        ConnectionState::Connecting => (colors.state_connecting, "pulse 1.5s ease-in-out infinite"),
        ConnectionState::Disconnected => (colors.state_disconnected, "none"),
        ConnectionState::Error => (colors.state_error, "none"),
    };

    let glow_color = format!("0 0 {}px {}", size * 0.8, color);

    rsx! {
        div {
            width: "{size}px",
            height: "{size}px",
            border_radius: "50%",
            background: "{color}",
            box_shadow: "{glow_color}",
            flex_shrink: "0",
            transition: "background 300ms ease-in-out, box_shadow 300ms ease-in-out",

            style: r#"
                @keyframes pulse {{
                    0%, 100% {{ opacity: 1; transform: scale(1); }}
                    50% {{ opacity: 0.6; transform: scale(0.9); }}
                }}
            "#,
        }
    }
}

#[component]
pub fn StatusIndicatorWithLabel(
    state: ConnectionState,
    colors: ThemeColors,
    show_label: Option<bool>,
    size: Option<f32>,
) -> Element {
    let show_label = show_label.unwrap_or(true);
    let i18n = use_i18n();
    let label = match state {
        ConnectionState::Connected => i18n.read().t("Connected"),
        ConnectionState::Connecting => i18n.read().t("Connecting"),
        ConnectionState::Disconnected => i18n.read().t("Disconnected"),
        ConnectionState::Error => i18n.read().t("Connection error"),
    };

    let label_color = match state {
        ConnectionState::Connected => colors.state_connected,
        ConnectionState::Connecting => colors.state_connecting,
        ConnectionState::Disconnected => colors.text_subtle,
        ConnectionState::Error => colors.state_error,
    };

    rsx! {
        div {
            display: "flex",
            align_items: "center",
            gap: "8px",

            StatusIndicator {
                state,
                colors,
                size,
            }

            if show_label {
                span {
                    color: "{label_color}",
                    font_size: "11px",
                    text_transform: "uppercase",
                    letter_spacing: "0.12em",
                    transition: "color 300ms ease-in-out",

                    {label}
                }
            }
        }
    }
}
