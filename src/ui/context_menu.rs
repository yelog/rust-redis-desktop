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
            background: "#2d2d2d",
            border: "1px solid #3c3c3c",
            border_radius: "4px",
            box_shadow: "0 4px 12px rgba(0, 0, 0, 0.4)",
            padding: "4px 0",
            z_index: "1000",
            min_width: "150px",

            {children}
        }
    }
}

#[component]
pub fn ContextMenuItem(
    icon: Option<Element>,
    label: String,
    danger: bool,
    onclick: EventHandler<()>,
) -> Element {
    let mut hover = use_signal(|| false);

    rsx! {
        div {
            padding: "8px 12px",
            display: "flex",
            align_items: "center",
            gap: "8px",
            cursor: "pointer",
            color: if danger { "#f87171" } else { "#cccccc" },
            font_size: "13px",
            background: if hover() { "#094771" } else { "transparent" },

            onmouseenter: move |_| hover.set(true),
            onmouseleave: move |_| hover.set(false),

            onclick: move |_| onclick.call(()),

            if let Some(icon_el) = icon {
                {icon_el}
            }

            span { "{label}" }
        }
    }
}
