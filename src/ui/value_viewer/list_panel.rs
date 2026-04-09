use super::data_loader;
use super::formatters::copy_value_to_clipboard;
use super::styles::{
    compact_icon_action_button_style, data_section_controls_style, data_section_count_style,
    data_section_toolbar_style, data_table_header_cell_style, data_table_header_row_style,
    primary_action_button_style, secondary_action_button_style, status_banner_style,
};
use super::{BinaryFormat, LARGE_KEY_THRESHOLD, ROW_EDIT_BG};
use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::redis::KeyInfo;
use crate::serialization::SerializationFormat;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_TEXT,
    COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use crate::ui::icons::{IconCopy, IconEdit, IconTrash};
use crate::ui::pagination::LargeKeyWarning;
use crate::ui::ToastManager;
use dioxus::prelude::*;
use serde_json;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn ListPanel(
    connection_pool: ConnectionPool,
    display_key: String,
    on_refresh: EventHandler<()>,
    mut toast_manager: Signal<ToastManager>,
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
    list_loading_more: Signal<bool>,
    mut list_status_message: Signal<String>,
    mut list_status_error: Signal<bool>,
    mut new_list_value: Signal<String>,
    mut list_action: Signal<Option<String>>,
    mut editing_list_index: Signal<Option<usize>>,
    mut editing_list_value: Signal<String>,
) -> Element {
    let i18n = use_i18n();
    let list_val = list_value();

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
                        width: "300px",
                        max_width: "100%",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{new_list_value}",
                        placeholder: i18n.read().t("Enter new element value"),
                        oninput: move |event| new_list_value.set(event.value()),
                    }

                    button {
                        style: "{primary_action_button_style(list_action().is_some())}",
                        disabled: list_action().is_some(),
                        onclick: {
                            let pool = connection_pool.clone();
                            let key = display_key.clone();
                            move |_| {
                                let pool = pool.clone();
                                let key = key.clone();
                                let value = new_list_value();
                                spawn(async move {
                                    if value.trim().is_empty() {
                                        list_status_message.set(i18n.read().t("Value cannot be empty"));
                                        list_status_error.set(true);
                                        return;
                                    }

                                    list_action.set(Some("push".to_string()));
                                    list_status_message.set(String::new());
                                    list_status_error.set(false);

                                    match pool.list_push(&key, &value, true).await {
                                        Ok(_) => {
                                            new_list_value.set(String::new());
                                            list_status_message.set(i18n.read().t("Added"));
                                            list_status_error.set(false);
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
                                        Err(error) => {
                                            list_status_message.set(format!("{}{}", i18n.read().t("Add failed: "), error));
                                            list_status_error.set(true);
                                        }
                                    }
                                    list_action.set(None);
                                });
                            }
                        },

                        {if list_action().as_deref() == Some("push") { i18n.read().t("Adding...") } else { "LPUSH".to_string() }}
                    }

                    button {
                        style: "{primary_action_button_style(list_action().is_some())}",
                        disabled: list_action().is_some(),
                        onclick: {
                            let pool = connection_pool.clone();
                            let key = display_key.clone();
                            move |_| {
                                let pool = pool.clone();
                                let key = key.clone();
                                let value = new_list_value();
                                spawn(async move {
                                    if value.trim().is_empty() {
                                        list_status_message.set(i18n.read().t("Value cannot be empty"));
                                        list_status_error.set(true);
                                        return;
                                    }

                                    list_action.set(Some("rpush".to_string()));
                                    match pool.list_push(&key, &value, false).await {
                                        Ok(_) => {
                                            new_list_value.set(String::new());
                                            list_status_message.set(i18n.read().t("Added"));
                                            list_status_error.set(false);
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
                                        Err(error) => {
                                            list_status_message.set(format!("{}{}", i18n.read().t("Add failed: "), error));
                                            list_status_error.set(true);
                                        }
                                    }
                                    list_action.set(None);
                                });
                            }
                        },

                        "RPUSH"
                    }

                    div {
                        style: "{data_section_count_style()}",

                        "List Items ({list_val.len()}/{list_total()})"
                    }
                }

                button {
                    flex_shrink: "0",
                    style: "{secondary_action_button_style()}",
                    title: i18n.read().t("Copy"),
                    onclick: {
                        let list = list_val.clone();
                        move |_| {
                            let json = serde_json::to_string_pretty(&list).unwrap_or_default();
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

            if !list_status_message.read().is_empty() {
                div {
                    style: "{status_banner_style(list_status_error())}",

                    "{list_status_message}"
                }
            }

            if list_val.len() > LARGE_KEY_THRESHOLD {
                LargeKeyWarning {
                    key_type: "List".to_string(),
                    size: list_val.len(),
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
                onscroll: {
                    let pool = connection_pool.clone();
                    let key = display_key.clone();
                    move |e| {
                        let scroll_top = e.data().scroll_top() as i32;
                        let scroll_height = e.data().scroll_height() as i32;
                        let client_height = e.data().client_height() as i32;

                        if list_has_more() && !list_loading_more() && scroll_height - scroll_top - client_height < 200 {
                            let pool = pool.clone();
                            let key = key.clone();
                            let total = list_total();
                            spawn(async move {
                                data_loader::load_more_list(
                                    pool,
                                    key,
                                    list_value,
                                    list_has_more,
                                    list_loading_more,
                                    total,
                                )
                                .await;
                            });
                        }
                    }
                },

                table {
                    width: "100%",
                    border_collapse: "collapse",

                    thead {
                        tr {
                            style: "{data_table_header_row_style()}",

                            th {
                                style: "{data_table_header_cell_style(Some(\"72px\"), \"left\")}",

                                "Index"
                            }

                            th {
                                style: "{data_table_header_cell_style(None, \"left\")}",

                                "Value"
                            }

                            th {
                                style: "{data_table_header_cell_style(Some(\"156px\"), \"left\")}",

                                "Action"
                            }
                        }
                    }

                    tbody {
                        if list_val.is_empty() {
                            tr {
                                td {
                                    colspan: "3",
                                    padding: "20px 12px",
                                    color: COLOR_TEXT_SUBTLE,
                                    text_align: "center",

                                    {i18n.read().t("This list has no elements")}
                                }
                            }
                        } else {
                            for (idx, value) in list_val.iter().enumerate() {
                                if editing_list_index() == Some(idx) {
                                    tr {
                                        background: ROW_EDIT_BG,
                                        border_bottom: "1px solid {COLOR_BORDER}",

                                        td {
                                            padding: "12px",
                                            color: COLOR_ACCENT,

                                            "{idx}"
                                        }

                                        td {
                                            padding: "12px",

                                            input {
                                                width: "100%",
                                                padding: "8px 10px",
                                                background: COLOR_BG,
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "6px",
                                                color: COLOR_TEXT,
                                                value: "{editing_list_value}",
                                                oninput: move |event| editing_list_value.set(event.value()),
                                            }
                                        }

                                        td {
                                            padding: "12px",

                                            div {
                                                display: "flex",
                                                gap: "8px",

                                                button {
                                                    style: "{primary_action_button_style(list_action().is_some())}",
                                                    disabled: list_action().is_some(),
                                                    onclick: {
                                                        let pool = connection_pool.clone();
                                                        let key = display_key.clone();
                                                        let idx = idx as i64;
                                                        move |_| {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let new_val = editing_list_value();
                                                            spawn(async move {
                                                                list_action.set(Some("set".to_string()));
                                                                match pool.list_set(&key, idx, &new_val).await {
                                                                    Ok(_) => {
                                                                        editing_list_index.set(None);
                                                                        list_status_message.set(i18n.read().t("Updated"));
                                                                        list_status_error.set(false);
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
                                                                    Err(error) => {
                                                                        list_status_message
                                                                            .set(format!("{}{}", i18n.read().t("Update failed: "), error));
                                                                        list_status_error.set(true);
                                                                    }
                                                                }
                                                                list_action.set(None);
                                                            });
                                                        }
                                                    },

                                                    {i18n.read().t("Save")}
                                                }

                                                button {
                                                    padding: "6px 10px",
                                                    background: COLOR_BG_TERTIARY,
                                                    color: COLOR_TEXT,
                                                    border: "none",
                                                    border_radius: "6px",
                                                    cursor: "pointer",
                                                    onclick: move |_| {
                                                        editing_list_index.set(None);
                                                    },

                                                    {i18n.read().t("Cancel")}
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    tr {
                                        key: "{idx}",
                                        border_bottom: "1px solid {COLOR_BORDER}",
                                        background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },

                                        td {
                                            padding: "12px",
                                            color: COLOR_TEXT_SECONDARY,

                                            "{idx}"
                                        }

                                        td {
                                            padding: "12px",
                                            color: COLOR_TEXT,
                                            font_family: "Consolas, monospace",
                                            font_size: "13px",
                                            word_break: "break-all",

                                            "{value}"
                                        }

                                        td {
                                            padding: "12px",

                                            div {
                                                display: "flex",
                                                gap: "6px",

                                                button {
                                                    style: "{compact_icon_action_button_style(false, false)}",
                                                    title: i18n.read().t("Copy"),
                                                    onclick: {
                                                        let value = value.clone();
                                                        move |_| {
                                                            match copy_value_to_clipboard(&value) {
                                                                Ok(_) => {
                                                                    toast_manager.write().success(&i18n.read().t("Copied"));
                                                                }
                                                                Err(error) => {
                                                                    toast_manager
                                                                        .write()
                                                                        .error(&format!("{}{}", i18n.read().t("Copy failed: "), error));
                                                                }
                                                            }
                                                        }
                                                    },

                                                    IconCopy { size: Some(15) }
                                                }

                                                button {
                                                    style: "{compact_icon_action_button_style(false, false)}",
                                                    title: i18n.read().t("Edit"),
                                                    onclick: {
                                                        let value = value.clone();
                                                        move |_| {
                                                            editing_list_index.set(Some(idx));
                                                            editing_list_value.set(value.clone());
                                                        }
                                                    },

                                                    IconEdit { size: Some(15) }
                                                }

                                                button {
                                                    style: "{compact_icon_action_button_style(true, false)}",
                                                    title: i18n.read().t("Delete"),
                                                    onclick: {
                                                        let pool = connection_pool.clone();
                                                        let key = display_key.clone();
                                                        let value = value.clone();
                                                        move |_| {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let value = value.clone();
                                                            spawn(async move {
                                                                list_action.set(Some("remove".to_string()));
                                                                match pool.list_remove(&key, 1, &value).await {
                                                                    Ok(_) => {
                                                                        list_status_message
                                                                            .set(i18n.read().t("Deleted"));
                                                                        list_status_error.set(false);
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
                                                                    Err(error) => {
                                                                        list_status_message
                                                                            .set(format!("{}{}", i18n.read().t("Delete failed: "), error));
                                                                        list_status_error.set(true);
                                                                    }
                                                                }
                                                                list_action.set(None);
                                                            });
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
                }
            }
        }
    }
}
