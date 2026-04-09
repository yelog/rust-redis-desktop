use super::data_loader;
use super::formatters::copy_value_to_clipboard;
use super::styles::{
    compact_icon_action_button_style, data_section_controls_style, data_section_count_style,
    data_section_toolbar_style, data_table_header_cell_style, data_table_header_row_style,
    destructive_action_button_style, overlay_modal_actions_style, overlay_modal_backdrop_style,
    overlay_modal_body_style, overlay_modal_keyframes, overlay_modal_surface_style,
    overlay_modal_title_style, secondary_action_button_style, status_banner_style,
};
use super::{BinaryFormat, LARGE_KEY_THRESHOLD};
use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::redis::KeyInfo;
use crate::serialization::SerializationFormat;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_TEXT,
    COLOR_TEXT_SECONDARY,
};
use crate::ui::icons::{IconCopy, IconTrash};
use crate::ui::pagination::LargeKeyWarning;
use crate::ui::ToastManager;
use dioxus::prelude::*;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn StreamPanel(
    connection_pool: ConnectionPool,
    display_key: String,
    on_refresh: EventHandler<()>,
    toast_manager: Signal<ToastManager>,
    key_info: Signal<Option<KeyInfo>>,
    string_value: Signal<String>,
    hash_value: Signal<HashMap<String, String>>,
    list_value: Signal<Vec<String>>,
    set_value: Signal<Vec<String>>,
    zset_value: Signal<Vec<(String, f64)>>,
    stream_value: Signal<Vec<(String, Vec<(String, String)>)>>,
    is_binary: Signal<bool>,
    binary_format: Signal<BinaryFormat>,
    serialization_data: Signal<Option<(SerializationFormat, Vec<u8>)>>,
    binary_bytes: Signal<Vec<u8>>,
    bitmap_info: Signal<Option<crate::redis::BitmapInfo>>,
    loading: Signal<bool>,
    hash_cursor: Signal<u64>,
    hash_total: Signal<usize>,
    hash_has_more: Signal<bool>,
    list_has_more: Signal<bool>,
    list_total: Signal<usize>,
    set_cursor: Signal<u64>,
    set_total: Signal<usize>,
    set_has_more: Signal<bool>,
    zset_cursor: Signal<u64>,
    zset_total: Signal<usize>,
    zset_has_more: Signal<bool>,
    mut stream_status_message: Signal<String>,
    mut stream_status_error: Signal<bool>,
    mut stream_search: Signal<String>,
    mut deleting_stream_entry: Signal<Option<String>>,
    mut deleting_stream_entry_exiting: Signal<bool>,
) -> Element {
    let i18n = use_i18n();
    let stream_val = stream_value();
    let stream_search_value = stream_search();
    let normalized_stream_search = stream_search_value.trim().to_lowercase();
    let filtered_stream_entries: Vec<(String, Vec<(String, String)>)> = stream_val
        .iter()
        .filter(|(entry_id, fields)| {
            if normalized_stream_search.is_empty() {
                true
            } else {
                entry_id.to_lowercase().contains(&normalized_stream_search)
                    || fields.iter().any(|(field_key, field_value)| {
                        field_key.to_lowercase().contains(&normalized_stream_search)
                            || field_value
                                .to_lowercase()
                                .contains(&normalized_stream_search)
                    })
            }
        })
        .cloned()
        .collect();

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100%",
            min_height: "0",

            div {
                style: "{data_section_toolbar_style()}",

                div {
                    style: "{data_section_controls_style()}",

                    input {
                        width: "260px",
                        max_width: "100%",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{stream_search}",
                        placeholder: i18n.read().t("Search ID or field"),
                        oninput: move |event| stream_search.set(event.value()),
                    }

                    div {
                        style: "{data_section_count_style()}",

                        if normalized_stream_search.is_empty() {
                            "Stream Entries ({stream_val.len()})"
                        } else {
                            "Stream Entries ({filtered_stream_entries.len()}/{stream_val.len()})"
                        }
                    }
                }

                button {
                    margin_left: "auto",
                    flex_shrink: "0",
                    style: "{secondary_action_button_style()}",
                    title: i18n.read().t("Copy"),
                    onclick: {
                        let stream = stream_val.clone();
                        move |_| {
                            let json = serde_json::to_string_pretty(&stream).unwrap_or_default();
                            match copy_value_to_clipboard(&json) {
                                Ok(_) => {
                                    toast_manager.write().success(&i18n.read().t("Copied"));
                                }
                                Err(error) => {
                                    toast_manager.write().error(&format!("{}{}", i18n.read().t("Copy failed: "), error));
                                }
                            }
                        }
                    },

                    IconCopy { size: Some(14) }
                    {i18n.read().t("Copy")}
                }
            }

            if !stream_status_message.read().is_empty() {
                div {
                    style: "{status_banner_style(stream_status_error())}",

                    "{stream_status_message}"
                }
            }

            if stream_val.len() > LARGE_KEY_THRESHOLD {
                LargeKeyWarning {
                    key_type: "Stream".to_string(),
                    size: stream_val.len(),
                    threshold: LARGE_KEY_THRESHOLD,
                }
            }

            div {
                flex: "1",
                min_height: "0",
                width: "100%",
                align_self: "stretch",
                overflow_x: "auto",
                overflow_y: "auto",
                border: "1px solid {COLOR_BORDER}",
                border_radius: "8px",
                background: COLOR_BG_SECONDARY,

                table {
                    width: "100%",
                    border_collapse: "collapse",

                    thead {
                        tr {
                            style: "{data_table_header_row_style()}",

                            th {
                                style: "{data_table_header_cell_style(Some(\"220px\"), \"left\")}",

                                "Entry ID"
                            }

                            th {
                                style: "{data_table_header_cell_style(None, \"left\")}",

                                "Fields"
                            }

                            th {
                                style: "{data_table_header_cell_style(Some(\"88px\"), \"center\")}",

                                "Action"
                            }
                        }
                    }

                    tbody {
                        if filtered_stream_entries.is_empty() {
                            tr {
                                td {
                                    colspan: "3",
                                    padding: "32px 12px",
                                    text_align: "center",
                                    color: COLOR_TEXT_SECONDARY,

                                    if normalized_stream_search.is_empty() {
                                        {i18n.read().t("No data")}
                                    } else {
                                        {i18n.read().t("No matching entries")}
                                    }
                                }
                            }
                        } else {
                            for (entry_id, fields) in filtered_stream_entries.iter() {
                                tr {
                                    key: "{entry_id}",
                                    border_bottom: "1px solid {COLOR_BORDER}",
                                    background: COLOR_BG_SECONDARY,

                                    td {
                                        padding: "10px 12px",
                                        color: COLOR_TEXT,
                                        font_size: "12px",
                                        font_family: "Consolas, monospace",
                                        word_break: "break-all",

                                        "{entry_id}"
                                    }

                                    td {
                                        padding: "10px 12px",

                                        div {
                                            display: "flex",
                                            flex_wrap: "wrap",
                                            gap: "8px",

                                            for (field_key, field_value) in fields.iter() {
                                                div {
                                                    padding: "4px 8px",
                                                    background: COLOR_BG_TERTIARY,
                                                    border: "1px solid {COLOR_BORDER}",
                                                    border_radius: "6px",
                                                    font_size: "11px",
                                                    line_height: "1.5",

                                                    span {
                                                        color: COLOR_ACCENT,
                                                        font_family: "Consolas, monospace",

                                                        "{field_key}:"
                                                    }
                                                    span {
                                                        color: COLOR_TEXT,
                                                        margin_left: "4px",
                                                        word_break: "break-all",

                                                        "{field_value}"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    td {
                                        padding: "10px 12px",
                                        text_align: "center",

                                        button {
                                            style: "{compact_icon_action_button_style(true, false)}",
                                            title: i18n.read().t("Delete"),
                                            onclick: {
                                                let entry_id = entry_id.clone();
                                                move |_| {
                                                    deleting_stream_entry.set(Some(entry_id.clone()));
                                                }
                                            },

                                            IconTrash { size: Some(15) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(entry_id) = deleting_stream_entry.read().clone() {
                div {
                    style: "{overlay_modal_backdrop_style(deleting_stream_entry_exiting())}",

                    style {
                        "{overlay_modal_keyframes()}"
                    }

                    div {
                        style: "{overlay_modal_surface_style(\"400px\", deleting_stream_entry_exiting())}",

                        h3 {
                            style: "{overlay_modal_title_style()}",

                            {i18n.read().t("Confirm delete")}
                        }

                        p {
                            style: "{overlay_modal_body_style()}",

                            {format!("{} \"{}\"?", i18n.read().t("Delete entry"), entry_id)}
                        }

                        div {
                            style: "{overlay_modal_actions_style()}",

                            button {
                                style: "{secondary_action_button_style()}",
                                onclick: {
                                    let deleting_stream_entry = deleting_stream_entry.clone();
                                    let mut deleting_stream_entry_exiting =
                                        deleting_stream_entry_exiting.clone();
                                    move |_| {
                                        deleting_stream_entry_exiting.set(true);
                                        let mut dse = deleting_stream_entry.clone();
                                        let mut dsee = deleting_stream_entry_exiting.clone();
                                        spawn(async move {
                                            tokio::time::sleep(std::time::Duration::from_millis(200))
                                                .await;
                                            dse.set(None);
                                            dsee.set(false);
                                        });
                                    }
                                },

                                {i18n.read().t("Cancel")}
                            }

                            button {
                                style: "{destructive_action_button_style(false)}",
                                onclick: {
                                    let pool = connection_pool.clone();
                                    let key = display_key.clone();
                                    let entry_id = entry_id.clone();
                                    let mut deleting_stream_entry = deleting_stream_entry.clone();
                                    let mut deleting_stream_entry_exiting =
                                        deleting_stream_entry_exiting.clone();
                                    move |_| {
                                        let pool = pool.clone();
                                        let key = key.clone();
                                        let entry_id = entry_id.clone();
                                        spawn(async move {
                                            match pool.stream_delete(&key, &entry_id).await {
                                                Ok(true) => {
                                                    stream_status_message.set(i18n.read().t("Deleted"));
                                                    stream_status_error.set(false);
                                                    deleting_stream_entry_exiting.set(true);
                                                    tokio::time::sleep(std::time::Duration::from_millis(
                                                        200,
                                                    ))
                                                    .await;
                                                    deleting_stream_entry.set(None);
                                                    deleting_stream_entry_exiting.set(false);
                                                    if let Err(error) = data_loader::load_key_data(
                                                        pool.clone(),
                                                        key.clone(),
                                                        key_info,
                                                        string_value,
                                                        hash_value,
                                                        list_value,
                                                        set_value,
                                                        zset_value,
                                                        stream_value,
                                                        is_binary,
                                                        binary_format,
                                                        serialization_data,
                                                        binary_bytes,
                                                        bitmap_info,
                                                        loading,
                                                        hash_cursor,
                                                        hash_total,
                                                        hash_has_more,
                                                        list_has_more,
                                                        list_total,
                                                        set_cursor,
                                                        set_total,
                                                        set_has_more,
                                                        zset_cursor,
                                                        zset_total,
                                                        zset_has_more,
                                                    )
                                                    .await
                                                    {
                                                        tracing::error!("{error}");
                                                    } else {
                                                        on_refresh.call(());
                                                    }
                                                }
                                                Ok(false) => {
                                                    stream_status_message
                                                        .set(i18n.read().t("Entry does not exist"));
                                                    stream_status_error.set(true);
                                                }
                                                Err(error) => {
                                                    stream_status_message
                                                        .set(format!("{}{}", i18n.read().t("Delete failed: "), error));
                                                    stream_status_error.set(true);
                                                }
                                            }
                                        });
                                    }
                                },

                                {i18n.read().t("Delete")}
                            }
                        }
                    }
                }
            }
        }
    }
}
