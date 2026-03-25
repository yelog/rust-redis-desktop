use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static CONTEXT_MENU_OPEN: AtomicBool = AtomicBool::new(false);

pub fn context_menu_is_open() -> bool {
    CONTEXT_MENU_OPEN.load(Ordering::SeqCst)
}

pub fn set_context_menu_open(open: bool) {
    CONTEXT_MENU_OPEN.store(open, Ordering::SeqCst);
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
    let mut mounted = use_signal(|| false);
    
    use_effect(move || {
        if !mounted() {
            mounted.set(true);
            set_context_menu_open(true);
            let on_close = on_close.clone();
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
                    on_close.call(());
                    break;
                }
            });
        }
    });

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