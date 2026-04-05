use super::data_loader;
use super::formatters::copy_value_to_clipboard;
use super::styles::{
    compact_icon_action_button_style, data_section_controls_style, data_section_count_style,
    data_section_toolbar_style, data_table_header_cell_style, data_table_header_row_style,
    primary_action_button_style, secondary_action_button_style, status_banner_style,
};
use super::{BinaryFormat, LARGE_KEY_THRESHOLD, PAGE_SIZE, ROW_EDIT_BG};
use crate::connection::ConnectionPool;
use crate::redis::KeyInfo;
use crate::serialization::SerializationFormat;
use crate::theme::{
    COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_PRIMARY, COLOR_TEXT,
    COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use crate::ui::icons::{IconCopy, IconEdit, IconTrash};
use crate::ui::pagination::LargeKeyWarning;
use crate::ui::ToastManager;
use dioxus::prelude::*;
use serde_json;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn SetPanel(
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
    set_loading_more: Signal<bool>,
    mut set_status_message: Signal<String>,
    mut set_status_error: Signal<bool>,
    mut new_set_member: Signal<String>,
    mut set_action: Signal<Option<String>>,
    mut set_search: Signal<String>,
    mut editing_set_member: Signal<Option<String>>,
    mut editing_set_member_value: Signal<String>,
) -> Element {
    let set_val = set_value();
    let set_search_val = set_search();
    let normalized_set_search = set_search_val.trim().to_lowercase();
    let filtered_set_members: Vec<String> = set_val
        .iter()
        .filter(|member| {
            if normalized_set_search.is_empty() {
                true
            } else {
                member.to_lowercase().contains(&normalized_set_search)
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
                        width: "220px",
                        max_width: "100%",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{set_search}",
                        placeholder: "搜索成员",
                        oninput: {
                            let pool = connection_pool.clone();
                            let key = display_key.clone();
                            move |event| {
                                let value = event.value();
                                let was_empty = set_search().is_empty();
                                set_search.set(value.clone());

                                if value.is_empty() && !was_empty {
                                    let pool = pool.clone();
                                    let key = key.clone();
                                    spawn(async move {
                                        if let Err(error) = data_loader::load_key_data(
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
                                            tracing::error!("重新加载 set 数据失败: {}", error);
                                        }
                                    });
                                }
                            }
                        },
                    }

                    if set_total() > PAGE_SIZE {
                        button {
                            padding: "6px 10px",
                            background: if set_search().len() >= 2 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                            color: if set_search().len() >= 2 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT_SECONDARY },
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            cursor: if set_search().len() >= 2 { "pointer" } else { "not-allowed" },
                            font_size: "12px",
                            disabled: set_search().len() < 2 || set_loading_more(),
                            onclick: {
                                let pool = connection_pool.clone();
                                let key = display_key.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let key = key.clone();
                                    let pattern = set_search();
                                    spawn(async move {
                                        data_loader::search_set_server(
                                            pool,
                                            key,
                                            pattern,
                                            set_value,
                                            set_cursor,
                                            set_has_more,
                                            set_loading_more,
                                        )
                                        .await;
                                    });
                                }
                            },

                            if set_loading_more() { "搜索中..." } else { "服务端搜索" }
                        }
                    }

                    input {
                        width: "220px",
                        max_width: "100%",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{new_set_member}",
                        placeholder: "输入新成员",
                        oninput: move |event| new_set_member.set(event.value()),
                    }

                    button {
                        style: "{primary_action_button_style(set_action().is_some())}",
                        disabled: set_action().is_some(),
                        onclick: {
                            let pool = connection_pool.clone();
                            let key = display_key.clone();
                            move |_| {
                                let pool = pool.clone();
                                let key = key.clone();
                                let member = new_set_member();
                                spawn(async move {
                                    if member.trim().is_empty() {
                                        set_status_message.set("成员不能为空".to_string());
                                        set_status_error.set(true);
                                        return;
                                    }

                                    set_action.set(Some("add".to_string()));
                                    set_status_message.set(String::new());
                                    set_status_error.set(false);

                                    match pool.set_add(&key, &member).await {
                                        Ok(true) => {
                                            new_set_member.set(String::new());
                                            set_status_message.set("添加成功".to_string());
                                            set_status_error.set(false);
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
                                            set_status_message.set("成员已存在".to_string());
                                            set_status_error.set(true);
                                        }
                                        Err(error) => {
                                            set_status_message.set(format!("添加失败：{error}"));
                                            set_status_error.set(true);
                                        }
                                    }
                                    set_action.set(None);
                                });
                            }
                        },

                        if set_action().as_deref() == Some("add") { "添加中..." } else { "添加成员" }
                    }

                    div {
                        style: "{data_section_count_style()}",

                        "Set Members ({filtered_set_members.len()}/{set_total()})"
                    }
                }

                button {
                    margin_left: "auto",
                    flex_shrink: "0",
                    style: "{secondary_action_button_style()}",
                    title: "复制",
                    onclick: {
                        let set = set_val.clone();
                        move |_| {
                            let json = serde_json::to_string_pretty(&set).unwrap_or_default();
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

            if !set_status_message.read().is_empty() {
                div {
                    style: "{status_banner_style(set_status_error())}",

                    "{set_status_message}"
                }
            }

            if set_total().max(set_val.len()) > LARGE_KEY_THRESHOLD {
                LargeKeyWarning {
                    key_type: "Set".to_string(),
                    size: set_total().max(set_val.len()),
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

                        if set_has_more()
                            && !set_loading_more()
                            && scroll_height - scroll_top - client_height < 200
                        {
                            let pool = pool.clone();
                            let key = key.clone();
                            let cursor = set_cursor();
                            spawn(async move {
                                data_loader::load_more_set(
                                    pool,
                                    key,
                                    set_value,
                                    cursor,
                                    set_cursor,
                                    set_has_more,
                                    set_loading_more,
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

                                "#"
                            }

                            th {
                                style: "{data_table_header_cell_style(None, \"left\")}",

                                "Member"
                            }

                            th {
                                style: "{data_table_header_cell_style(Some(\"156px\"), \"left\")}",

                                "Action"
                            }
                        }
                    }

                    tbody {
                        if filtered_set_members.is_empty() {
                            tr {
                                td {
                                    colspan: "3",
                                    padding: "20px 12px",
                                    color: COLOR_TEXT_SUBTLE,
                                    text_align: "center",

                                    if set_search().trim().is_empty() {
                                        "当前集合没有成员"
                                    } else {
                                        "未找到匹配的成员"
                                    }
                                }
                            }
                        } else {
                            for (idx, member) in filtered_set_members.iter().enumerate() {
                                if editing_set_member() == Some(member.clone()) {
                                    tr {
                                        key: "edit-{member}",
                                        background: ROW_EDIT_BG,
                                        border_bottom: "1px solid {COLOR_BORDER}",

                                        td {
                                            padding: "10px 12px",
                                            color: COLOR_TEXT_SECONDARY,

                                            "{idx + 1}"
                                        }

                                        td {
                                            padding: "10px 12px",

                                            input {
                                                width: "100%",
                                                padding: "8px 10px",
                                                background: COLOR_BG,
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "6px",
                                                color: COLOR_TEXT,
                                                font_family: "Consolas, monospace",
                                                font_size: "13px",
                                                value: "{editing_set_member_value}",
                                                oninput: move |event| editing_set_member_value.set(event.value()),
                                            }
                                        }

                                        td {
                                            padding: "10px 12px",

                                            div {
                                                display: "flex",
                                                gap: "6px",

                                                button {
                                                    style: "{primary_action_button_style(set_action().is_some())}",
                                                    disabled: set_action().is_some(),
                                                    onclick: {
                                                        let pool = connection_pool.clone();
                                                        let key = display_key.clone();
                                                        let old_member = member.clone();
                                                        move |_| {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let old_member = old_member.clone();
                                                            let new_member = editing_set_member_value();
                                                            spawn(async move {
                                                                if new_member.trim().is_empty() {
                                                                    set_status_message.set("成员不能为空".to_string());
                                                                    set_status_error.set(true);
                                                                    return;
                                                                }

                                                                set_action.set(Some(format!("edit:{old_member}")));
                                                                set_status_message.set(String::new());
                                                                set_status_error.set(false);

                                                                let edit_result = if new_member == old_member {
                                                                    Ok(())
                                                                } else {
                                                                    match pool.set_remove(&key, &old_member).await {
                                                                        Ok(_) => {
                                                                            pool.set_add(&key, &new_member)
                                                                                .await
                                                                                .map(|_| ())
                                                                        }
                                                                        Err(error) => Err(error),
                                                                    }
                                                                };

                                                                match edit_result {
                                                                    Ok(_) => {
                                                                        editing_set_member.set(None);
                                                                        editing_set_member_value.set(String::new());
                                                                        set_status_message.set("修改成功".to_string());
                                                                        set_status_error.set(false);
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
                                                                        set_status_message.set(format!("修改失败：{error}"));
                                                                        set_status_error.set(true);
                                                                    }
                                                                }

                                                                set_action.set(None);
                                                            });
                                                        }
                                                    },

                                                    "保存"
                                                }

                                                button {
                                                    style: "{secondary_action_button_style()}",
                                                    onclick: move |_| {
                                                        editing_set_member.set(None);
                                                        editing_set_member_value.set(String::new());
                                                    },

                                                    "取消"
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    tr {
                                        key: "{member}",
                                        border_bottom: "1px solid {COLOR_BORDER}",
                                        background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },

                                        td {
                                            padding: "10px 12px",
                                            color: COLOR_TEXT_SECONDARY,

                                            "{idx + 1}"
                                        }

                                        td {
                                            padding: "10px 12px",
                                            color: COLOR_TEXT,
                                            font_family: "Consolas, monospace",
                                            font_size: "13px",
                                            word_break: "break-all",

                                            "{member}"
                                        }

                                        td {
                                            padding: "10px 12px",

                                            div {
                                                display: "flex",
                                                gap: "6px",

                                                button {
                                                    style: "{compact_icon_action_button_style(false, false)}",
                                                    title: "复制",
                                                    onclick: {
                                                        let member = member.clone();
                                                        move |_| {
                                                            match copy_value_to_clipboard(&member) {
                                                                Ok(_) => {
                                                                    toast_manager.write().success("复制成功");
                                                                }
                                                                Err(error) => {
                                                                    toast_manager
                                                                        .write()
                                                                        .error(&format!("复制失败：{error}"));
                                                                }
                                                            }
                                                        }
                                                    },

                                                    IconCopy { size: Some(15) }
                                                }

                                                button {
                                                    style: "{compact_icon_action_button_style(false, set_action().is_some())}",
                                                    disabled: set_action().is_some(),
                                                    title: "编辑",
                                                    onclick: {
                                                        let member = member.clone();
                                                        move |_| {
                                                            editing_set_member.set(Some(member.clone()));
                                                            editing_set_member_value.set(member.clone());
                                                        }
                                                    },

                                                    IconEdit { size: Some(15) }
                                                }

                                                button {
                                                    style: "{compact_icon_action_button_style(true, set_action().is_some())}",
                                                    disabled: set_action().is_some(),
                                                    title: "删除",
                                                    onclick: {
                                                        let pool = connection_pool.clone();
                                                        let key = display_key.clone();
                                                        let member = member.clone();
                                                        move |_| {
                                                            let pool = pool.clone();
                                                            let key = key.clone();
                                                            let member = member.clone();
                                                            spawn(async move {
                                                                set_action.set(Some(format!("delete:{member}")));
                                                                match pool.set_remove(&key, &member).await {
                                                                    Ok(_) => {
                                                                        set_status_message.set("删除成功".to_string());
                                                                        set_status_error.set(false);
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
                                                                        set_status_message.set(format!("删除失败：{error}"));
                                                                        set_status_error.set(true);
                                                                    }
                                                                }
                                                                set_action.set(None);
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

            if set_loading_more() {
                div {
                    padding: "12px",
                    text_align: "center",
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",

                    "加载中..."
                }
            }

            if set_has_more() && !set_loading_more() {
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
