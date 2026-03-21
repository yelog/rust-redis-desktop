use dioxus::prelude::*;

#[component]
pub fn ResizableDivider(width: Signal<f64>, min_width: f64, max_width: f64) -> Element {
    let mut is_dragging = use_signal(|| false);
    let mut drag_start_x = use_signal(|| 0.0);
    let mut drag_start_width = use_signal(|| 0.0);

    rsx! {
        div {
            width: "5px",
            height: "100%",
            background: if is_dragging() { "#4ec9b0" } else { "#3c3c3c" },
            cursor: "col-resize",
            flex_shrink: "0",
            transition: "background 0.2s",

            onmousedown: move |e: Event<MouseData>| {
                e.prevent_default();
                is_dragging.set(true);
                drag_start_x.set(e.data().client_coordinates().x);
                drag_start_width.set(width());
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
                cursor: "col-resize",

                onmousemove: move |e: Event<MouseData>| {
                    let current_x = e.data().client_coordinates().x;
                    let delta = current_x - drag_start_x();
                    let new_width = drag_start_width() + delta;
                    let clamped = new_width.clamp(min_width, max_width);
                    width.set(clamped);
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
