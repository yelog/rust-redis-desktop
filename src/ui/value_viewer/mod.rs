mod bitmap_viewer;
mod data_loader;
mod formatters;
mod image_preview;
mod protobuf_viewer;
mod styles;

pub use self::image_preview::{ImagePreview, PreviewImageData, PREVIEW_IMAGE};
use self::bitmap_viewer::BitmapViewer;
use self::formatters::{
    base64_decode, copy_value_to_clipboard, detect_image_format, format_bytes,
    format_memory_usage, format_ttl_label, is_binary_data, sorted_hash_entries,
    value_metric_label,
};
use self::protobuf_viewer::ProtobufViewer;
use self::styles::{
    compact_icon_action_button_style, data_section_controls_style, data_section_count_style,
    data_section_toolbar_style, data_table_header_cell_style, data_table_header_row_style,
    destructive_action_button_style, image_preview_button_style, image_preview_info_chip_style,
    overlay_modal_actions_style, overlay_modal_backdrop_style, overlay_modal_body_style,
    overlay_modal_keyframes, overlay_modal_surface_style, overlay_modal_title_style,
    primary_action_button_style, secondary_action_button_style, status_banner_style,
};
use crate::connection::ConnectionPool;
use crate::protobuf_schema::PROTO_REGISTRY;
use crate::redis::{KeyInfo, KeyType};
use crate::serialization::{
    detect_serialization_format, is_java_serialization, is_protobuf_data, parse_to_json,
    SerializationFormat,
};
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER,
    COLOR_BUTTON_SECONDARY, COLOR_BUTTON_SECONDARY_BORDER, COLOR_ERROR, COLOR_ERROR_BG, COLOR_INFO,
    COLOR_INFO_BG, COLOR_OVERLAY_BACKDROP, COLOR_PRIMARY, COLOR_ROW_CREATE_BG, COLOR_ROW_EDIT_BG,
    COLOR_SUCCESS, COLOR_SUCCESS_BG, COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
    COLOR_TEXT_SUBTLE, COLOR_WARNING,
};
use crate::ui::context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
use crate::ui::editable_field::EditableField;
use crate::ui::icons::{IconCopy, IconEdit, IconMoreHorizontal, IconTrash};
use crate::ui::java_viewer::JavaSerializedViewer;
use crate::ui::json_viewer::{is_json_content, JsonViewer};
use crate::ui::pagination::LargeKeyWarning;
use crate::ui::{copy_text_to_clipboard, ServerInfoPanel, ToastManager};
use dioxus::prelude::*;
use serde_json;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

const PAGE_SIZE: usize = 100;

const LARGE_KEY_THRESHOLD: usize = 1000;
const STATUS_SUCCESS_BG: &str = COLOR_SUCCESS_BG;
const STATUS_ERROR_BG: &str = COLOR_ERROR_BG;
const ROW_CREATE_BG: &str = COLOR_ROW_CREATE_BG;
const ROW_EDIT_BG: &str = COLOR_ROW_EDIT_BG;


#[derive(Clone, Copy, PartialEq, Default)]
pub enum BinaryFormat {
    #[default]
    Hex,
    Base64,
    Image,
    Protobuf,
    JavaSerialized,
    Php,
    MsgPack,
    Pickle,
    Kryo,
    Bitmap,
    Bson,
    Cbor,
}

#[derive(Clone, PartialEq)]
struct HashDeleteTarget {
    field: String,
}
#[component]
pub fn ValueViewer(
    connection_pool: ConnectionPool,
    connection_version: u32,
    selected_key: Signal<String>,
    on_refresh: EventHandler<()>,
) -> Element {
    let key_info = use_signal(|| None::<KeyInfo>);
    let mut string_value = use_signal(String::new);
    let hash_value = use_signal(HashMap::new);
    let list_value = use_signal(Vec::new);
    let set_value = use_signal(Vec::new);
    let zset_value = use_signal(Vec::new);
    let stream_value = use_signal(Vec::new);
    let loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut is_binary = use_signal(|| false);
    let mut binary_format = use_signal(BinaryFormat::default);
    let mut serialization_data = use_signal(|| None::<(SerializationFormat, Vec<u8>)>);
    let binary_bytes = use_signal(Vec::<u8>::new);

    let mut hash_search = use_signal(String::new);
    let mut hash_status_message = use_signal(String::new);
    let mut hash_status_error = use_signal(|| false);
    let mut editing_hash_field = use_signal(|| None::<String>);
    let mut editing_hash_key = use_signal(String::new);
    let mut editing_hash_value = use_signal(String::new);
    let mut creating_hash_row = use_signal(|| false);
    let mut new_hash_key = use_signal(String::new);
    let mut new_hash_value = use_signal(String::new);
    let mut deleting_hash_field = use_signal(|| None::<HashDeleteTarget>);
    let deleting_hash_field_exiting = use_signal(|| false);
    let mut hash_action = use_signal(|| None::<String>);

    let mut list_status_message = use_signal(String::new);
    let mut list_status_error = use_signal(|| false);
    let mut new_list_value = use_signal(String::new);
    let mut list_action = use_signal(|| None::<String>);
    let mut editing_list_index = use_signal(|| None::<usize>);
    let mut editing_list_value = use_signal(String::new);

    let mut set_status_message = use_signal(String::new);
    let mut set_status_error = use_signal(|| false);
    let mut new_set_member = use_signal(String::new);
    let mut set_action = use_signal(|| None::<String>);
    let mut set_search = use_signal(String::new);
    let mut deleting_set_member = use_signal(|| None::<String>);
    let mut editing_set_member = use_signal(|| None::<String>);
    let mut editing_set_member_value = use_signal(String::new);

    let mut zset_status_message = use_signal(String::new);
    let mut zset_status_error = use_signal(|| false);
    let mut new_zset_member = use_signal(String::new);
    let mut new_zset_score = use_signal(String::new);
    let mut zset_action = use_signal(|| None::<String>);
    let mut zset_search = use_signal(String::new);
    let mut deleting_zset_member = use_signal(|| None::<String>);
    let mut editing_zset_member = use_signal(|| None::<String>);
    let mut editing_zset_score = use_signal(String::new);

    let mut list_page = use_signal(|| 0usize);
    let mut list_total = use_signal(|| 0usize);
    let _list_cursor = use_signal(|| 0u64);
    let list_has_more = use_signal(|| false);
    let list_loading_more = use_signal(|| false);
    let mut set_page = use_signal(|| 0usize);
    let mut set_total = use_signal(|| 0usize);
    let set_cursor = use_signal(|| 0u64);
    let set_has_more = use_signal(|| false);
    let set_loading_more = use_signal(|| false);
    let mut zset_page = use_signal(|| 0usize);
    let mut zset_total = use_signal(|| 0usize);
    let zset_cursor = use_signal(|| 0u64);
    let zset_has_more = use_signal(|| false);
    let zset_loading_more = use_signal(|| false);
    let hash_cursor = use_signal(|| 0u64);
    let hash_total = use_signal(|| 0usize);
    let hash_has_more = use_signal(|| false);
    let hash_loading_more = use_signal(|| false);
    let mut show_large_key_warning = use_signal(|| false);
    let mut memory_usage = use_signal(|| None::<u64>);
    let mut ttl_input = use_signal(String::new);
    let mut toast_manager = use_context::<Signal<ToastManager>>();
    let mut ttl_editing = use_signal(|| false);
    let mut ttl_processing = use_signal(|| false);
    let mut header_menu = use_signal(|| None::<ContextMenuState<()>>);
    let mut delete_key_confirm = use_signal(|| false);
    let mut delete_key_processing = use_signal(|| false);

    let mut bitmap_info = use_signal(|| None::<crate::redis::BitmapInfo>);
    let _bitmap_editing_offset = use_signal(String::new);
    let _bitmap_editing_value = use_signal(String::new);

    let mut stream_status_message = use_signal(String::new);
    let mut stream_status_error = use_signal(|| false);
    let mut stream_search = use_signal(String::new);
    let mut deleting_stream_entry = use_signal(|| None::<String>);
    let deleting_stream_entry_exiting = use_signal(|| false);

    let pool = connection_pool.clone();
    let pool_for_edit = connection_pool.clone();
    let pool_for_reload = connection_pool.clone();
    let pool_for_meta = connection_pool.clone();

    use_effect(move || {
        let key = selected_key.read().clone();

        hash_search.set(String::new());
        hash_status_message.set(String::new());
        hash_status_error.set(false);
        editing_hash_field.set(None);
        editing_hash_key.set(String::new());
        editing_hash_value.set(String::new());
        creating_hash_row.set(false);
        new_hash_key.set(String::new());
        new_hash_value.set(String::new());
        deleting_hash_field.set(None);
        hash_action.set(None);
        is_binary.set(false);
        serialization_data.set(None);
        binary_format.set(BinaryFormat::default());

        list_status_message.set(String::new());
        list_status_error.set(false);
        new_list_value.set(String::new());
        list_action.set(None);
        editing_list_index.set(None);
        editing_list_value.set(String::new());
        list_page.set(0);
        list_total.set(0);

        set_status_message.set(String::new());
        set_status_error.set(false);
        new_set_member.set(String::new());
        set_action.set(None);
        set_search.set(String::new());
        deleting_set_member.set(None);
        editing_set_member.set(None);
        editing_set_member_value.set(String::new());
        set_page.set(0);
        set_total.set(0);

        zset_status_message.set(String::new());
        zset_status_error.set(false);
        new_zset_member.set(String::new());
        new_zset_score.set(String::new());
        zset_action.set(None);
        zset_search.set(String::new());
        deleting_zset_member.set(None);
        editing_zset_member.set(None);
        editing_zset_score.set(String::new());
        zset_page.set(0);
        zset_total.set(0);

        show_large_key_warning.set(false);
        memory_usage.set(None);
        ttl_input.set(String::new());
        ttl_editing.set(false);
        ttl_processing.set(false);
        header_menu.set(None);
        delete_key_confirm.set(false);
        delete_key_processing.set(false);

        let pool = pool.clone();

        spawn(async move {
            tracing::info!("Loading key: {}", key);

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
                tracing::error!("{error}");
                hash_status_message.set(error);
                hash_status_error.set(true);
            }
        });
    });

    use_effect(move || {
        let key = selected_key.read().clone();
        let format = binary_format();

        if key.is_empty() || !is_binary() {
            return;
        }

        let pool = pool_for_reload.clone();

        spawn(async move {
            match pool.get_string_bytes(&key).await {
                Ok(bytes) => string_value.set(format_bytes(&bytes, format)),
                Err(error) => tracing::error!("Failed to reload binary string bytes: {}", error),
            }
        });
    });

    use_effect(move || {
        if let Some(info) = key_info() {
            let key = info.name.clone();
            ttl_input.set(
                info.ttl
                    .map(|ttl| ttl.to_string())
                    .unwrap_or_else(|| "-1".to_string()),
            );
            ttl_editing.set(false);

            let pool = pool_for_meta.clone();
            spawn(async move {
                match pool.memory_usage(&key).await {
                    Ok(usage) => memory_usage.set(usage),
                    Err(error) => {
                        tracing::error!("Failed to load memory usage: {}", error);
                        memory_usage.set(None);
                    }
                }
            });
        } else {
            ttl_input.set(String::new());
            ttl_editing.set(false);
            memory_usage.set(None);
        }
    });

    let key_for_edit = selected_key;

    let info = key_info();
    let is_loading = loading();
    let str_val = string_value();
    let hash_val = hash_value();
    let list_val = list_value();
    let set_val = set_value();
    let zset_val = zset_value();
    let stream_val = stream_value();
    let display_key = selected_key.read().clone();

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
                        flex: "1",
                        height: "100%",
                        background: COLOR_BG,
                        display: "flex",
                        flex_direction: "column",

                        if !display_key.is_empty() {
                            div {
                                padding: "16px 18px 12px",
                                border_bottom: "1px solid {COLOR_BORDER}",
                                background: COLOR_BG,

                                if let Some(ref info) = info {
                                    {
                                        let value_metric = value_metric_label(
                                            &info.key_type,
                                            &str_val,
                                            &hash_val,
                                            &list_val,
                                            &set_val,
                                            &zset_val,
                                            &stream_val,
                                        );
                                        let ttl_badge = format_ttl_label(info.ttl);
                                        let ttl_reset_value = info
                                            .ttl
                                            .map(|ttl| ttl.to_string())
                                            .unwrap_or_else(|| "-1".to_string());
                                        let memory_badge = format_memory_usage(memory_usage());

                                        rsx! {
                                            div {
                                                display: "flex",
                                                flex_direction: "column",
                                                gap: "10px",

                                                div {
                                                    display: "flex",
                                                    justify_content: "space_between",
                                                    align_items: "center",
                                                    gap: "12px",

                                                    div {
                                                        flex: "1",
                                                        min_width: "0",

                                                        div {
                                                            color: COLOR_ACCENT,
                                                            font_size: "16px",
                                                            font_weight: "700",
                                                            font_family: "Consolas, 'Courier New', monospace",
                                                            white_space: "nowrap",
                                                            overflow: "hidden",
                                                            text_overflow: "ellipsis",
                                                            title: "{display_key}",

                                                            "{display_key}"
                                                        }
                                                    }

                                                    div {
                                                        display: "flex",
                                                        align_items: "center",
                                                        gap: "6px",
                                                        flex_shrink: "0",

                                                        button {
                                                            width: "28px",
                                                            height: "28px",
                                                            background: COLOR_BG_TERTIARY,
                                                            border: "1px solid {COLOR_BORDER}",
                                                            border_radius: "6px",
                                                            cursor: "pointer",
                                                            display: "flex",
                                                            align_items: "center",
                                                            justify_content: "center",
                                                            color: COLOR_TEXT_SECONDARY,
                                                            title: "复制路径",
                                                            aria_label: "复制路径",
                                                            onclick: {
                                                                let key = display_key.clone();
                                                                move |_| match copy_value_to_clipboard(&key) {
                                                                    Ok(_) => {
                                                                        toast_manager.write().success("Key 路径已复制");
                                                                    }
                                                                    Err(error) => {
                                                                        toast_manager.write().error(&format!("复制失败：{error}"));
                                                                    }
                                                                }
                                                            },

                                                            IconCopy { size: Some(14) }
                                                        }

                                                        button {
                                                            width: "28px",
                                                            height: "28px",
                                                            background: COLOR_BG_TERTIARY,
                                                            border: "1px solid {COLOR_BORDER}",
                                                            border_radius: "6px",
                                                            cursor: "pointer",
                                                            display: "flex",
                                                            align_items: "center",
                                                            justify_content: "center",
                                                            color: COLOR_TEXT_SECONDARY,
                                                            title: "更多操作",
                                                            aria_label: "更多操作",
                                                            onclick: move |event| {
                                                                let coords = event.client_coordinates();
                                                                header_menu.set(Some(ContextMenuState::new(
                                                                    (),
                                                                    coords.x as i32,
                                                                    coords.y as i32,
                                                                )));
                                                            },

                                                            IconMoreHorizontal { size: Some(14) }
                                                        }
                                                    }
                                                }

                                                div {
                                                    display: "flex",
                                                    align_items: "center",
                                                    gap: "8px",
                                                    flex_wrap: "wrap",

                                                    span {
                                                        padding: "0 10px",
                                                        height: "22px",
                                                        border_radius: "999px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        color: COLOR_PRIMARY,
                                                        font_size: "11px",
                                                        font_weight: "700",
                                                        display: "inline-flex",
                                                        align_items: "center",
                                                        text_transform: "uppercase",
                                                        letter_spacing: "0.08em",

                                                        "{info.key_type}"
                                                    }

                                                    span {
                                                        padding: "0 10px",
                                                        height: "22px",
                                                        border_radius: "999px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "11px",
                                                        display: "inline-flex",
                                                        align_items: "center",

                                                        "{value_metric}"
                                                    }

                                                    span {
                                                        padding: "0 10px",
                                                        height: "22px",
                                                        border_radius: "999px",
                                                        background: COLOR_BG_TERTIARY,
                                                        border: "1px solid {COLOR_BORDER}",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "11px",
                                                        display: "inline-flex",
                                                        align_items: "center",

                                                        "内存 {memory_badge}"
                                                    }

                                                    if ttl_editing() {
                                                        div {
                                                            display: "flex",
                                                            align_items: "center",
                                                            gap: "6px",

                                                            input {
                                                                width: "80px",
                                                                min_width: "80px",
                                                                height: "26px",
                                                                box_sizing: "border-box",
                                                                padding: "0 8px",
                                                                background: COLOR_BG_TERTIARY,
                                                                border: "1px solid {COLOR_BORDER}",
                                                                border_radius: "6px",
                                                                color: COLOR_TEXT,
                                                                font_size: "11px",
                                                                font_family: "Consolas, 'Courier New', monospace",
                                                                text_align: "center",
                                                                r#type: "text",
                                                                value: "{ttl_input}",
                                                                placeholder: "秒",
                                                                oninput: move |event| ttl_input.set(event.value()),
                                                            }

                                                            button {
                                                                width: "26px",
                                                                height: "26px",
                                                                background: COLOR_PRIMARY,
                                                                color: COLOR_TEXT_CONTRAST,
                                                                border: "none",
                                                                border_radius: "6px",
                                                                cursor: "pointer",
                                                                font_size: "11px",
                                                                disabled: ttl_processing(),
                                                                title: "应用 TTL",
                                                                onclick: {
                                                                    let pool = connection_pool.clone();
                                                                    let key = display_key.clone();
                                                                    move |_| {
                                                                        let ttl_text = ttl_input().trim().to_string();
                                                                        if ttl_text.is_empty() {
                                                                            toast_manager.write().error("请输入 TTL");
                                                                            return;
                                                                        }

                                                                        let ttl = match ttl_text.parse::<i64>() {
                                                                            Ok(ttl) if ttl > 0 || ttl == -1 => ttl,
                                                                            _ => {
                                                                                toast_manager.write().error("TTL 必须大于 0 或 -1 表示永久");
                                                                                return;
                                                                            }
                                                                        };

                                                                        let pool = pool.clone();
                                                                        let key = key.clone();
                                                                        spawn(async move {
                                                                            ttl_processing.set(true);

                                                                            if ttl == -1 {
                                                                                match pool.remove_ttl(&key).await {
                                                                                    Ok(_) => {
                                                                                        toast_manager.write().success("已设为永久");
                                                                                        ttl_editing.set(false);
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
                                                                                        ).await {
                                                                                            tracing::error!("{error}");
                                                                                        } else {
                                                                                            on_refresh.call(());
                                                                                        }
                                                                                    }
                                                                                    Err(error) => {
                                                                                        toast_manager.write().error(&format!("设置失败：{error}"));
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                match pool.set_ttl(&key, ttl).await {
                                                                                    Ok(_) => {
                                                                                        toast_manager.write().success("TTL 已更新");
                                                                                        ttl_editing.set(false);
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
                                                                                        ).await {
                                                                                            tracing::error!("{error}");
                                                                                        } else {
                                                                                            on_refresh.call(());
                                                                                        }
                                                                                    }
                                                                                    Err(error) => {
                                                                                        toast_manager.write().error(&format!("TTL 更新失败：{error}"));
                                                                                    }
                                                                                }
                                                                            }

                                                                            ttl_processing.set(false);
                                                                        });
                                                                    }
                                                                },

                                                                if ttl_processing() { "…" } else { "✓" }
                                                            }

                                                            button {
                                                                width: "26px",
                                                                height: "26px",
                                                                background: COLOR_BG_TERTIARY,
                                                                color: COLOR_TEXT_SECONDARY,
                                                                border: "1px solid {COLOR_BORDER}",
                                                                border_radius: "6px",
                                                                cursor: "pointer",
                                                                font_size: "11px",
                                                                title: "取消 TTL 编辑",
                                                                onclick: {
                                                                    let ttl_reset_value = ttl_reset_value.clone();
                                                                    move |_| {
                                                                        ttl_input.set(ttl_reset_value.clone());
                                                                        ttl_editing.set(false);
                                                                    }
                                                                },

                                                                "×"
                                                            }
                                                        }
                                                    } else {
                                                        button {
                                                            padding: "0 10px",
                                                            height: "22px",
                                                            background: COLOR_BG_TERTIARY,
                                                            border: "1px solid {COLOR_BORDER}",
                                                            border_radius: "999px",
                                                            color: COLOR_TEXT_SECONDARY,
                                                            cursor: "pointer",
                                                            font_size: "11px",
                                                            display: "inline-flex",
                                                            align_items: "center",
                                                            onclick: move |_| ttl_editing.set(true),

                                                            "TTL {ttl_badge}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    span {
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "13px",

                                        "选择一个 Key 以查看和编辑详情"
                                    }
                                }
                            }
                        }

                        div {
                            flex: "1",
                            min_height: "0",
                            overflow: "hidden",
                            padding: "18px",
                            display: "flex",
                            flex_direction: "column",

                            if is_loading {
                                div {
                                    flex: "1",
                                    display: "flex",
                                    align_items: "center",
                                    justify_content: "center",
                                    color: COLOR_TEXT_SECONDARY,
                                    text_align: "center",
                                    border: "1px solid {COLOR_BORDER}",
                                    border_radius: "12px",
                                    background: COLOR_BG_SECONDARY,

                                    "正在加载 Key 内容..."
                                }
                            } else if display_key.is_empty() {
                                ServerInfoPanel {
                                    connection_pool: connection_pool.clone(),
                                    connection_version: connection_version,
                                    auto_refresh_interval: 0,
                                }
                            } else if let Some(info) = info.clone() {
                                {
                                    rsx! {
                                    div {
                                        flex: "1",
                                        min_height: "0",
                                        overflow: "hidden",
                                        display: "flex",
                                        flex_wrap: "wrap",
                                        gap: "18px",
                                        align_items: "stretch",

                                        div {
                                            flex: "1 1 640px",
                                            min_width: "320px",
                                            max_height: "100%",
                                            display: "flex",
                                            flex_direction: "column",
                                            gap: "14px",
            overflow: "hidden",

                                            div {
                                                flex: "1",
                                                min_height: "0",
                                                overflow: "hidden",
                                                display: "flex",
                                                flex_direction: "column",
                                                padding: "16px",
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "12px",
                                                background: COLOR_BG_SECONDARY,

            match info.key_type {
                                            KeyType::String => {
                                                let is_json = !is_binary() && is_json_content(&str_val);
                                                let serialization_info = serialization_data();
                                                let detected_format = serialization_info.as_ref().map(|(f, _)| *f);
                                                let is_serialized = serialization_info.is_some();

                                                rsx! {
                                                    div {
                                                        flex: "1",
                                                        min_height: "0",
                                                        display: "flex",
                                                        flex_direction: "column",
    if is_binary() {
                                                            div {
                                                                display: "flex",
                                                                gap: "8px",
                                                                align_items: "center",
                                                                margin_bottom: "12px",
                                                                flex_wrap: "wrap",

                                                                if is_serialized {
                                                                    span {
                                                                        color: COLOR_SUCCESS,
                                                                        font_size: "12px",

                                                                        match detected_format {
                                                                            Some(SerializationFormat::Java) => "Java 序列化对象",
                                                                            Some(SerializationFormat::Php) => "PHP 序列化数据",
                                                                            Some(SerializationFormat::MsgPack) => "MessagePack 数据",
                                                                            Some(SerializationFormat::Pickle) => "Python Pickle 数据",
                                                                            Some(SerializationFormat::Kryo) => "Kryo 序列化数据",
                                                                            Some(SerializationFormat::Fst) => "FST 序列化数据",
                                                                            Some(SerializationFormat::Bson) => "BSON 数据",
                                                                            Some(SerializationFormat::Cbor) => "CBOR 数据",
                                                                            Some(SerializationFormat::Protobuf) => "Protobuf 数据",
                                                                            _ => "序列化数据",
                                                                        }
                                                                    }
                                                                } else {
                                                                    span {
                                                                        color: COLOR_WARNING,
                                                                        font_size: "12px",

                                                                        "二进制数据"
                                                                    }
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Hex { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Hex { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Hex),

                                                                    "Hex"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Base64 { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Base64 { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: "none",
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Base64),

                                                                    "Base64"
                                                                }

                                                                {
                                                                    let bytes = binary_bytes();
                                                                    let is_image = detect_image_format(&bytes).is_some();
                                                                    rsx! {
                                                                        button {
                                                                            padding: "4px 8px",
                                                                            background: if binary_format() == BinaryFormat::Image { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                            color: if binary_format() == BinaryFormat::Image { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                            border: if is_image { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                            border_radius: "4px",
                                                                            cursor: "pointer",
                                                                            font_size: "12px",
                                                                            opacity: if is_image { "1.0" } else { "0.6" },
                                                                            onclick: move |_| binary_format.set(BinaryFormat::Image),

                                                                            "图片"
                                                                        }
                                                                    }
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::JavaSerialized { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::JavaSerialized { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if detected_format == Some(SerializationFormat::Java) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if detected_format == Some(SerializationFormat::Java) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::JavaSerialized),

                                                                    "Java"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Php { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Php { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if detected_format == Some(SerializationFormat::Php) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if detected_format == Some(SerializationFormat::Php) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Php),

                                                                    "PHP"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::MsgPack { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::MsgPack { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if detected_format == Some(SerializationFormat::MsgPack) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if detected_format == Some(SerializationFormat::MsgPack) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::MsgPack),

                                                                    "MsgPack"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Pickle { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Pickle { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if detected_format == Some(SerializationFormat::Pickle) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if detected_format == Some(SerializationFormat::Pickle) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Pickle),

                                                                    "Pickle"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Kryo { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Kryo { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if matches!(detected_format, Some(SerializationFormat::Kryo) | Some(SerializationFormat::Fst)) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if matches!(detected_format, Some(SerializationFormat::Kryo) | Some(SerializationFormat::Fst)) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Kryo),

                                                                    "Kryo"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Bson { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Bson { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if detected_format == Some(SerializationFormat::Bson) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if detected_format == Some(SerializationFormat::Bson) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Bson),

                                                                    "BSON"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Cbor { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Cbor { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if detected_format == Some(SerializationFormat::Cbor) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if detected_format == Some(SerializationFormat::Cbor) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Cbor),

                                                                    "CBOR"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Protobuf { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Protobuf { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if detected_format == Some(SerializationFormat::Protobuf) { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if detected_format == Some(SerializationFormat::Protobuf) { "1.0" } else { "0.6" },
                                                                    onclick: move |_| binary_format.set(BinaryFormat::Protobuf),

                                                                    "Protobuf"
                                                                }

                                                                button {
                                                                    padding: "4px 8px",
                                                                    background: if binary_format() == BinaryFormat::Bitmap { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                                    color: if binary_format() == BinaryFormat::Bitmap { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                                    border: if !is_serialized { "none" } else { format!("1px dashed {}", COLOR_BORDER) },
                                                                    border_radius: "4px",
                                                                    cursor: "pointer",
                                                                    font_size: "12px",
                                                                    opacity: if !is_serialized { "1.0" } else { "0.6" },
                                                                    onclick: {
                                                                        let pool = connection_pool.clone();
                                                                        let key = display_key.clone();
                                                                        move |_| {
                                                                            let pool = pool.clone();
                                                                            let key = key.clone();
                                                                            spawn(async move {
                                                                                match pool.get_bitmap_info(&key).await {
                                                                                    Ok(info) => {
                                                                                        bitmap_info.set(Some(info));
                                                                                        binary_format.set(BinaryFormat::Bitmap);
                                                                                    }
                                                                                    Err(e) => {
                                                                                        toast_manager.write().error(&format!("加载 Bitmap 失败: {}", e));
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    },

                                                                    "Bitmap"
                                                                }

                                                                button {
                                                                    style: "{secondary_action_button_style()}",
                                                                    title: "复制",
                                                                    onclick: move |_| {
                                                                        let current_format = binary_format();
                                                                        let serial_info = serialization_data();
                                                                        let current_str = string_value();
                                                                        let copy_text = match current_format {
                                                                            BinaryFormat::JavaSerialized
                                                                            | BinaryFormat::Php
                                                                            | BinaryFormat::MsgPack
                                                                            | BinaryFormat::Pickle
                                                                            | BinaryFormat::Kryo
                                                                            | BinaryFormat::Protobuf
                                                                            | BinaryFormat::Bson
                                                                            | BinaryFormat::Cbor => {
                                                                                if let Some((fmt, data)) = serial_info.as_ref() {
                                                                                    parse_to_json(data, *fmt).unwrap_or(current_str)
                                                                                } else {
                                                                                    current_str
                                                                                }
                                                                            }
                                                                            _ => current_str,
                                                                        };
                                                                        match copy_text_to_clipboard(&copy_text) {
                                                                            Ok(_) => {
                                                                                toast_manager.write().success("复制成功");
                                                                            }
                                                                            Err(e) => {
                                                                                toast_manager.write().error(&format!("复制失败：{}", e));
                                                                            }
                                                                        }
                                                                    },

                                                                    IconCopy { size: Some(14) }
                                                                    "复制"
                                                                }
                                                            }
                                                        }

                                                        if is_binary() {
                                                            match binary_format() {
                                                                BinaryFormat::JavaSerialized => {
                                                                    if let Some((SerializationFormat::Java, ref data)) = serialization_info {
                                                                        rsx! {
                                                                            JavaSerializedViewer {
                                                                                data: data.clone(),
                                                                            }
                                                                        }
                                                                    } else {
                                                                        rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,

                                                                                "解析失败"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                BinaryFormat::Php => {
                                                                    if let Some((SerializationFormat::Php, ref data)) = serialization_info {
                                                                        match parse_to_json(data, SerializationFormat::Php) {
                                                                            Ok(json_str) => rsx! {
                                                                                JsonViewer {
                                                                                    value: json_str,
                                                                                    editable: false,
                                                                                    on_change: move |_| {},
                                                                                }
                                                                            },
                                                                            Err(e) => rsx! {
                                                                                div {
                                                                                    padding: "16px",
                                                                                    background: COLOR_ERROR_BG,
                                                                                    border_radius: "8px",
                                                                                    color: COLOR_ERROR,

                                                                                    "PHP 解析错误: {e}"
                                                                                }
                                                                            },
                                                                        }
                                                                    } else {
                                                                        rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,

                                                                                "解析失败"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                BinaryFormat::MsgPack => {
                                                                    if let Some((SerializationFormat::MsgPack, ref data)) = serialization_info {
                                                                        match parse_to_json(data, SerializationFormat::MsgPack) {
                                                                            Ok(json_str) => rsx! {
                                                                                JsonViewer {
                                                                                    value: json_str,
                                                                                    editable: false,
                                                                                    on_change: move |_| {},
                                                                                }
                                                                            },
                                                                            Err(e) => rsx! {
                                                                                div {
                                                                                    padding: "16px",
                                                                                    background: COLOR_ERROR_BG,
                                                                                    border_radius: "8px",
                                                                                    color: COLOR_ERROR,

                                                                                    "MsgPack 解析错误: {e}"
                                                                                }
                                                                            },
                                                                        }
                                                                    } else {
                                                                        rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,

                                                                                "解析失败"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                BinaryFormat::Pickle => {
                                                                    if let Some((SerializationFormat::Pickle, ref data)) = serialization_info {
                                                                        match parse_to_json(data, SerializationFormat::Pickle) {
                                                                            Ok(json_str) => rsx! {
                                                                                JsonViewer {
                                                                                    value: json_str,
                                                                                    editable: false,
                                                                                    on_change: move |_| {},
                                                                                }
                                                                            },
                                                                            Err(e) => rsx! {
                                                                                div {
                                                                                    padding: "16px",
                                                                                    background: COLOR_ERROR_BG,
                                                                                    border_radius: "8px",
                                                                                    color: COLOR_ERROR,

                                                                                    "Pickle 解析错误: {e}"
                                                                                }
                                                                            },
                                                                        }
                                                                    } else {
                                                                        rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,

                                                                                "解析失败"
                                                                            }
                                                                        }
                                                                    }
                                                                }
    BinaryFormat::Kryo => {
                                                                     if let Some((ref format, ref data)) = serialization_info {
                                                                         if matches!(format, SerializationFormat::Kryo | SerializationFormat::Fst) {
                                                                             match parse_to_json(data, *format) {
                                                                                 Ok(json_str) => rsx! {
                                                                                     JsonViewer {
                                                                                         value: json_str,
                                                                                         editable: false,
                                                                                         on_change: move |_| {},
                                                                                     }
                                                                                 },
                                                                                 Err(e) => rsx! {
                                                                                     div {
                                                                                         padding: "16px",
                                                                                         background: COLOR_ERROR_BG,
                                                                                         border_radius: "8px",
                                                                                         color: COLOR_ERROR,

                                                                                         "Kryo/FST 解析错误: {e}"
                                                                                     }
                                                                                 },
                                                                             }
                                                                         } else {
                                                                             rsx! {
                                                                                 div {
                                                                                     padding: "16px",
                                                                                     background: COLOR_BG_TERTIARY,
                                                                                     border_radius: "8px",
                                                                                     color: COLOR_TEXT_SECONDARY,

                                                                                     "非 Kryo/FST 数据"
                                                                                 }
                                                                             }
                                                                         }
                                                                     } else {
                                                                         rsx! {
                                                                             div {
                                                                                 padding: "16px",
                                                                                 background: COLOR_BG_TERTIARY,
                                                                                 border_radius: "8px",
                                                                                 color: COLOR_TEXT_SECONDARY,

                                                                                 "解析失败"
                                                                             }
                                                                         }
                                                                     }
                                                                 }
                                                                 BinaryFormat::Bson => {
                                                                     if let Some((SerializationFormat::Bson, ref data)) = serialization_info {
                                                                         match parse_to_json(data, SerializationFormat::Bson) {
                                                                             Ok(json_str) => rsx! {
                                                                                 JsonViewer {
                                                                                     value: json_str,
                                                                                     editable: false,
                                                                                     on_change: move |_| {},
                                                                                 }
                                                                             },
                                                                             Err(e) => rsx! {
                                                                                 div {
                                                                                     padding: "16px",
                                                                                     background: COLOR_ERROR_BG,
                                                                                     border_radius: "8px",
                                                                                     color: COLOR_ERROR,

                                                                                     "BSON 解析错误: {e}"
                                                                                 }
                                                                             },
                                                                         }
                                                                     } else {
                                                                         rsx! {
                                                                             div {
                                                                                 padding: "16px",
                                                                                 background: COLOR_BG_TERTIARY,
                                                                                 border_radius: "8px",
                                                                                 color: COLOR_TEXT_SECONDARY,

                                                                                 "解析失败"
                                                                             }
                                                                         }
                                                                     }
                                                                 }
                                                                 BinaryFormat::Cbor => {
                                                                     if let Some((SerializationFormat::Cbor, ref data)) = serialization_info {
                                                                         match parse_to_json(data, SerializationFormat::Cbor) {
                                                                             Ok(json_str) => rsx! {
                                                                                 JsonViewer {
                                                                                     value: json_str,
                                                                                     editable: false,
                                                                                     on_change: move |_| {},
                                                                                 }
                                                                             },
                                                                             Err(e) => rsx! {
                                                                                 div {
                                                                                     padding: "16px",
                                                                                     background: COLOR_ERROR_BG,
                                                                                     border_radius: "8px",
                                                                                     color: COLOR_ERROR,

                                                                                     "CBOR 解析错误: {e}"
                                                                                 }
                                                                             },
                                                                         }
                                                                     } else {
                                                                         rsx! {
                                                                             div {
                                                                                 padding: "16px",
                                                                                 background: COLOR_BG_TERTIARY,
                                                                                 border_radius: "8px",
                                                                                 color: COLOR_TEXT_SECONDARY,

                                                                                 "解析失败"
                                                                             }
                                                                         }
                                                                     }
                                                                 }
                                                                 BinaryFormat::Image => {
                                                                    let bytes = binary_bytes();
                                                                    tracing::info!("Image preview: {} bytes, first 10: {:02x?}", bytes.len(), &bytes[..10.min(bytes.len())]);
                                                                    if let Some(format) = detect_image_format(&bytes) {
                                                                        use base64::{engine::general_purpose, Engine as _};
                                                                        let base64_data = general_purpose::STANDARD.encode(&bytes);
                                                                        let mime_type = match format {
                                                                            "PNG" => "image/png",
                                                                            "JPEG" => "image/jpeg",
                                                                            "GIF" => "image/gif",
                                                                            "WEBP" => "image/webp",
                                                                            "BMP" => "image/bmp",
                                                                            _ => "application/octet-stream",
                                                                        };
                                                                        let data_uri = format!("data:{};base64,{}", mime_type, base64_data);

                                                                        let temp_dir = std::env::temp_dir();
                                                                        let file_name = format!("redis_image_{}.{}", uuid::Uuid::new_v4(), format.to_lowercase());
                                                                        let file_path = temp_dir.join(&file_name);
                                                                        let file_path_clone = file_path.clone();
                                                                        let bytes_clone = bytes.clone();
                                                                        let file_size_formatted = format_memory_usage(Some(bytes.len() as u64));
                                                                        let format_str = format.to_string();
                                                                        let data_uri_for_preview = data_uri.clone();
                                                                        let format_for_preview = format_str.clone();
                                                                        let size_for_preview = file_size_formatted.clone();

                                                                        rsx! {
                                                                            div {
                                                                                display: "flex",
                                                                                flex_direction: "column",
                                                                                align_items: "center",
                                                                                gap: "12px",

                                                                                div {
                                                                                    padding: "12px",
                                                                                    background: COLOR_BG_TERTIARY,
                                                                                    border_radius: "8px",
                                                                                    color: COLOR_TEXT_SECONDARY,
                                                                                    font_size: "13px",

                                                                                    "{format} 图片 - {file_size_formatted}"
                                                                                }

                                                                                div {
                                                                                    max_width: "100%",
                                                                                    max_height: "500px",
                                                                                    overflow: "auto",
                                                                                    background: COLOR_BG_TERTIARY,
                                                                                    border_radius: "8px",
                                                                                    padding: "8px",

                                                                                    img {
                                                                                        src: "{data_uri}",
                                                                                        max_width: "100%",
                                                                                        max_height: "500px",
                                                                                        object_fit: "contain",
                                                                                        border_radius: "4px",
                                                                                        cursor: "pointer",
                                                                                        transition: "transform 0.2s, box-shadow 0.2s",

                                                                                        onclick: move |_| {
                                                                                            *PREVIEW_IMAGE.write() = Some(PreviewImageData {
                                                                                                data_uri: data_uri_for_preview.clone(),
                                                                                                format: format_for_preview.clone(),
                                                                                                size: size_for_preview.clone(),
                                                                                            });
                                                                                        },
                                                                                    }
                                                                                }

                                                                                button {
                                                                                    padding: "8px 16px",
                                                                                    background: COLOR_PRIMARY,
                                                                                    color: COLOR_TEXT_CONTRAST,
                                                                                    border: "none",
                                                                                    border_radius: "6px",
                                                                                    cursor: "pointer",
                                                                                    font_size: "13px",

                                                                                    onclick: move |_| {
                                                                                        let _ = std::fs::write(&file_path_clone, &bytes_clone);
                                                                                        let _ = open::that(&file_path_clone);
                                                                                    },

                                                                                    "用系统图片查看器打开"
                                                                                }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,

                                                                                "非图片数据"
                                                                            }
                                                                        }
                                                                    }
                                                                }
    BinaryFormat::Protobuf => {
                                                                    if let Some((SerializationFormat::Protobuf, ref data)) = serialization_info {
                                                                        rsx! {
                                                                            ProtobufViewer {
                                                                                data: data.clone(),
                                                                            }
                                                                        }
                                                                    } else {
                                                                        rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,

                                                                                "非 Protobuf 数据"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                BinaryFormat::Bitmap => {
                                                                    if let Some(ref info) = bitmap_info() {
                                                                        rsx! {
                                                                            BitmapViewer {
                                                                                info: info.clone(),
                                                                                pool: connection_pool.clone(),
                                                                                redis_key: display_key.clone(),
                                                                                on_update: {
                                                                                    let pool = connection_pool.clone();
                                                                                    let key = display_key.clone();
                                                                                    move || {
                                                                                        let pool = pool.clone();
                                                                                        let key = key.clone();
                                                                                        spawn(async move {
                                                                                            if let Ok(new_info) = pool.get_bitmap_info(&key).await {
                                                                                                bitmap_info.set(Some(new_info));
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                },
                                                                            }
                                                                        }
                                                                    } else {
                                                                        rsx! {
                                                                            div {
                                                                                padding: "16px",
                                                                                background: COLOR_BG_TERTIARY,
                                                                                border_radius: "8px",
                                                                                color: COLOR_TEXT_SECONDARY,

                                                                                "点击 \"Bitmap\" 按钮加载可视化数据"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                _ => {
                                                                    rsx! {
                                                                        EditableField {
                                                                            label: String::new(),
                                                                            value: str_val.clone(),
                                                                            multiline: true,
                                                                            editable: false,
                                                                            on_change: move |_| {},
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        } else if is_json {
                                                        JsonViewer {
                                                            value: str_val.clone(),
                                                            editable: true,
                                                            on_change: {
                                                                let pool = pool_for_edit.clone();
                                                                let key_sig = key_for_edit.clone();
                                                                move |new_val: String| {
                                                                    let pool = pool.clone();
                                                                    let key = key_sig.read().clone();
                                                                    let val = new_val.clone();
                                                                    spawn(async move {
                                                                        saving.set(true);
                                                                        if pool.set_string_value(&key, &val).await.is_ok() {
                                                                            string_value.set(val);
                                                                            on_refresh.call(());
                                                                        }
                                                                        saving.set(false);
                                                                    });
                                                                }
                                                            },
                                                        }
                                                    } else {
                                                        EditableField {
                                                            label: "Value".to_string(),
                                                            value: str_val.clone(),
                                                            editable: !is_binary(),
                                                            multiline: true,
                                                            on_change: {
                                                                let pool = pool_for_edit.clone();
                                                                let key_sig = key_for_edit.clone();
                                                                move |new_val: String| {
                                                                    let pool = pool.clone();
                                                                    let key = key_sig.read().clone();
                                                                    let val = new_val.clone();
                                                                    spawn(async move {
                                                                        saving.set(true);
                                                                        if pool.set_string_value(&key, &val).await.is_ok() {
                                                                            string_value.set(val);
                                                                            on_refresh.call(());
                                                                        }
                                                                        saving.set(false);
                                                                    });
                                                                }
                                                            },
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        KeyType::Hash => {
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
                                                                                ).await {
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
                                                                                ).await;
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
                                                                    ).await;
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
                                                                                            deleting_hash_field.set(Some(HashDeleteTarget { field: field.clone() }));
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

                                                                    if active_hash_action.as_deref() == Some(format!("delete:{}", target.field).as_str()) {
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
                                        KeyType::List => {
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
                                                                placeholder: "输入新元素值",
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
                                                                                list_status_message.set("值不能为空".to_string());
                                                                                list_status_error.set(true);
                                                                                return;
                                                                            }

                                                                            list_action.set(Some("push".to_string()));
                                                                            list_status_message.set(String::new());
                                                                            list_status_error.set(false);

                                                                            match pool.list_push(&key, &value, true).await {
                                                                                Ok(_) => {
                                                                                    new_list_value.set(String::new());
                                                                                    list_status_message.set("添加成功".to_string());
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
                                                                                    ).await {
                                                                                        tracing::error!("{error}");
                                                                                    } else {
                                                                                        on_refresh.call(());
                                                                                    }
                                                                                }
                                                                                Err(error) => {
                                                                                    list_status_message.set(format!("添加失败：{error}"));
                                                                                    list_status_error.set(true);
                                                                                }
                                                                            }
                                                                            list_action.set(None);
                                                                        });
                                                                    }
                                                                },

                                                                if list_action().as_deref() == Some("push") { "添加中..." } else { "LPUSH" }
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
                                                                                list_status_message.set("值不能为空".to_string());
                                                                                list_status_error.set(true);
                                                                                return;
                                                                            }

                                                                            list_action.set(Some("rpush".to_string()));
                                                                            match pool.list_push(&key, &value, false).await {
                                                                                Ok(_) => {
                                                                                    new_list_value.set(String::new());
                                                                                    list_status_message.set("添加成功".to_string());
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
                                                                                    ).await {
                                                                                        tracing::error!("{error}");
                                                                                    } else {
                                                                                        on_refresh.call(());
                                                                                    }
                                                                                }
                                                                                Err(error) => {
                                                                                    list_status_message.set(format!("添加失败：{error}"));
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
                                                            title: "复制",
                                                            onclick: {
                                                                let list = list_val.clone();
                                                                move |_| {
                                                                    let json = serde_json::to_string_pretty(&list).unwrap_or_default();
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
                                                                    ).await;
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

                                                                        "当前列表没有元素"
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
                                                                                                            list_status_message.set("修改成功".to_string());
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
                                                                                                             ).await {
                                                                                                                tracing::error!("{error}");
                                                                                                            } else {
                                                                                                                on_refresh.call(());
                                                                                                            }
                                                                                                        }
                                                                                                        Err(error) => {
                                                                                                            list_status_message.set(format!("修改失败：{error}"));
                                                                                                            list_status_error.set(true);
                                                                                                        }
                                                                                                    }
                                                                                                    list_action.set(None);
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
                                                                                            editing_list_index.set(None);
                                                                                        },

                                                                                        "取消"
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
                                                                                        title: "复制",
                                                                                        onclick: {
                                                                                            let value = value.clone();
                                                                                            move |_| {
                                                                                                match copy_value_to_clipboard(&value) {
                                                                                                    Ok(_) => {
                                                                                                        toast_manager.write().success("复制成功");
                                                                                                    }
                                                                                                    Err(error) => {
                                                                                                        toast_manager.write().error(&format!("复制失败：{error}"));
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        },

                                                                                        IconCopy { size: Some(15) }
                                                                                    }

                                                                                    button {
                                                                                        style: "{compact_icon_action_button_style(false, false)}",
                                                                                        title: "编辑",
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
                                                                                        title: "删除",
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
                                                                                                            list_status_message.set("删除成功".to_string());
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
                                                                                                             ).await {
                                                                                                                tracing::error!("{error}");
                                                                                                            } else {
                                                                                                                on_refresh.call(());
                                                                                                            }
                                                                                                        }
                                                                                                        Err(error) => {
                                                                                                            list_status_message.set(format!("删除失败：{error}"));
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
                                        KeyType::Set => {
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
                                                                                    tracing::error!(
                                                                                        "重新加载 set 数据失败: {}",
                                                                                        error
                                                                                    );
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
                                                                                                                Ok(_) => pool.set_add(&key, &new_member).await.map(|_| ()),
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
                                                                                                            toast_manager.write().error(&format!("复制失败：{error}"));
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
                                        KeyType::ZSet => {
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
                                                                            ).await {
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
                                                                            ).await;
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
                                                                                ).await {
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

                                                            if zset_has_more() && !zset_loading_more() && scroll_height - scroll_top - client_height < 200 {
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
                                                                    ).await;
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
                                                                                                        zset_status_message.set("Score 必须是有效数字".to_string());
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
                                                                                                         ).await {
                                                                                                            tracing::error!("{error}");
                                                                                                        } else {
                                                                                                            on_refresh.call(());
                                                                                                        }
                                                                                                    }
                                                                                                    Err(error) => {
                                                                                                        zset_status_message.set(format!("修改失败：{error}"));
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
                                                                                                    toast_manager.write().error(&format!("复制失败：{error}"));
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
                                                                                                zset_action.set(Some(format!("delete:{}", member)));
                                                                                                match pool.zset_remove(&key, &member).await {
                                                                                                    Ok(_) => {
                                                                                                        zset_status_message.set("删除成功".to_string());
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
                                                                                                         ).await {
                                                                                                            tracing::error!("{error}");
                                                                                                        } else {
                                                                                                            on_refresh.call(());
                                                                                                        }
                                                                                                    }
                                                                                                    Err(error) => {
                                                                                                        zset_status_message.set(format!("删除失败：{error}"));
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
                                        KeyType::Stream => {
                                            let stream_search_value = stream_search();
                                            let normalized_stream_search =
                                                stream_search_value.trim().to_lowercase();
                                            let filtered_stream_entries: Vec<(String, Vec<(String, String)>)> =
                                                stream_val
                                                    .iter()
                                                    .filter(|(entry_id, fields)| {
                                                        if normalized_stream_search.is_empty() {
                                                            true
                                                        } else {
                                                            entry_id
                                                                .to_lowercase()
                                                                .contains(&normalized_stream_search)
                                                                || fields.iter().any(|(field_key, field_value)| {
                                                                    field_key
                                                                        .to_lowercase()
                                                                        .contains(&normalized_stream_search)
                                                                        || field_value
                                                                            .to_lowercase()
                                                                            .contains(
                                                                                &normalized_stream_search,
                                                                            )
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
                                                                placeholder: "搜索 ID 或字段",
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
                                                            title: "复制",
                                                            onclick: {
                                                                let stream = stream_val.clone();
                                                                move |_| {
                                                                    let json = serde_json::to_string_pretty(&stream).unwrap_or_default();
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
                                                                                "暂无数据"
                                                                            } else {
                                                                                "未找到匹配的条目"
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
                                                                                    title: "删除",
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

                                                                    "确认删除"
                                                                }

                                                                p {
                                                                    style: "{overlay_modal_body_style()}",

                                                                    "确定要删除 entry \"{entry_id}\" 吗？"
                                                                }

                                                                div {
                                                                    style: "{overlay_modal_actions_style()}",

                                                                    button {
                                                                        style: "{secondary_action_button_style()}",
                                                                        onclick: {
                                                                            let deleting_stream_entry = deleting_stream_entry.clone();
                                                                            let mut deleting_stream_entry_exiting = deleting_stream_entry_exiting.clone();
                                                                            move |_| {
                                                                                deleting_stream_entry_exiting.set(true);
                                                                                let mut dse = deleting_stream_entry.clone();
                                                                                let mut dsee = deleting_stream_entry_exiting.clone();
                                                                                spawn(async move {
                                                                                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                                                                    dse.set(None);
                                                                                    dsee.set(false);
                                                                                });
                                                                            }
                                                                        },

                                                                        "取消"
                                                                    }

                                                                    button {
                                                                        style: "{destructive_action_button_style(false)}",
                                                                        onclick: {
                                                                            let pool = connection_pool.clone();
                                                                            let key = display_key.clone();
                                                                            let entry_id = entry_id.clone();
                                                                            let mut deleting_stream_entry = deleting_stream_entry.clone();
                                                                            let mut deleting_stream_entry_exiting = deleting_stream_entry_exiting.clone();
                                                                            move |_| {
                                                                                let pool = pool.clone();
                                                                                let key = key.clone();
                                                                                let entry_id = entry_id.clone();
                                                                                spawn(async move {
                                                                                    match pool.stream_delete(&key, &entry_id).await {
                                                                                        Ok(true) => {
                                                                                            stream_status_message.set("删除成功".to_string());
                                                                                            stream_status_error.set(false);
                                                                                            deleting_stream_entry_exiting.set(true);
                                                                                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
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
                                                                                            stream_status_message.set("Entry 不存在".to_string());
                                                                                            stream_status_error.set(true);
                                                                                        }
                                                                                        Err(error) => {
                                                                                            stream_status_message.set(format!("删除失败：{error}"));
                                                                                            stream_status_error.set(true);
                                                                                        }
                                                                                    }
                                                                                });
                                                                            }
                                                                        },

                                                                        "删除"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                                        rsx! {
                                                            div {
                                                                color: COLOR_TEXT_SECONDARY,

                                                                "暂不支持该类型的编辑"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                }
                            } else {
                                div {
                                    height: "100%",
                                    display: "flex",
                                    align_items: "center",
                                    justify_content: "center",
                                    color: COLOR_TEXT_SECONDARY,
                                    text_align: "center",
                                    border: "1px solid {COLOR_BORDER}",
                                    border_radius: "12px",
                                    background: COLOR_BG_SECONDARY,

                                    "未能加载 Key 数据"
                                }
                            }
                        }

                        if let Some(menu) = header_menu() {
                            {
                                let menu_id = menu.id;
                                let x = menu.x;
                                let y = menu.y;
                                let mut header_menu_for_close = header_menu.clone();

                                rsx! {
                                    ContextMenu {
                                        key: "{menu_id}",
                                        menu_id: menu_id,
                                        x: x,
                                        y: y,
                                        on_close: move |closing_menu_id| {
                                            if header_menu_for_close()
                                                .as_ref()
                                                .map(|menu| menu.id)
                                                == Some(closing_menu_id)
                                            {
                                                header_menu_for_close.set(None);
                                            }
                                        },

                                        ContextMenuItem {
                                            icon: Some(rsx! { IconEdit { size: Some(14) } }),
                                            label: if ttl_editing() {
                                                "收起 TTL 编辑".to_string()
                                            } else {
                                                "编辑 TTL".to_string()
                                            },
                                            danger: false,
                                            disabled: false,
                                            onclick: move |_| {
                                                header_menu.set(None);
                                                delete_key_confirm.set(false);
                                                ttl_editing.set(!ttl_editing());
                                            },
                                        }

                                        ContextMenuItem {
                                            icon: Some(rsx! { IconTrash { size: Some(14) } }),
                                            label: "删除 Key".to_string(),
                                            danger: true,
                                            disabled: false,
                                            onclick: move |_| {
                                                header_menu.set(None);
                                                ttl_editing.set(false);
                                                delete_key_confirm.set(true);
                                            },
                                        }
                                    }
                                }
                            }
                        }

                        if delete_key_confirm() {
                            div {
                                style: "{overlay_modal_backdrop_style(false)}",

                                style { "{overlay_modal_keyframes()}" }

                                div {
                                    style: "{overlay_modal_surface_style(\"420px\", false)}",

                                    h3 {
                                        style: "{overlay_modal_title_style()}",

                                        "确认删除"
                                    }

                                    p {
                                        style: "{overlay_modal_body_style()}",

                                        "确定删除当前 Key \"{display_key}\" 吗？此操作不可恢复。"
                                    }

                                    div {
                                        style: "{overlay_modal_actions_style()}",

                                        button {
                                            style: "{secondary_action_button_style()}",
                                            disabled: delete_key_processing(),
                                            onclick: move |_| delete_key_confirm.set(false),

                                            "取消"
                                        }

                                        button {
                                            style: "{destructive_action_button_style(delete_key_processing())}",
                                            disabled: delete_key_processing(),
                                            onclick: {
                                                let pool = connection_pool.clone();
                                                let key = display_key.clone();
                                                move |_| {
                                                    let pool = pool.clone();
                                                    let key = key.clone();
                                                    spawn(async move {
                                                        delete_key_processing.set(true);

                                                        match pool.delete_key(&key).await {
                                                            Ok(_) => {
                                                                delete_key_confirm.set(false);
                                                                toast_manager.write().success("Key 已删除");
                                                                selected_key.set(String::new());
                                                                on_refresh.call(());
                                                            }
                                                            Err(error) => {
                                                                toast_manager.write().error(&format!("删除失败：{error}"));
                                                            }
                                                        }

                                                        delete_key_processing.set(false);
                                                    });
                                                }
                                            },

                                            if delete_key_processing() { "删除中..." } else { "确认删除" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
}
