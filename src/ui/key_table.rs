use crate::redis::KeyType;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_ERROR,
    COLOR_PRIMARY, COLOR_TEXT, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use crate::ui::icons::{IconCopy, IconEdit, IconTrash};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct KeyTableRow {
    pub key: String,
    pub key_type: Option<KeyType>,
    pub ttl: Option<i64>,
    pub has_details: bool,
    pub is_selected: bool,
}

fn key_type_label(key_type: Option<&KeyType>) -> &'static str {
    match key_type {
        Some(KeyType::String) => "STR",
        Some(KeyType::Hash) => "HASH",
        Some(KeyType::List) => "LIST",
        Some(KeyType::Set) => "SET",
        Some(KeyType::ZSet) => "ZSET",
        Some(KeyType::Stream) => "STREAM",
        Some(KeyType::None) => "NONE",
        None => "--",
    }
}

fn key_type_tone(key_type: Option<&KeyType>) -> (&'static str, &'static str, &'static str) {
    match key_type {
        Some(KeyType::String) => (
            "rgba(255, 180, 166, 0.12)",
            COLOR_PRIMARY,
            "rgba(255, 180, 166, 0.24)",
        ),
        Some(KeyType::Hash) => (
            "rgba(0, 218, 243, 0.10)",
            COLOR_ACCENT,
            "rgba(0, 218, 243, 0.22)",
        ),
        Some(KeyType::List) => (
            "rgba(229, 226, 225, 0.08)",
            COLOR_TEXT_SECONDARY,
            "rgba(229, 226, 225, 0.18)",
        ),
        Some(KeyType::Set) => (
            "rgba(255, 180, 166, 0.10)",
            COLOR_PRIMARY,
            "rgba(255, 180, 166, 0.20)",
        ),
        Some(KeyType::ZSet) => (
            "rgba(0, 218, 243, 0.10)",
            COLOR_ACCENT,
            "rgba(0, 218, 243, 0.22)",
        ),
        Some(KeyType::Stream) => (
            "rgba(48, 209, 88, 0.10)",
            "#30d158",
            "rgba(48, 209, 88, 0.20)",
        ),
        Some(KeyType::None) | None => (
            "rgba(169, 138, 132, 0.10)",
            COLOR_TEXT_SUBTLE,
            "rgba(169, 138, 132, 0.18)",
        ),
    }
}

fn ttl_label(row: &KeyTableRow) -> String {
    if !row.has_details {
        "--".to_string()
    } else if let Some(ttl) = row.ttl {
        format!("{ttl}s")
    } else {
        "无期限".to_string()
    }
}

#[component]
pub fn KeyTable(
    rows: Vec<KeyTableRow>,
    selected_key: String,
    selection_mode: bool,
    on_select: EventHandler<String>,
    on_toggle_select: EventHandler<String>,
    on_copy_key: EventHandler<String>,
    on_request_delete: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            width: "100%",
            height: "100%",
            overflow: "auto",
            background: COLOR_BG_SECONDARY,
            border: "1px solid {COLOR_BORDER}",
            border_radius: "10px",

            table {
                width: "100%",
                border_collapse: "collapse",

                thead {
                    tr {
                        background: COLOR_BG_TERTIARY,
                        border_bottom: "1px solid {COLOR_BORDER}",

                        th {
                            width: "44px",
                            padding: "12px 16px",
                            text_align: "left",
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",
                            font_weight: "700",
                            text_transform: "uppercase",
                            letter_spacing: "0.14em",

                            if selection_mode { "选" } else { "" }
                        }

                        th {
                            padding: "12px 16px",
                            text_align: "left",
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",
                            font_weight: "700",
                            text_transform: "uppercase",
                            letter_spacing: "0.14em",

                            "Key Name"
                        }

                        th {
                            width: "120px",
                            padding: "12px 16px",
                            text_align: "left",
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",
                            font_weight: "700",
                            text_transform: "uppercase",
                            letter_spacing: "0.14em",

                            "Type"
                        }

                        th {
                            width: "120px",
                            padding: "12px 16px",
                            text_align: "left",
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",
                            font_weight: "700",
                            text_transform: "uppercase",
                            letter_spacing: "0.14em",

                            "TTL"
                        }

                        th {
                            width: "140px",
                            padding: "12px 16px",
                            text_align: "right",
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",
                            font_weight: "700",
                            text_transform: "uppercase",
                            letter_spacing: "0.14em",

                            "Actions"
                        }
                    }
                }

                tbody {
                    for row in rows {
                        {
                            let is_active = selected_key == row.key;
                            let (badge_bg, badge_fg, badge_border) = key_type_tone(row.key_type.as_ref());
                            let row_key = row.key.clone();
                            let checkbox_key = row.key.clone();
                            let edit_key = row.key.clone();
                            let copy_key = row.key.clone();
                            let delete_key = row.key.clone();

                            rsx! {
                                tr {
                                    key: "{row.key}",
                                    background: if is_active {
                                        "rgba(0, 218, 243, 0.06)"
                                    } else if row.is_selected {
                                        "rgba(48, 209, 88, 0.08)"
                                    } else {
                                        COLOR_BG
                                    },
                                    border_bottom: "1px solid {COLOR_BORDER}",
                                    cursor: "pointer",
                                    onclick: move |_| on_select.call(row_key.clone()),

                                    td {
                                        padding: "12px 16px",
                                        vertical_align: "middle",

                                        if selection_mode {
                                            input {
                                                r#type: "checkbox",
                                                checked: row.is_selected,
                                                onchange: move |_| on_toggle_select.call(checkbox_key.clone()),
                                            }
                                        }
                                    }

                                    td {
                                        padding: "12px 16px",
                                        vertical_align: "middle",

                                        div {
                                            display: "flex",
                                            align_items: "center",
                                            gap: "10px",

                                            div {
                                                width: "3px",
                                                height: "22px",
                                                border_radius: "999px",
                                                background: if is_active { COLOR_ACCENT } else { "transparent" },
                                            }

                                            span {
                                                color: if is_active { COLOR_ACCENT } else { COLOR_TEXT },
                                                font_size: "13px",
                                                font_family: "Consolas, 'Courier New', monospace",

                                                "{row.key}"
                                            }
                                        }
                                    }

                                    td {
                                        padding: "12px 16px",
                                        vertical_align: "middle",

                                        span {
                                            padding: "3px 8px",
                                            background: "{badge_bg}",
                                            color: "{badge_fg}",
                                            border: "1px solid {badge_border}",
                                            border_radius: "999px",
                                            font_size: "11px",
                                            font_weight: "700",

                                            "{key_type_label(row.key_type.as_ref())}"
                                        }
                                    }

                                    td {
                                        padding: "12px 16px",
                                        vertical_align: "middle",

                                        span {
                                            color: COLOR_TEXT_SECONDARY,
                                            font_size: "12px",
                                            font_family: "Consolas, 'Courier New', monospace",

                                            "{ttl_label(&row)}"
                                        }
                                    }

                                    td {
                                        padding: "12px 16px",
                                        vertical_align: "middle",

                                        div {
                                            display: "flex",
                                            justify_content: "flex_end",
                                            gap: "6px",

                                            button {
                                                width: "28px",
                                                height: "28px",
                                                background: "transparent",
                                                border: "1px solid transparent",
                                                border_radius: "6px",
                                                color: COLOR_TEXT_SECONDARY,
                                                cursor: "pointer",
                                                display: "flex",
                                                align_items: "center",
                                                justify_content: "center",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    on_select.call(edit_key.clone());
                                                },

                                                IconEdit { size: Some(14) }
                                            }

                                            button {
                                                width: "28px",
                                                height: "28px",
                                                background: "transparent",
                                                border: "1px solid transparent",
                                                border_radius: "6px",
                                                color: COLOR_TEXT_SECONDARY,
                                                cursor: "pointer",
                                                display: "flex",
                                                align_items: "center",
                                                justify_content: "center",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    on_copy_key.call(copy_key.clone());
                                                },

                                                IconCopy { size: Some(14) }
                                            }

                                            button {
                                                width: "28px",
                                                height: "28px",
                                                background: "transparent",
                                                border: "1px solid transparent",
                                                border_radius: "6px",
                                                color: COLOR_ERROR,
                                                cursor: "pointer",
                                                display: "flex",
                                                align_items: "center",
                                                justify_content: "center",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    on_request_delete.call(delete_key.clone());
                                                },

                                                IconTrash { size: Some(14) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
