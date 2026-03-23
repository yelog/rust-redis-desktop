use crate::theme::ThemeColors;
use crate::ui::animation_utils::{prefers_reduced_motion, TriggerPosition};
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
    colors: ThemeColors,
    width: Option<String>,
    max_height: Option<String>,
    trigger_selector: Option<String>,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    let width_val = width.unwrap_or_else(|| "450px".to_string());
    let max_height_val = max_height.unwrap_or_else(|| "90vh".to_string());

    let mut visibility = use_signal(VisibilityState::default);
    let mut trigger_position = use_signal(|| None::<TriggerPosition>);
    let reduced_motion = prefers_reduced_motion();
    let backdrop_color = colors.overlay_backdrop;

    use_effect(move || {
        if *visibility.read() == VisibilityState::Hidden {
            visibility.set(VisibilityState::Visible);

            if let Some(selector) = &trigger_selector {
                let selector = selector.clone();
                spawn(async move {
                    let js = format!(
                        r#"
                        (function() {{
                            const el = document.querySelector('{}');
                            if (!el) {{ dioxus.send(''); return; }}
                            const rect = el.getBoundingClientRect();
                            dioxus.send(JSON.stringify({{
                                x: (rect.left + rect.width / 2) / window.innerWidth,
                                y: (rect.top + rect.height / 2) / window.innerHeight
                            }}));
                        }})()
                        "#,
                        selector
                    );

                    let mut eval = dioxus::document::eval(&js);
                    if let Ok(result) = eval.recv::<String>().await {
                        if !result.is_empty() {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                                if let (Some(x), Some(y)) = (parsed["x"].as_f64(), parsed["y"].as_f64()) {
                                    trigger_position.set(Some(TriggerPosition {
                                        x: x as f32,
                                        y: y as f32,
                                    }));
                                }
                            }
                        }
                    }
                });
            }
        }
    });

    let mut escape_received = use_signal(|| false);
    let on_close_for_escape = on_close.clone();
    let mut visibility_for_escape = visibility.clone();

    use_future(move || {
        let mut escape_received = escape_received.clone();
        async move {
            let mut eval = dioxus::document::eval(
                r#"
                document.addEventListener('keydown', function(e) {
                    if (e.key === 'Escape') {
                        dioxus.send('escape');
                    }
                });
                await new Promise(() => {});
                "#,
            );
            while let Ok(msg) = eval.recv::<String>().await {
                if msg == "escape" {
                    escape_received.set(true);
                }
            }
        }
    });

    let close_dialog = {
        let on_close = on_close.clone();
        let mut visibility = visibility.clone();
        move || {
            let state = *visibility.read();
            if state == VisibilityState::Visible {
                visibility.set(VisibilityState::Exiting);
                
                let mut vis = visibility.clone();
                spawn(async move {
                    tokio::time::sleep(Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                    vis.set(VisibilityState::Hidden);
                });
                
                on_close.call(());
            }
        }
    };

    use_effect(move || {
        if escape_received() && *visibility_for_escape.read() == VisibilityState::Visible {
            close_dialog();
        }
        if escape_received() {
            escape_received.set(false);
        }
    });

    let state = *visibility.read();

    if state == VisibilityState::Hidden {
        return rsx! {};
    }

    let is_exiting = state == VisibilityState::Exiting;
    let animation_name = if is_exiting { "modalFadeOut" } else { "modalFadeIn" };

    let (origin_x, origin_y) = match *trigger_position.read() {
        Some(p) => p.to_transform_origin(),
        None => ("50%".to_string(), "50%".to_string()),
    };

    let on_backdrop_click = {
        let close_dialog = close_dialog.clone();
        move |_| close_dialog()
    };

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
            onclick: on_backdrop_click,

            div {
                width: "{width_val}",
                max_height: "{max_height_val}",
                padding: "24px",
                background: "{colors.background}",
                border_radius: "8px",
                box_shadow: "0 4px 24px rgba(0, 0, 0, 0.5)",
                overflow_y: "auto",
                animation: "{animation_name} 0.2s ease-out forwards",
                transform_origin: "{origin_x} {origin_y}",
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