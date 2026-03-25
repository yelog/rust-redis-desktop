use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ContextMenuPosition {
    pub x: i32,
    pub y: i32,
}

#[component]
pub fn ContextMenu(x: i32, y: i32, on_close: EventHandler<()>, children: Element) -> Element {
    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            z_index: "999",
            onclick: move |_| on_close.call(()),
        }

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
