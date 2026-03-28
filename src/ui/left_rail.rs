use crate::connection::ConnectionState;
use crate::theme::{
    ThemeColors, COLOR_ACCENT, COLOR_BG, COLOR_BG_LOWEST, COLOR_BORDER, COLOR_PRIMARY,
    COLOR_SURFACE_HIGH, COLOR_SURFACE_LOW, COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
    COLOR_TEXT_SUBTLE,
};
use crate::ui::context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
use crate::ui::icons::{
    IconAlert, IconDownload, IconEdit, IconMoreHorizontal, IconPlus, IconRefresh, IconSettings,
    IconTrash, IconUpload, IconX,
};
use crate::ui::status_indicator::StatusIndicatorWithLabel;
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

fn state_label(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connected => "已连接",
        ConnectionState::Connecting => "连接中",
        ConnectionState::Disconnected => "未连接",
        ConnectionState::Error => "连接异常",
    }
}

fn primary_connection_action_label(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Connecting => "连接中",
        ConnectionState::Connected => "重连",
        ConnectionState::Disconnected | ConnectionState::Error => "连接",
    }
}

fn state_dot_color(state: ConnectionState, colors: ThemeColors) -> &'static str {
    match state {
        ConnectionState::Connected => colors.state_connected,
        ConnectionState::Connecting => colors.state_connecting,
        ConnectionState::Disconnected => colors.state_disconnected,
        ConnectionState::Error => colors.state_error,
    }
}

#[component]
fn ConnectionDragOverlay(
    width: f64,
    drag_start_y: f64,
    item_height: f64,
    gap_height: f64,
    item_count: usize,
    drag_index: usize,
    drag_name: String,
    drag_dot_color: &'static str,
    on_drag_over_change: EventHandler<usize>,
    on_drop: EventHandler<usize>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut drag_current_y = use_signal(|| drag_start_y);
    let mut current_over_index = use_signal(|| drag_index);
    let preview_base_top = (drag_start_y - item_height / 2.0).max(0.0);
    let preview_offset = drag_current_y() - drag_start_y;
    let preview_width = (width - 24.0).max(0.0);

    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            z_index: "9998",
            cursor: "grabbing",
            user_select: "none",

            onmousemove: move |e| {
                let current_y = e.client_coordinates().y;
                drag_current_y.set(current_y);

                let delta_y = current_y - drag_start_y;
                let item_stride = item_height + gap_height;
                let rounded_shift = (delta_y / item_stride).round() as i32;
                let next_over_index = (drag_index as i32 + rounded_shift)
                    .clamp(0, item_count.saturating_sub(1) as i32) as usize;

                if current_over_index() != next_over_index {
                    current_over_index.set(next_over_index);
                    on_drag_over_change.call(next_over_index);
                }
            },

            onmouseup: move |_| {
                on_drop.call(current_over_index());
            },

            onmouseleave: move |_| {
                on_cancel.call(());
            },
        }

        div {
            position: "fixed",
            top: "{preview_base_top}px",
            left: "12px",
            width: "{preview_width}px",
            z_index: "9999",
            pointer_events: "none",
            transform: "translate3d(0, {preview_offset}px, 0)",
            will_change: "transform",

            div {
                padding: "12px",
                background: COLOR_BG,
                color: COLOR_TEXT,
                border: format!("1px solid {}", COLOR_ACCENT),
                border_radius: "8px",
                box_shadow: "0 6px 16px rgba(0, 0, 0, 0.28)",
                opacity: "0.96",
                display: "flex",
                align_items: "center",
                gap: "8px",

                div {
                    width: "8px",
                    height: "8px",
                    border_radius: "50%",
                    background: "{drag_dot_color}",
                    flex_shrink: "0",
                    box_shadow: "0 0 4px {drag_dot_color}",
                }

                span {
                    font_size: "13px",
                    font_weight: "600",
                    color: COLOR_TEXT,
                    flex: "1",

                    "{drag_name}"
                }

                svg {
                    width: "12",
                    height: "12",
                    view_box: "0 0 24 24",
                    fill: "currentColor",

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
}

#[component]
pub fn LeftRail(
    width: Signal<f64>,
    connections: Vec<(Uuid, String)>,
    connection_states: HashMap<Uuid, ConnectionState>,
    readonly_connections: HashMap<Uuid, bool>,
    selected_connection: Option<Uuid>,
    colors: ThemeColors,
    on_add_connection: EventHandler<()>,
    on_select_connection: EventHandler<Uuid>,
    on_edit_connection: EventHandler<Uuid>,
    on_delete_connection: EventHandler<Uuid>,
    on_reconnect_connection: EventHandler<Uuid>,
    on_close_connection: EventHandler<Uuid>,
    on_flush_connection: EventHandler<Uuid>,
    on_import_connection: EventHandler<Uuid>,
    on_export_connections: EventHandler<()>,
    on_import_connections: EventHandler<()>,
    on_open_settings: EventHandler<()>,
    on_reorder_connection: EventHandler<(usize, usize)>,
) -> Element {
    let mut context_menu = use_signal(|| None::<ContextMenuState<Uuid>>);
    let mut dragging_index = use_signal(|| None::<usize>);
    let mut drag_over_index = use_signal(|| None::<usize>);
    let mut drag_start_y = use_signal(|| 0.0);
    let item_height = 58.0;
    let gap_height = 6.0;
    let has_connections = !connections.is_empty();
    let selected_name = selected_connection.and_then(|id| {
        connections
            .iter()
            .find(|(conn_id, _)| *conn_id == id)
            .map(|(_, name)| name.clone())
    });
    let selected_state = selected_connection
        .and_then(|id| connection_states.get(&id).copied())
        .unwrap_or(ConnectionState::Disconnected);
    let is_readonly = selected_connection
        .and_then(|id| readonly_connections.get(&id).copied())
        .unwrap_or(false);

    rsx! {
        style { "
            .dot-tooltip-wrapper {{
                position: relative;
            }}
            .dot-tooltip {{
                position: absolute;
                left: calc(100% + 6px);
                top: 50%;
                transform: translateY(-50%);
                background: {COLOR_SURFACE_HIGH};
                border: 1px solid {COLOR_BORDER};
                border-radius: 4px;
                padding: 4px 8px;
                font-size: 11px;
                color: {COLOR_TEXT_SECONDARY};
                white-space: nowrap;
                pointer-events: none;
                opacity: 0;
                transition: opacity 0.15s;
                z-index: 100;
            }}
            .dot-tooltip-wrapper:hover .dot-tooltip {{
                opacity: 1;
            }}
        " }
        div {
            width: "{width()}px",
            height: "100%",
            background: COLOR_BG_LOWEST,
            border_right: "1px solid {COLOR_BORDER}",
            display: "flex",
            flex_direction: "column",
            overflow: "hidden",

            div {
                padding: "16px",
                display: "flex",
                flex_direction: "column",
                gap: "12px",
                border_bottom: "1px solid {COLOR_BORDER}",

                div {
                    padding: "12px",
                    background: COLOR_BG,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "8px",
                    display: "flex",
                    flex_direction: "column",
                    gap: "8px",

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "10px",

                        StatusIndicatorWithLabel {
                            state: selected_state,
                            colors,
                            show_label: Some(false),
                            size: Some(10.0),
                        }

                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: "3px",

                            span {
                                color: COLOR_TEXT,
                                font_size: "14px",
                                font_weight: "700",

                                if let Some(name) = selected_name.as_ref() {
                                    "{name}"
                                } else {
                                    "未选择连接"
                                }
                            }

                            div {
                                display: "flex",
                                align_items: "center",
                                gap: "8px",

                                span {
                                    color: COLOR_TEXT_SUBTLE,
                                    font_size: "11px",
                                    text_transform: "uppercase",
                                    letter_spacing: "0.12em",

                                    "{state_label(selected_state)}"
                                }

                                if is_readonly {
                                    span {
                                        padding: "2px 6px",
                                        background: "{colors.accent}",
                                        color: "{colors.primary_text}",
                                        font_size: "9px",
                                        font_weight: "600",
                                        border_radius: "4px",
                                        text_transform: "uppercase",

                                        "只读"
                                    }
                                }
                            }
                        }
                    }

                    if let Some(id) = selected_connection {
                        div {
                            display: "flex",
                            gap: "8px",

                            button {
                                flex: "1",
                                height: "32px",
                                padding: "0 10px",
                                background: COLOR_PRIMARY,
                                color: COLOR_TEXT_CONTRAST,
                                border: "1px solid {COLOR_PRIMARY}",
                                border_radius: "6px",
                                cursor: if selected_state == ConnectionState::Connecting { "default" } else { "pointer" },
                                opacity: if selected_state == ConnectionState::Connecting { "0.65" } else { "1" },
                                font_size: "12px",
                                font_weight: "600",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                gap: "6px",
                                disabled: selected_state == ConnectionState::Connecting,
                                onclick: move |_| on_reconnect_connection.call(id),

                                IconRefresh { size: Some(13) }
                                "{primary_connection_action_label(selected_state)}"
                            }

                            button {
                                width: "32px",
                                height: "32px",
                                background: COLOR_SURFACE_HIGH,
                                color: COLOR_TEXT_SECONDARY,
                                border: "1px solid {COLOR_BORDER}",
                                border_radius: "6px",
                                cursor: "pointer",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                onclick: {
                                    let mut context_menu = context_menu.clone();
                                    move |event| {
                                        event.stop_propagation();
                                        crate::ui::context_menu::close_all_context_menus();
                                        let x = event.client_coordinates().x as i32;
                                        let y = event.client_coordinates().y as i32;
                                        context_menu.set(Some(ContextMenuState::new(id, x, y)));
                                    }
                                },

                                IconMoreHorizontal { size: Some(14), color: None }
                            }
                        }
                    }
                }

                button {
                    width: "100%",
                    height: "36px",
                    padding: "0 12px",
                    background: COLOR_PRIMARY,
                    color: COLOR_TEXT_CONTRAST,
                    border: "none",
                    border_radius: "8px",
                    cursor: "pointer",
                    font_size: "13px",
                    font_weight: "700",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    gap: "8px",
                    onclick: move |_| on_add_connection.call(()),

                    IconPlus { size: Some(14) }
                    "新建连接"
                }
            }

            div {
                padding: "12px 16px 8px",
                display: "flex",
                align_items: "center",
                justify_content: "space_between",
                gap: "8px",

                span {
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "11px",
                    text_transform: "uppercase",
                    letter_spacing: "0.16em",

                    "Connections"
                }

                div {
                    display: "flex",
                    gap: "6px",

                    button {
                        width: "28px",
                        height: "28px",
                        background: COLOR_SURFACE_LOW,
                        color: COLOR_TEXT_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        cursor: "pointer",
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        title: "导出连接",
                        onclick: move |_| on_export_connections.call(()),

                        IconDownload { size: Some(12) }
                    }

                    button {
                        width: "28px",
                        height: "28px",
                        background: COLOR_SURFACE_LOW,
                        color: COLOR_TEXT_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        cursor: "pointer",
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        title: "导入连接",
                        onclick: move |_| on_import_connections.call(()),

                        IconUpload { size: Some(12) }
                    }
                }
            }

            div {
                flex: "1",
                overflow_y: "auto",
                padding: "0 12px 12px",
                display: "flex",
                flex_direction: "column",
                gap: "6px",

                for (index, (id, name)) in connections.iter().enumerate() {
                    {
                        let id = *id;
                        let name = name.clone();
                        let state = connection_states
                            .get(&id)
                            .copied()
                            .unwrap_or(ConnectionState::Disconnected);
                        let is_selected = selected_connection == Some(id);
                        let is_dragging = dragging_index() == Some(index);
                        let is_drag_over = drag_over_index() == Some(index);
                        let dot_color = state_dot_color(state, colors);
                        let mut context_menu_clone = context_menu.clone();

                        rsx! {
                            div {
                                key: "{id}",
                                padding: "12px",
                                background: if is_dragging {
                                    COLOR_SURFACE_LOW
                                } else if is_drag_over {
                                    COLOR_SURFACE_HIGH
                                } else if is_selected {
                                    COLOR_BG
                                } else {
                                    COLOR_SURFACE_LOW
                                },
                                color: if is_selected { COLOR_TEXT } else { COLOR_TEXT_SECONDARY },
                                border: if is_selected {
                                    format!("1px solid {}", COLOR_BORDER)
                                } else {
                                    "1px solid transparent".to_string()
                                },
                                outline: if is_drag_over && !is_dragging {
                                    format!("2px solid {}", COLOR_ACCENT)
                                } else {
                                    "none".to_string()
                                },
                                border_radius: "8px",
                                cursor: if is_dragging { "grabbing" } else { "pointer" },
                                text_align: "left",
                                display: "flex",
                                align_items: "center",
                                opacity: if is_dragging { "0.3" } else { "1" },
                                transition: "background 120ms ease, opacity 120ms ease, outline-color 120ms ease",
                                onclick: move |_| on_select_connection.call(id),
                                oncontextmenu: move |e| {
                                    e.prevent_default();
                                    crate::ui::context_menu::close_all_context_menus();
                                    let x = e.client_coordinates().x as i32;
                                    let y = e.client_coordinates().y as i32;
                                    context_menu_clone.set(Some(ContextMenuState::new(id, x, y)));
                                },

                                div {
                                    display: "flex",
                                    align_items: "center",
                                    gap: "8px",
                                    width: "100%",

                                    div {
                                        class: "dot-tooltip-wrapper",
                                        position: "relative",
                                        display: "flex",
                                        align_items: "center",
                                        justify_content: "center",
                                        width: "16px",
                                        height: "16px",
                                        flex_shrink: "0",
                                        cursor: "default",

                                        div {
                                            width: "8px",
                                            height: "8px",
                                            border_radius: "50%",
                                            background: "{dot_color}",
                                            flex_shrink: "0",
                                            box_shadow: "0 0 4px {dot_color}",
                                            transition: "box-shadow 0.2s ease",
                                        }

                                        div {
                                            class: "dot-tooltip",
                                            "{state_label(state)}"
                                        }
                                    }

                                    span {
                                        font_size: "13px",
                                        font_weight: if is_selected { "700" } else { "500" },
                                        color: if is_selected { colors.accent } else { colors.text },
                                        flex: "1",

                                        "{name}"
                                    }

                                    div {
                                        padding: "4px",
                                        cursor: "grab",
                                        opacity: "0.4",
                                        display: "flex",
                                        align_items: "center",
                                        margin_left: "auto",
                                        border_radius: "4px",
                                        transition: "opacity 0.15s ease, background 0.15s ease",

                                        onmouseenter: move |_| {},
                                        onmouseleave: move |_| {},

                                        onmousedown: move |e| {
                                            e.prevent_default();
                                            e.stop_propagation();
                                            let y = e.client_coordinates().y;
                                            dragging_index.set(Some(index));
                                            drag_over_index.set(Some(index));
                                            drag_start_y.set(y);
                                        },

                                        svg {
                                            width: "12",
                                            height: "12",
                                            view_box: "0 0 24 24",
                                            fill: "currentColor",

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
                        }
                    }
                }

                if let Some(drag_idx) = dragging_index() {
                    if let Some((drag_id, drag_name)) = connections.get(drag_idx) {
                        {
                            let drag_name = drag_name.clone();
                            let drag_dot_color = state_dot_color(
                                connection_states
                                    .get(drag_id)
                                    .copied()
                                    .unwrap_or(ConnectionState::Disconnected),
                                colors,
                            );

                            rsx! {
                                ConnectionDragOverlay {
                                    width: width(),
                                    drag_start_y: drag_start_y(),
                                    item_height: item_height,
                                    gap_height: gap_height,
                                    item_count: connections.len(),
                                    drag_index: drag_idx,
                                    drag_name: drag_name,
                                    drag_dot_color: drag_dot_color,
                                    on_drag_over_change: move |to| {
                                        drag_over_index.set(Some(to));
                                    },
                                    on_drop: move |to| {
                                        if let Some(from) = dragging_index() {
                                            if from != to {
                                                on_reorder_connection.call((from, to));
                                            }
                                        }
                                        dragging_index.set(None);
                                        drag_over_index.set(None);
                                    },
                                    on_cancel: move |_| {
                                        dragging_index.set(None);
                                        drag_over_index.set(None);
                                    },
                                }
                            }
                        }
                    }
                }

                if selected_connection.is_none() && !has_connections {
                    div {
                        padding: "12px",
                        color: COLOR_TEXT_SUBTLE,
                        font_size: "12px",
                        background: COLOR_SURFACE_LOW,
                        border_radius: "8px",

                        "还没有连接，先创建一个 Redis 连接。"
                    }
                }
            }

            div {
                padding: "12px",
                border_top: "1px solid {COLOR_BORDER}",
                display: "flex",
                flex_direction: "column",
                gap: "8px",

                button {
                    padding: "10px 12px",
                    background: COLOR_SURFACE_LOW,
                    color: COLOR_TEXT_SECONDARY,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "8px",
                    cursor: "pointer",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    font_size: "13px",
                    onclick: move |_| on_open_settings.call(()),

                    IconSettings { size: Some(14) }
                    "设置"
                }
            }

            if let Some(menu) = context_menu() {
                {
                    let ctx_id = menu.data;
                    let menu_id = menu.id;
                    let x = menu.x;
                    let y = menu.y;
                    let state = connection_states
                        .get(&ctx_id)
                        .copied()
                        .unwrap_or(ConnectionState::Disconnected);
                    let is_connected = matches!(state, ConnectionState::Connected);
                    let is_connecting = matches!(state, ConnectionState::Connecting);
                    let mut context_menu_for_close = context_menu.clone();

                    rsx! {
                        ContextMenu {
                            key: "{menu_id}",
                            menu_id: menu_id,
                            x: x,
                            y: y,
                            on_close: move |closing_menu_id| {
                                if context_menu_for_close()
                                    .as_ref()
                                    .map(|menu| menu.id)
                                    == Some(closing_menu_id)
                                {
                                    context_menu_for_close.set(None);
                                }
                            },

                            ContextMenuItem {
                                icon: Some(rsx! { IconEdit { size: Some(14) } }),
                                label: "编辑".to_string(),
                                danger: false,
                                disabled: false,
                                onclick: {
                                    let ctx_id = ctx_id;
                                    move |_| {
                                        context_menu.set(None);
                                        on_edit_connection.call(ctx_id);
                                    }
                                },
                            }

                            ContextMenuItem {
                                icon: Some(rsx! { IconRefresh { size: Some(14) } }),
                                label: "重连".to_string(),
                                danger: false,
                                disabled: is_connecting,
                                onclick: {
                                    let ctx_id = ctx_id;
                                    move |_| {
                                        context_menu.set(None);
                                        on_reconnect_connection.call(ctx_id);
                                    }
                                },
                            }

                            ContextMenuItem {
                                icon: Some(rsx! { IconX { size: Some(14) } }),
                                label: "断开".to_string(),
                                danger: false,
                                disabled: !is_connected,
                                onclick: {
                                    let ctx_id = ctx_id;
                                    move |_| {
                                        context_menu.set(None);
                                        on_close_connection.call(ctx_id);
                                    }
                                },
                            }

                            ContextMenuItem {
                                icon: Some(rsx! { IconUpload { size: Some(14) } }),
                                label: "导入数据".to_string(),
                                danger: false,
                                disabled: false,
                                onclick: {
                                    let ctx_id = ctx_id;
                                    move |_| {
                                        context_menu.set(None);
                                        on_import_connection.call(ctx_id);
                                    }
                                },
                            }

                            ContextMenuItem {
                                icon: Some(rsx! { IconTrash { size: Some(14) } }),
                                label: "删除".to_string(),
                                danger: true,
                                disabled: is_connecting || is_connected,
                                onclick: {
                                    let ctx_id = ctx_id;
                                    move |_| {
                                        context_menu.set(None);
                                        on_delete_connection.call(ctx_id);
                                    }
                                },
                            }

                            ContextMenuItem {
                                icon: Some(rsx! { IconAlert { size: Some(14) } }),
                                label: "清空数据".to_string(),
                                danger: true,
                                disabled: !is_connected,
                                onclick: {
                                    let ctx_id = ctx_id;
                                    move |_| {
                                        context_menu.set(None);
                                        on_flush_connection.call(ctx_id);
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
