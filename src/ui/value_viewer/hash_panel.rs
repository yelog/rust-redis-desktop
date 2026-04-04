use super::data_loader;
use super::formatters::{copy_value_to_clipboard, sorted_hash_entries};
use super::styles::{
    compact_icon_action_button_style, data_section_controls_style, data_section_count_style,
    data_section_toolbar_style, data_table_header_cell_style, data_table_header_row_style,
    destructive_action_button_style, overlay_modal_actions_style, overlay_modal_backdrop_style,
    overlay_modal_body_style, overlay_modal_keyframes, overlay_modal_surface_style,
    overlay_modal_title_style, primary_action_button_style, secondary_action_button_style,
    status_banner_style,
};
use super::{
    BinaryFormat, HashDeleteTarget, LARGE_KEY_THRESHOLD, PAGE_SIZE, ROW_CREATE_BG, ROW_EDIT_BG,
};
use crate::connection::ConnectionPool;
use crate::redis::KeyInfo;
use crate::serialization::SerializationFormat;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_PRIMARY,
    COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use crate::ui::icons::{IconCopy, IconEdit, IconTrash};
use crate::ui::pagination::LargeKeyWarning;
use crate::ui::ToastManager;
use dioxus::prelude::*;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn HashPanel(
    connection_pool: ConnectionPool,
    display_key: String,
    on_refresh: EventHandler<()>,
    toast_manager: Signal<ToastManager>,
    key_info: Signal<Option<KeyInfo>>,
    string_value: Signal<String>,
    mut hash_value: Signal<HashMap<String, String>>,
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
    hash_loading_more: Signal<bool>,
    list_has_more: Signal<bool>,
    list_total: Signal<usize>,
    set_cursor: Signal<u64>,
    set_total: Signal<usize>,
    set_has_more: Signal<bool>,
    zset_cursor: Signal<u64>,
    zset_total: Signal<usize>,
    zset_has_more: Signal<bool>,
    mut hash_search: Signal<String>,
    mut hash_status_message: Signal<String>,
    mut hash_status_error: Signal<bool>,
    mut editing_hash_field: Signal<Option<String>>,
    mut editing_hash_key: Signal<String>,
    mut editing_hash_value: Signal<String>,
    mut creating_hash_row: Signal<bool>,
    mut new_hash_key: Signal<String>,
    mut new_hash_value: Signal<String>,
    mut deleting_hash_field: Signal<Option<HashDeleteTarget>>,
    deleting_hash_field_exiting: Signal<bool>,
    mut hash_action: Signal<Option<String>>,
) -> Element {
    let hash_val = hash_value();
    let active_hash_action = hash_action();
    let editing_field_name = editing_hash_field();

    let sorted_entries = sorted_hash_entries(&hash_val);
    let search_value = hash_search();
    let normalized_search = search_value.trim().to_lowercase();
    let filtered_entries: Vec<(String, String)> = sorted_entries
        .into_iter()
        .filter(|(field, value)| {
            if normalized_search.is_empty() {
                true
            } else {
                field.to_lowercase().contains(&normalized_search)
                    || value.to_lowercase().contains(&normalized_search)
            }
        })
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
                        width: "280px",
                        max_width: "100%",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{search_value}",
                        placeholder: "搜索 key 或 value",
                        oninput: {
                            let pool = connection_pool.clone();
                            let key = display_key.clone();
                            move |event| {
                                let value = event.value();
                                let was_empty = hash_search().is_empty();
                                hash_search.set(value.clone());

                                if value.is_empty() && !was_empty {
                                    let pool = pool.clone();
                                    let key = key.clone();
                                    spawn(async move {
                                        if let Err(e) = data_loader::load_key_data(
                                            pool,
                                            key,
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
                                            tracing::error!("重新加载 hash 数据失败: {}", e);
                                        }
                                    });
                                }
                            }
                        },
                    }

                    if hash_total() > PAGE_SIZE {
                        button {
                            padding: "6px 10px",
                            background: if hash_search().len() >= 2 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                            color: if hash_search().len() >= 2 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT_SECONDARY },
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            cursor: if hash_search().len() >= 2 { "pointer" } else { "not-allowed" },
                            font_size: "12px",
                            disabled: hash_search().len() < 2 || hash_loading_more(),
                            onclick: {
                                let pool = connection_pool.clone();
                                let key = display_key.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let key = key.clone();
                                    let pattern = hash_search();
                                    spawn(async move {
                                        data_loader::search_hash_server(
                                            pool,
                                            key,
                                            pattern,
                                            hash_value,
                                            hash_cursor,
                                            hash_has_more,
                                            hash_loading_more,
                                        )
                                        .await;
                                    });
                                }
                            },

                            if hash_loading_more() { "搜索中..." } else { "服务端搜索" }
                        }
                    }

                    button {
                        style: "{primary_action_button_style(active_hash_action.is_some())}",
                        disabled: active_hash_action.is_some(),
                        onclick: move |_| {
                            creating_hash_row.set(true);
                            editing_hash_field.set(None);
                            editing_hash_key.set(String::new());
                            editing_hash_value.set(String::new());
                            new_hash_key.set(String::new());
                            new_hash_value.set(String::new());
                            hash_status_message.set(String::new());
                            hash_status_error.set(false);
                        },

                        "+ 新增行"
                    }

                    div {
                        style: "{data_section_count_style()}",

                        "Hash Fields ({hash_val.len()}/{hash_total()})"
                    }
                }

                button {
                    margin_left: "auto",
                    flex_shrink: "0",
                    style: "{secondary_action_button_style()}",
                    title: "复制",
                    onclick: {
                        let hash = hash_val.clone();
                        move |_| {
                            let json = serde_json::to_string_pretty(&hash).unwrap_or_default();
                            match copy_value_to_clipboard(&json) {
                                Ok(_) => {
                                    toast_manager.write().success("复制成功");
                                }
                                Err(error) => {
                                    toast_manager.write().error(&format!("复制失败：{error}"));
                                }
                            }
                        }
                    },

                    IconCopy { size: Some(14) }
                    "复制"
                }
            }

            if !hash_status_message.read().is_empty() {
                div {
                    style: "{status_banner_style(hash_status_error())}",

                    "{hash_status_message}"
                }
            }

            if hash_total().max(hash_val.len()) > LARGE_KEY_THRESHOLD {
                LargeKeyWarning {
                    key_type: "Hash".to_string(),
                    size: hash_total().max(hash_val.len()),
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

                        if hash_has_more() && !hash_loading_more() && scroll_height - scroll_top - client_height < 200 {
                            let pool = pool.clone();
                            let key = key.clone();
                            let cursor = hash_cursor();
                            spawn(async move {
                                data_loader::load_more_hash(
                                    pool,
                                    key,
                                    hash_value,
                                    cursor,
                                    hash_cursor,
                                    hash_has_more,
                                    hash_loading_more,
                                    hash_total,
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

                                "ID"
                            }

                            th {
                                style: "{data_table_header_cell_style(Some(\"32%\"), \"left\")}",

                                "key"
                            }

                            th {
                                style: "{data_table_header_cell_style(None, \"left\")}",

                                "value"
                            }

                            th {
                                style: "{data_table_header_cell_style(Some(\"156px\"), \"left\")}",

                                "action"
                            }
                        }
                    }

                    tbody {
                        if creating_hash_row() {
                            tr {
                                background: ROW_CREATE_BG,
                                border_bottom: "1px solid {COLOR_BORDER}",

                                td {
                                    padding: "12px",
                                    color: COLOR_ACCENT,
                                    vertical_align: "top",

                                    "+"
                                }

                                td {
                                    padding: "12px",
                                    vertical_align: "top",

                                    input {
                                        width: "100%",
                                        padding: "8px 10px",
                                        background: COLOR_BG,
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "6px",
                                        color: COLOR_TEXT,
                                        value: "{new_hash_key}",
                                        placeholder: "输入 field key",
                                        oninput: move |event| new_hash_key.set(event.value()),
                                    }
                                }

                                td {
                                    padding: "12px",
                                    vertical_align: "top",

                                    textarea {
                                        width: "100%",
                                        min_height: "92px",
                                        padding: "8px 10px",
                                        background: COLOR_BG,
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "6px",
                                        color: COLOR_TEXT,
                                        font_family: "Consolas, 'Courier New', monospace",
                                        resize: "vertical",
                                        value: "{new_hash_value}",
                                        placeholder: "输入 field value",
                                        oninput: move |event| new_hash_value.set(event.value()),
                                    }
                                }

                                td {
                                    padding: "12px",
                                    vertical_align: "top",

                                    div {
                                        display: "flex",
                                        gap: "8px",
                                        align_items: "center",
                                        flex_wrap: "wrap",

                                        button {
                                            style: "{primary_action_button_style(active_hash_action.is_some())}",
                                            disabled: active_hash_action.is_some(),
                                            onclick: {
                                                let pool = connection_pool.clone();
                                                let key = display_key.clone();
                                                let existing_hash = hash_val.clone();
                                                move |_| {
                                                    let pool = pool.clone();
                                                    let key = key.clone();
                                                    let existing_hash = existing_hash.clone();
                                                    let field = new_hash_key().trim().to_string();
                                                    let value = new_hash_value();
                                                    spawn(async move {
                                                        if field.is_empty() {
                                                            hash_status_message.set("新增失败：key 不能为空".to_string());
                                                            hash_status_error.set(true);
                                                            return;
                                                        }

                                                        if existing_hash.contains_key(&field) {
                                                            hash_status_message.set("新增失败：key 已存在".to_string());
                                                            hash_status_error.set(true);
                                                            return;
                                                        }

                                                        hash_action.set(Some("create".to_string()));
                                                        hash_status_message.set(String::new());
                                                        hash_status_error.set(false);

                                                        match pool.hash_set_field(&key, &field, &value).await {
                                                            Ok(_) => {
                                                                creating_hash_row.set(false);
                                                                new_hash_key.set(String::new());
                                                                new_hash_value.set(String::new());
                                                                hash_status_message.set("新增成功".to_string());
                                                                hash_status_error.set(false);
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
                                                                    hash_status_message.set(error);
                                                                    hash_status_error.set(true);
                                                                } else {
                                                                    on_refresh.call(());
                                                                }
                                                            }
                                                            Err(error) => {
                                                                tracing::error!("Failed to create hash field: {}", error);
                                                                hash_status_message.set(format!("新增失败：{error}"));
                                                                hash_status_error.set(true);
                                                            }
                                                        }

                                                        hash_action.set(None);
                                                    });
                                                }
                                            },

                                            if active_hash_action.as_deref() == Some("create") { "保存中..." } else { "保存" }
                                        }

                                        button {
                                            padding: "6px 10px",
                                            background: COLOR_BG_TERTIARY,
                                            color: COLOR_TEXT,
                                            border: "none",
                                            border_radius: "6px",
                                            cursor: "pointer",
                                            disabled: active_hash_action.is_some(),
                                            onclick: move |_| {
                                                creating_hash_row.set(false);
                                                new_hash_key.set(String::new());
                                                new_hash_value.set(String::new());
                                            },

                                            "取消"
                                        }
                                    }
                                }
                            }
                        }

                        for (index, (field, value)) in filtered_entries.iter().enumerate() {
                            if editing_field_name.as_deref() == Some(field.as_str()) {
                                tr {
                                    key: "edit-{field}",
                                    background: ROW_EDIT_BG,
                                    border_bottom: "1px solid {COLOR_BORDER}",

                                    td {
                                        padding: "12px",
                                        color: COLOR_ACCENT,
                                        vertical_align: "top",

                                        "{index + 1}"
                                    }

                                    td {
                                        padding: "12px",
                                        vertical_align: "top",

                                        input {
                                            width: "100%",
                                            padding: "8px 10px",
                                            background: COLOR_BG,
                                            border: "1px solid {COLOR_BORDER}",
                                            border_radius: "6px",
                                            color: COLOR_TEXT,
                                            value: "{editing_hash_key}",
                                            oninput: move |event| editing_hash_key.set(event.value()),
                                        }
                                    }

                                    td {
                                        padding: "12px",
                                        vertical_align: "top",

                                        textarea {
                                            width: "100%",
                                            min_height: "92px",
                                            padding: "8px 10px",
                                            background: COLOR_BG,
                                            border: "1px solid {COLOR_BORDER}",
                                            border_radius: "6px",
                                            color: COLOR_TEXT,
                                            font_family: "Consolas, 'Courier New', monospace",
                                            resize: "vertical",
                                            value: "{editing_hash_value}",
                                            oninput: move |event| editing_hash_value.set(event.value()),
                                        }
                                    }

                                    td {
                                        padding: "12px",
                                        vertical_align: "top",

                                        div {
                                            display: "flex",
                                            gap: "8px",
                                            align_items: "center",
                                            flex_wrap: "wrap",

                                            button {
                                                style: "{primary_action_button_style(active_hash_action.is_some())}",
                                                disabled: active_hash_action.is_some(),
                                                onclick: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    let original_field = field.clone();
                                                    let existing_hash = hash_val.clone();
                                                    move |_| {
                                                        let pool = pool.clone();
                                                        let key = key.clone();
                                                        let original_field = original_field.clone();
                                                        let existing_hash = existing_hash.clone();
                                                        let next_field = editing_hash_key().trim().to_string();
                                                        let next_value = editing_hash_value();
                                                        spawn(async move {
                                                            if next_field.is_empty() {
                                                                hash_status_message.set("保存失败：key 不能为空".to_string());
                                                                hash_status_error.set(true);
                                                                return;
                                                            }

                                                            if next_field != original_field && existing_hash.contains_key(&next_field) {
                                                                hash_status_message.set("保存失败：目标 key 已存在".to_string());
                                                                hash_status_error.set(true);
                                                                return;
                                                            }

                                                            hash_action.set(Some(format!("save:{}", original_field)));
                                                            hash_status_message.set(String::new());
                                                            hash_status_error.set(false);

                                                            let save_result = if next_field == original_field {
                                                                pool.hash_set_field(&key, &next_field, &next_value).await
                                                            } else {
                                                                match pool.hash_set_field(&key, &next_field, &next_value).await {
                                                                    Ok(_) => pool.hash_delete_field(&key, &original_field).await.map(|_| ()),
                                                                    Err(error) => Err(error),
                                                                }
                                                            };

                                                            match save_result {
                                                                Ok(_) => {
                                                                    editing_hash_field.set(None);
                                                                    editing_hash_key.set(String::new());
                                                                    editing_hash_value.set(String::new());
                                                                    hash_status_message.set("保存成功".to_string());
                                                                    hash_status_error.set(false);
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
                                                                        hash_status_message.set(error);
                                                                        hash_status_error.set(true);
                                                                    } else {
                                                                        on_refresh.call(());
                                                                    }
                                                                }
                                                                Err(error) => {
                                                                    tracing::error!("Failed to save hash field: {}", error);
                                                                    hash_status_message.set(format!("保存失败：{error}"));
                                                                    hash_status_error.set(true);
                                                                }
                                                            }

                                                            hash_action.set(None);
                                                        });
                                                    }
                                                },

                                                if active_hash_action.as_deref() == Some(format!("save:{field}").as_str()) { "保存中..." } else { "保存" }
                                            }

                                            button {
                                                padding: "6px 10px",
                                                background: COLOR_BG_TERTIARY,
                                                color: COLOR_TEXT,
                                                border: "none",
                                                border_radius: "6px",
                                                cursor: "pointer",
                                                disabled: active_hash_action.is_some(),
                                                onclick: move |_| {
                                                    editing_hash_field.set(None);
                                                    editing_hash_key.set(String::new());
                                                    editing_hash_value.set(String::new());
                                                },

                                                "取消"
                                            }
                                        }
                                    }
                                }
                            } else {
                                tr {
                                    key: "{field}",
                                    border_bottom: "1px solid {COLOR_BORDER}",
                                    background: COLOR_BG_SECONDARY,

                                    td {
                                        padding: "12px",
                                        color: COLOR_TEXT_SECONDARY,
                                        vertical_align: "top",

                                        "{index + 1}"
                                    }

                                    td {
                                        padding: "12px",
                                        vertical_align: "top",

                                        div {
                                            color: COLOR_ACCENT,
                                            font_size: "13px",
                                            line_height: "1.5",
                                            word_break: "break-all",

                                            "{field}"
                                        }
                                    }

                                    td {
                                        padding: "12px",
                                        vertical_align: "top",

                                        div {
                                            color: COLOR_TEXT,
                                            font_size: "13px",
                                            line_height: "1.6",
                                            white_space: "pre-wrap",
                                            word_break: "break-all",

                                            "{value}"
                                        }
                                    }

                                    td {
                                        padding: "12px",
                                        vertical_align: "top",

                                        div {
                                            display: "flex",
                                            gap: "6px",
                                            align_items: "center",
                                            flex_wrap: "nowrap",

                                            button {
                                                style: "{compact_icon_action_button_style(false, active_hash_action.is_some())}",
                                                disabled: active_hash_action.is_some(),
                                                title: "复制值",
                                                aria_label: "复制值",
                                                onclick: {
                                                    let value = value.clone();
                                                    move |_| {
                                                        match copy_value_to_clipboard(&value) {
                                                            Ok(_) => {
                                                                toast_manager.write().success("复制成功");
                                                            }
                                                            Err(error) => {
                                                                tracing::error!("Failed to copy hash value: {}", error);
                                                                toast_manager.write().error(&format!("复制失败：{error}"));
                                                            }
                                                        }
                                                    }
                                                },

                                                IconCopy { size: Some(15) }
                                            }

                                            button {
                                                style: "{compact_icon_action_button_style(false, active_hash_action.is_some())}",
                                                disabled: active_hash_action.is_some(),
                                                title: "编辑 key-value",
                                                aria_label: "编辑 key-value",
                                                onclick: {
                                                    let field = field.clone();
                                                    let value = value.clone();
                                                    move |_| {
                                                        creating_hash_row.set(false);
                                                        new_hash_key.set(String::new());
                                                        new_hash_value.set(String::new());
                                                        editing_hash_field.set(Some(field.clone()));
                                                        editing_hash_key.set(field.clone());
                                                        editing_hash_value.set(value.clone());
                                                        hash_status_message.set(String::new());
                                                        hash_status_error.set(false);
                                                    }
                                                },

                                                IconEdit { size: Some(15) }
                                            }

                                            button {
                                                style: "{compact_icon_action_button_style(true, active_hash_action.is_some())}",
                                                disabled: active_hash_action.is_some(),
                                                onclick: {
                                                    let field = field.clone();
                                                    move |_| {
                                                        deleting_hash_field
                                                            .set(Some(HashDeleteTarget { field: field.clone() }));
                                                    }
                                                },
                                                title: "删除",
                                                aria_label: "删除",

                                                IconTrash { size: Some(15) }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if filtered_entries.is_empty() && !creating_hash_row() {
                            tr {
                                td {
                                    colspan: "4",
                                    padding: "20px 12px",
                                    color: COLOR_TEXT_SECONDARY,
                                    text_align: "center",

                                    if normalized_search.is_empty() {
                                        "当前 hash 没有字段"
                                    } else {
                                        "没有匹配的字段"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(target) = deleting_hash_field() {
                div {
                    style: "{overlay_modal_backdrop_style(deleting_hash_field_exiting())}",

                    style {
                        "{overlay_modal_keyframes()}"
                    }

                    div {
                        style: "{overlay_modal_surface_style(\"420px\", deleting_hash_field_exiting())}",

                        h3 {
                            style: "{overlay_modal_title_style()}",

                            "确认删除"
                        }

                        p {
                            style: "{overlay_modal_body_style()}",

                            "确定删除 hash field '{target.field}' 吗？"
                        }

                        div {
                            style: "{overlay_modal_actions_style()}",

                            button {
                                style: "{secondary_action_button_style()}",
                                disabled: active_hash_action.is_some(),
                                onclick: {
                                    let deleting_hash_field = deleting_hash_field.clone();
                                    let mut deleting_hash_field_exiting = deleting_hash_field_exiting.clone();
                                    move |_| {
                                        deleting_hash_field_exiting.set(true);
                                        let mut dhf = deleting_hash_field.clone();
                                        let mut dhfe = deleting_hash_field_exiting.clone();
                                        spawn(async move {
                                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                            dhf.set(None);
                                            dhfe.set(false);
                                        });
                                    }
                                },

                                "取消"
                            }

                            button {
                                style: "{destructive_action_button_style(active_hash_action.is_some())}",
                                disabled: active_hash_action.is_some(),
                                onclick: {
                                    let pool = connection_pool.clone();
                                    let key = display_key.clone();
                                    let field = target.field.clone();
                                    let delete_action = format!("delete:{}", target.field);
                                    let deleting_hash_field = deleting_hash_field.clone();
                                    let mut deleting_hash_field_exiting = deleting_hash_field_exiting.clone();
                                    move |_| {
                                        let pool = pool.clone();
                                        let key = key.clone();
                                        let field = field.clone();
                                        let delete_action = delete_action.clone();
                                        spawn(async move {
                                            hash_action.set(Some(delete_action));
                                            hash_status_message.set(String::new());
                                            hash_status_error.set(false);

                                            match pool.hash_delete_field(&key, &field).await {
                                                Ok(_) => {
                                                    deleting_hash_field_exiting.set(true);
                                                    let mut dhf = deleting_hash_field.clone();
                                                    let mut dhfe = deleting_hash_field_exiting.clone();
                                                    if editing_hash_field().as_deref() == Some(field.as_str()) {
                                                        editing_hash_field.set(None);
                                                        editing_hash_key.set(String::new());
                                                        editing_hash_value.set(String::new());
                                                    }
                                                    hash_status_message.set("删除成功".to_string());
                                                    hash_status_error.set(false);
                                                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                                    dhf.set(None);
                                                    dhfe.set(false);
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
                                                        hash_status_message.set(error);
                                                        hash_status_error.set(true);
                                                    } else {
                                                        on_refresh.call(());
                                                    }
                                                }
                                                Err(error) => {
                                                    tracing::error!("Failed to delete hash field: {}", error);
                                                    hash_status_message.set(format!("删除失败：{error}"));
                                                    hash_status_error.set(true);
                                                }
                                            }

                                            hash_action.set(None);
                                        });
                                    }
                                },

                                if active_hash_action.as_deref()
                                    == Some(format!("delete:{}", target.field).as_str())
                                {
                                    "删除中..."
                                } else {
                                    "确认删除"
                                }
                            }
                        }
                    }
                }
            }

            if hash_loading_more() {
                div {
                    padding: "12px",
                    text_align: "center",
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",

                    "加载中..."
                }
            }

            if hash_has_more() && !hash_loading_more() {
                div {
                    padding: "8px",
                    text_align: "center",
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "12px",

                    "向下滚动加载更多..."
                }
            }
        }
    }
}
