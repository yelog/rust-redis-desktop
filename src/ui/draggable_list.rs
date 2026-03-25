use crate::theme::COLOR_ACCENT;
use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub struct DragItem {
    pub id: Uuid,
    pub name: String,
}

#[component]
pub fn DraggableList(
    items: Vec<DragItem>,
    selected_id: Option<Uuid>,
    on_reorder: EventHandler<(usize, usize)>,
    on_select: EventHandler<Uuid>,
    on_context_menu: EventHandler<(Uuid, i32, i32)>,
    children: Element,
) -> Element {
    let mut dragging_index = use_signal(|| None::<usize>);
    let mut drag_over_index = use_signal(|| None::<usize>);
    let mut drag_start_y = use_signal(|| 0.0);
    let mut drag_current_y = use_signal(|| 0.0);

    rsx! {
        div {
            position: "relative",

            {children}

            if let Some(drag_idx) = dragging_index() {
                div {
                    position: "fixed",
                    top: "0",
                    left: "0",
                    right: "0",
                    bottom: "0",
                    z_index: "9999",
                    cursor: "grabbing",

                    onmousemove: move |e: Event<MouseData>| {
                        let current_y = e.data().client_coordinates().y;
                        drag_current_y.set(current_y);

                        let delta_y = current_y - drag_start_y();
                        let item_height = 58.0;
                        let move_count = (delta_y.abs() / item_height).floor() as i32;

                        if move_count > 0 {
                            let direction: i32 = if delta_y > 0.0 { 1 } else { -1 };
                            let new_over_index = (drag_idx as i32 + direction * move_count) as usize;
                            let clamped = new_over_index.min(items.len().saturating_sub(1));
                            if drag_over_index() != Some(clamped) {
                                drag_over_index.set(Some(clamped));
                            }
                        } else {
                            if drag_over_index() != Some(drag_idx) {
                                drag_over_index.set(Some(drag_idx));
                            }
                        }
                    },

                    onmouseup: move |_| {
                        if let (Some(from), Some(to)) = (dragging_index(), drag_over_index()) {
                            if from != to {
                                on_reorder.call((from, to));
                            }
                        }
                        dragging_index.set(None);
                        drag_over_index.set(None);
                    },

                    onmouseleave: move |_| {
                        dragging_index.set(None);
                        drag_over_index.set(None);
                    },
                }
            }
        }
    }
}

#[component]
pub fn DragHandle(index: usize, on_drag_start: EventHandler<(usize, f64)>) -> Element {
    rsx! {
        div {
            padding: "4px",
            cursor: "grab",
            opacity: "0.5",
            margin_right: "4px",
            display: "flex",
            align_items: "center",

            onmousedown: move |e: Event<MouseData>| {
                e.prevent_default();
                e.stop_propagation();
                let y = e.data().client_coordinates().y;
                on_drag_start.call((index, y));
            },

            svg {
                width: "12",
                height: "12",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",

                circle {
                    cx: "8",
                    cy: "6",
                    r: "1.5",
                }
                circle {
                    cx: "16",
                    cy: "6",
                    r: "1.5",
                }
                circle {
                    cx: "8",
                    cy: "12",
                    r: "1.5",
                }
                circle {
                    cx: "16",
                    cy: "12",
                    r: "1.5",
                }
                circle {
                    cx: "8",
                    cy: "18",
                    r: "1.5",
                }
                circle {
                    cx: "16",
                    cy: "18",
                    r: "1.5",
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ConnectionDragItemProps {
    pub id: Uuid,
    pub name: String,
    pub state_color: String,
    pub state_label: String,
    pub is_selected: bool,
    pub is_dragging: bool,
    pub is_drag_over: bool,
    pub on_select: EventHandler<Uuid>,
    pub on_context_menu: EventHandler<(Uuid, i32, i32)>,
    pub on_drag_start: EventHandler<(usize, f64)>,
    pub index: usize,
    pub accent_color: String,
    pub text_color: String,
}

#[component]
pub fn ConnectionDragItem(props: ConnectionDragItemProps) -> Element {
    let id = props.id;
    let state_color = props.state_color.clone();
    let state_label = props.state_label.clone();
    let is_selected = props.is_selected;
    let is_dragging = props.is_dragging;
    let is_drag_over = props.is_drag_over;
    let name = props.name.clone();
    let accent_color = props.accent_color.clone();
    let text_color = props.text_color.clone();

    rsx! {
        div {
            padding: "12px",
            background: if is_dragging {
                "var(--theme-surface-high, #2d2d2d)"
            } else if is_selected {
                "var(--theme-surface-base, #1e1e1e)"
            } else {
                "var(--theme-surface-low, #252526)"
            },
            color: if is_selected {
                "var(--theme-text, #e0e0e0)"
            } else {
                "var(--theme-text-secondary, #9d9d9d)"
            },
            border: if is_drag_over {
                format!("2px solid {}", COLOR_ACCENT)
            } else if is_selected {
                "1px solid var(--theme-border, #3c3c3c)".to_string()
            } else {
                "1px solid transparent".to_string()
            },
            border_radius: "8px",
            cursor: "pointer",
            text_align: "left",
            display: "flex",
            flex_direction: "column",
            gap: "6px",
            opacity: if is_dragging { "0.5" } else { "1" },
            position: "relative",

            onclick: move |_| props.on_select.call(id),
            oncontextmenu: move |e| {
                e.prevent_default();
                let x = e.data().client_coordinates().x as i32;
                let y = e.data().client_coordinates().y as i32;
                props.on_context_menu.call((id, x, y));
            },

            div {
                display: "flex",
                align_items: "center",
                gap: "8px",

                div {
                    width: "8px",
                    height: "8px",
                    border_radius: "50%",
                    background: "{state_color}",
                    flex_shrink: "0",
                }

                span {
                    font_size: "13px",
                    font_weight: if is_selected { "700" } else { "500" },
                    color: if is_selected {
                        accent_color.clone()
                    } else {
                        text_color.clone()
                    },
                    flex: "1",

                    "{name}"
                }

                DragHandle {
                    index: props.index,
                    on_drag_start: props.on_drag_start.clone(),
                }
            }

            div {
                display: "flex",
                align_items: "center",
                justify_content: "space_between",

                span {
                    color: "var(--theme-text-subtle, #6b6b6b)",
                    font_size: "11px",

                    "{state_label}"
                }
            }
        }
    }
}
