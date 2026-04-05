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
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_PRIMARY,
    COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE, COLOR_WARNING,
};
use crate::ui::icons::{IconCopy, IconEdit, IconTrash};
use crate::ui::pagination::LargeKeyWarning;
use crate::ui::ToastManager;
use dioxus::prelude::*;
use serde_json;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn ZSetPanel(
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
    zset_loading_more: Signal<bool>,
    mut zset_status_message: Signal<String>,
    mut zset_status_error: Signal<bool>,
    mut new_zset_member: Signal<String>,
    mut new_zset_score: Signal<String>,
    mut zset_action: Signal<Option<String>>,
    mut zset_search: Signal<String>,
    deleting_zset_member: Signal<Option<String>>,
    mut editing_zset_member: Signal<Option<String>>,
    mut editing_zset_score: Signal<String>,
) -> Element {
    let _ = deleting_zset_member;
    let zset_val = zset_value();
    let zset_search_val = zset_search();
    let normalized_zset_search = zset_search_val.trim().to_lowercase();
    let filtered_zset_members: Vec<(String, f64)> = zset_val
        .iter()
        .filter(|(member, _)| {
            if normalized_zset_search.is_empty() {
                true
            } else {
                member.to_lowercase().contains(&normalized_zset_search)
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
                        width: "160px",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{zset_search}",
                        placeholder: "搜索成员",
                        oninput: {
                            let pool = connection_pool.clone();
                            let key = display_key.clone();
                            move |event| {
                                let value = event.value();
                                let was_empty = zset_search().is_empty();
                                zset_search.set(value.clone());

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
                                            tracing::error!("重新加载 zset 数据失败: {}", e);
                                        }
                                    });
                                }
                            }
                        },
                    }

                    if zset_total() > PAGE_SIZE {
                        button {
                            padding: "6px 10px",
                            background: if zset_search().len() >= 2 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                            color: if zset_search().len() >= 2 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT_SECONDARY },
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            cursor: if zset_search().len() >= 2 { "pointer" } else { "not-allowed" },
                            font_size: "12px",
                            disabled: zset_search().len() < 2 || zset_loading_more(),
                            onclick: {
                                let pool = connection_pool.clone();
                                let key = display_key.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let key = key.clone();
                                    let pattern = zset_search();
                                    spawn(async move {
                                        data_loader::search_zset_server(
                                            pool,
                                            key,
                                            pattern,
                                            zset_value,
                                            zset_cursor,
                                            zset_has_more,
                                            zset_loading_more,
                                        )
                                        .await;
                                    });
                                }
                            },

                            if zset_loading_more() { "搜索中..." } else { "服务端搜索" }
                        }
                    }

                    input {
                        width: "80px",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{new_zset_score}",
                        placeholder: "Score",
                        oninput: move |event| new_zset_score.set(event.value()),
                    }

                    input {
                        width: "200px",
                        padding: "8px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        value: "{new_zset_member}",
                        placeholder: "输入新成员",
                        oninput: move |event| new_zset_member.set(event.value()),
                    }

                    button {
                        style: "{primary_action_button_style(zset_action().is_some())}",
                        disabled: zset_action().is_some(),
                        onclick: {
                            let pool = connection_pool.clone();
                            let key = display_key.clone();
                            move |_| {
                                let pool = pool.clone();
                                let key = key.clone();
                                let member = new_zset_member();
                                let score_str = new_zset_score();
                                spawn(async move {
                                    if member.trim().is_empty() {
                                        zset_status_message.set("成员不能为空".to_string());
                                        zset_status_error.set(true);
                                        return;
                                    }

                                    let score: f64 = match score_str.parse() {
                                        Ok(s) => s,
                                        Err(_) => {
                                            zset_status_message.set("Score 必须是有效数字".to_string());
                                            zset_status_error.set(true);
                                            return;
                                        }
                                    };

                                    zset_action.set(Some("add".to_string()));
                                    zset_status_message.set(String::new());
                                    zset_status_error.set(false);

                                    match pool.zset_add(&key, &member, score).await {
                                        Ok(_) => {
                                            new_zset_member.set(String::new());
                                            new_zset_score.set(String::new());
                                            zset_status_message.set("添加成功".to_string());
                                            zset_status_error.set(false);
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
                                            zset_status_message.set(format!("添加失败：{error}"));
                                            zset_status_error.set(true);
                                        }
                                    }
                                    zset_action.set(None);
                                });
                            }
                        },

                        if zset_action().as_deref() == Some("add") { "添加中..." } else { "添加成员" }
                    }

                    div {
                        style: "{data_section_count_style()}",

                        "ZSet Members ({zset_val.len()}/{zset_total()})"
                    }
                }

                button {
                    margin_left: "auto",
                    flex_shrink: "0",
                    style: "{secondary_action_button_style()}",
                    title: "复制",
                    onclick: {
                        let zset = zset_val.clone();
                        move |_| {
                            let json = serde_json::to_string_pretty(&zset).unwrap_or_default();
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

            if !zset_status_message.read().is_empty() {
                div {
                    style: "{status_banner_style(zset_status_error())}",

                    "{zset_status_message}"
                }
            }

            if zset_val.len() > LARGE_KEY_THRESHOLD {
                LargeKeyWarning {
                    key_type: "ZSet".to_string(),
                    size: zset_val.len(),
                    threshold: LARGE_KEY_THRESHOLD,
                }
            }

            div {
                flex: "1",
                min_height: "0",
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

                        if zset_has_more()
                            && !zset_loading_more()
                            && scroll_height - scroll_top - client_height < 200
                        {
                            let pool = pool.clone();
                            let key = key.clone();
                            let cursor = zset_cursor();
                            spawn(async move {
                                data_loader::load_more_zset(
                                    pool,
                                    key,
                                    zset_value,
                                    cursor,
                                    zset_cursor,
                                    zset_has_more,
                                    zset_loading_more,
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
                                style: "{data_table_header_cell_style(Some(\"100px\"), \"left\")}",

                                "Score"
                            }

                            th {
                                style: "{data_table_header_cell_style(None, \"left\")}",

                                "Member"
                            }

                            th {
                                style: "{data_table_header_cell_style(Some(\"100px\"), \"left\")}",

                                "Action"
                            }
                        }
                    }

                    tbody {
                        for (idx, (member, score)) in filtered_zset_members.iter().enumerate() {
                            if editing_zset_member() == Some(member.clone()) {
                                tr {
                                    background: ROW_EDIT_BG,
                                    border_bottom: "1px solid {COLOR_BORDER}",

                                    td {
                                        padding: "12px",
                                        color: COLOR_TEXT_SECONDARY,

                                        "{idx + 1}"
                                    }

                                    td {
                                        padding: "12px",

                                        input {
                                            width: "80px",
                                            padding: "8px 10px",
                                            background: COLOR_BG,
                                            border: "1px solid {COLOR_BORDER}",
                                            border_radius: "6px",
                                            color: COLOR_TEXT,
                                            value: "{editing_zset_score}",
                                            oninput: move |event| editing_zset_score.set(event.value()),
                                        }
                                    }

                                    td {
                                        padding: "12px",
                                        color: COLOR_ACCENT,

                                        "{member}"
                                    }

                                    td {
                                        padding: "12px",

                                        div {
                                            display: "flex",
                                            gap: "8px",

                                            button {
                                                style: "{primary_action_button_style(zset_action().is_some())}",
                                                disabled: zset_action().is_some(),
                                                onclick: {
                                                    let pool = connection_pool.clone();
                                                    let key = display_key.clone();
                                                    let member = member.clone();
                                                    move |_| {
                                                        let pool = pool.clone();
                                                        let key = key.clone();
                                                        let member = member.clone();
                                                        let score_str = editing_zset_score();
                                                        spawn(async move {
                                                            let score: f64 = match score_str.parse() {
                                                                Ok(s) => s,
                                                                Err(_) => {
                                                                    zset_status_message.set(
                                                                        "Score 必须是有效数字".to_string(),
                                                                    );
                                                                    zset_status_error.set(true);
                                                                    return;
                                                                }
                                                            };

                                                            zset_action.set(Some("update".to_string()));
                                                            match pool.zset_add(&key, &member, score).await {
                                                                Ok(_) => {
                                                                    editing_zset_member.set(None);
                                                                    zset_status_message.set("修改成功".to_string());
                                                                    zset_status_error.set(false);
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
                                                                    zset_status_message
                                                                        .set(format!("修改失败：{error}"));
                                                                    zset_status_error.set(true);
                                                                }
                                                            }
                                                            zset_action.set(None);
                                                        });
                                                    }
                                                },

                                                "保存"
                                            }

                                            button {
                                                padding: "6px 10px",
                                                background: COLOR_BG_TERTIARY,
                                                color: COLOR_TEXT,
                                                border: "none",
                                                border_radius: "6px",
                                                cursor: "pointer",
                                                onclick: move |_| {
                                                    editing_zset_member.set(None);
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
                                        color: COLOR_WARNING,
                                        font_family: "Consolas, monospace",
                                        font_size: "13px",

                                        "{score}"
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
                                                style: "{compact_icon_action_button_style(false, false)}",
                                                title: "修改 Score",
                                                onclick: {
                                                    let member = member.clone();
                                                    let score = *score;
                                                    move |_| {
                                                        editing_zset_member.set(Some(member.clone()));
                                                        editing_zset_score.set(score.to_string());
                                                    }
                                                },

                                                IconEdit { size: Some(15) }
                                            }

                                            button {
                                                style: "{compact_icon_action_button_style(true, zset_action().is_some())}",
                                                disabled: zset_action().is_some(),
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
                                                            zset_action
                                                                .set(Some(format!("delete:{}", member)));
                                                            match pool.zset_remove(&key, &member).await {
                                                                Ok(_) => {
                                                                    zset_status_message
                                                                        .set("删除成功".to_string());
                                                                    zset_status_error.set(false);
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
                                                                    zset_status_message
                                                                        .set(format!("删除失败：{error}"));
                                                                    zset_status_error.set(true);
                                                                }
                                                            }
                                                            zset_action.set(None);
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

            if zset_loading_more() {
                div {
                    padding: "12px",
                    text_align: "center",
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",

                    "加载中..."
                }
            }

            if zset_has_more() && !zset_loading_more() {
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
