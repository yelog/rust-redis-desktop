use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum DividerDirection {
    Vertical,
    Horizontal,
}

#[component]
pub fn ResizableDivider(
    size: Signal<f64>,
    min_size: f64,
    max_size: f64,
    direction: Option<DividerDirection>,
) -> Element {
    let direction = direction.unwrap_or(DividerDirection::Vertical);
    let is_horizontal = direction == DividerDirection::Horizontal;
    let mut is_dragging = use_signal(|| false);
    let mut drag_start_pos = use_signal(|| 0.0);
    let mut drag_start_size = use_signal(|| 0.0);

    rsx! {
        div {
            width: if is_horizontal { "100%" } else { "6px" },
            height: if is_horizontal { "6px" } else { "100%" },
            background: if is_dragging() {
                "var(--theme-accent, #00daf3)"
            } else {
                "var(--theme-outline-variant, #3c3c3c)"
            },
            cursor: if is_horizontal { "row-resize" } else { "col-resize" },
            flex_shrink: "0",
            transition: "background 0.2s",
            opacity: if is_dragging() { "1" } else { "0.7" },

            onmousedown: move |e: Event<MouseData>| {
                e.prevent_default();
                is_dragging.set(true);
                drag_start_pos.set(if is_horizontal {
                    e.data().client_coordinates().y
                } else {
                    e.data().client_coordinates().x
                });
                drag_start_size.set(size());
            },
        }

        if is_dragging() {
            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                z_index: "9999",
                cursor: if is_horizontal { "row-resize" } else { "col-resize" },

                onmousemove: move |e: Event<MouseData>| {
                    let current_pos = if is_horizontal {
                        e.data().client_coordinates().y
                    } else {
                        e.data().client_coordinates().x
                    };
                    let delta = current_pos - drag_start_pos();
                    let new_size = drag_start_size() + delta;
                    let clamped = new_size.clamp(min_size, max_size);
                    size.set(clamped);
                },

                onmouseup: move |_| {
                    is_dragging.set(false);
                },

                onmouseleave: move |_| {
                    is_dragging.set(false);
                },
            }
        }
    }
}
