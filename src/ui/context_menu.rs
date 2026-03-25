use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static CONTEXT_MENU_OPEN: AtomicBool = AtomicBool::new(false);

pub fn context_menu_is_open() -> bool {
    CONTEXT_MENU_OPEN.load(Ordering::SeqCst)
}

pub fn set_context_menu_open(open: bool) {
    CONTEXT_MENU_OPEN.store(open, Ordering::SeqCst);
}

const ENTER_DURATION_MS: u64 = 150;
const EXIT_DURATION_MS: u64 = 100;

#[derive(Clone, Copy, PartialEq, Default)]
enum VisibilityState {
    #[default]
    Hidden,
    Visible,
    Exiting,
}

#[derive(Clone, PartialEq)]
pub struct ContextMenuPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, PartialEq)]
pub struct ContextMenuState<T> {
    pub data: T,
    pub x: i32,
    pub y: i32,
    pub id: u64,
}

impl<T> ContextMenuState<T> {
    pub fn new(data: T, x: i32, y: i32) -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Self { data, x, y, id }
    }
}

#[component]
pub fn ContextMenu(x: i32, y: i32, on_close: EventHandler<()>, children: Element) -> Element {
    let mut visibility = use_signal(VisibilityState::default);
    let mut mounted = use_signal(|| false);

    {
        let current = *visibility.read();
        if current == VisibilityState::Hidden && !mounted() {
            visibility.set(VisibilityState::Visible);
            mounted.set(true);
            set_context_menu_open(true);
        }
    }

    use_effect(move || {
        if *visibility.read() == VisibilityState::Visible {
            let on_close = on_close.clone();
            let mut visibility = visibility.clone();
            spawn(async move {
                let mut eval = dioxus::document::eval(
                    r#"
                    let handler = function(e) {
                        dioxus.send(e.type);
                    };
                    document.addEventListener('mousedown', handler, true);
                    document.addEventListener('click', handler, true);
                    await new Promise(() => {});
                    "#,
                );
                while let Ok(_) = eval.recv::<String>().await {
                    visibility.set(VisibilityState::Exiting);
                    let on_close = on_close.clone();
                    let mut vis = visibility.clone();
                    spawn(async move {
                        tokio::time::sleep(Duration::from_millis(EXIT_DURATION_MS)).await;
                        vis.set(VisibilityState::Hidden);
                        set_context_menu_open(false);
                        on_close.call(());
                    });
                    break;
                }
            });
        }
    });

    let state = *visibility.read();
    if state == VisibilityState::Hidden {
        return rsx! {};
    }

    let is_exiting = state == VisibilityState::Exiting;
    let (animation_name, duration_ms) = if is_exiting {
        ("contextMenuFadeOut", EXIT_DURATION_MS)
    } else {
        ("contextMenuFadeIn", ENTER_DURATION_MS)
    };

    rsx! {
        div {
            position: "fixed",
            left: "{x}px",
            top: "{y}px",
            background: crate::theme::COLOR_BG,
            border: "1px solid {crate::theme::COLOR_BORDER}",
            border_radius: "8px",
            box_shadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
            padding: "6px",
            z_index: "1000",
            min_width: "160px",
            display: "flex",
            flex_direction: "column",
            gap: "2px",
            transform_origin: "top left",
            animation: "{animation_name} {duration_ms}ms ease-out forwards",

            style {
                r#"
                @keyframes contextMenuFadeIn {{
                    from {{
                        opacity: 0;
                        transform: scale(0.9);
                    }}
                    to {{
                        opacity: 1;
                        transform: scale(1);
                    }}
                }}
                @keyframes contextMenuFadeOut {{
                    from {{
                        opacity: 1;
                        transform: scale(1);
                    }}
                    to {{
                        opacity: 0;
                        transform: scale(0.95);
                    }}
                }}
                "#
            }

            {children}
        }
    }
}

#[component]
pub fn ContextMenuItem(
    icon: Option<Element>,
    label: String,
    danger: bool,
    disabled: bool,
    onclick: EventHandler<()>,
) -> Element {
    let mut hover = use_signal(|| false);

    rsx! {
        button {
            padding: "8px 12px",
            display: "flex",
            align_items: "center",
            gap: "8px",
            cursor: if disabled { "default" } else { "pointer" },
            color: if disabled {
                crate::theme::COLOR_TEXT_SUBTLE
            } else if danger {
                crate::theme::COLOR_ERROR
            } else {
                crate::theme::COLOR_TEXT
            },
            font_size: "13px",
            background: if hover() && !disabled {
                if danger { "rgba(239, 68, 68, 0.1)" } else { crate::theme::COLOR_BG_TERTIARY }
            } else {
                "transparent"
            },
            border: "none",
            border_radius: "6px",
            width: "100%",
            text_align: "left",
            disabled: disabled,

            onmouseenter: move |_| hover.set(true),
            onmouseleave: move |_| hover.set(false),

            onclick: move |_| {
                if !disabled {
                    onclick.call(());
                }
            },

            if let Some(icon_el) = icon {
                {icon_el}
            }

            span { "{label}" }
        }
    }
}