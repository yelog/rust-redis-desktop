mod bitmap_viewer;
mod data_loader;
mod formatters;
mod hash_panel;
mod image_preview;
mod list_panel;
mod protobuf_viewer;
mod set_panel;
mod stream_panel;
mod styles;
mod zset_panel;

use self::bitmap_viewer::BitmapViewer;
use self::formatters::{
    base64_decode, copy_value_to_clipboard, detect_image_format, format_bytes, format_memory_usage,
    format_ttl_label, value_metric_label,
};
use self::hash_panel::HashPanel;
pub use self::image_preview::{ImagePreview, PreviewImageData, PREVIEW_IMAGE};
use self::list_panel::ListPanel;
use self::protobuf_viewer::ProtobufViewer;
use self::set_panel::SetPanel;
use self::stream_panel::StreamPanel;
use self::styles::{
    destructive_action_button_style, image_preview_button_style, image_preview_info_chip_style,
    overlay_modal_actions_style, overlay_modal_backdrop_style, overlay_modal_body_style,
    overlay_modal_keyframes, overlay_modal_surface_style, overlay_modal_title_style,
    secondary_action_button_style,
};
use self::zset_panel::ZSetPanel;
use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::redis::{KeyInfo, KeyType};
use crate::serialization::{parse_to_json, SerializationFormat};
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_ERROR,
    COLOR_ERROR_BG, COLOR_PRIMARY, COLOR_ROW_CREATE_BG, COLOR_ROW_EDIT_BG, COLOR_SUCCESS,
    COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_WARNING,
};
use crate::ui::context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
use crate::ui::editable_field::EditableField;
use crate::ui::icons::{IconCopy, IconEdit, IconMoreHorizontal, IconTrash};
use crate::ui::java_viewer::JavaSerializedViewer;
use crate::ui::json_viewer::{is_json_content, JsonViewer};
use crate::ui::{copy_text_to_clipboard, ServerInfoPanel, ToastManager};
use dioxus::prelude::*;
use std::collections::HashMap;

const PAGE_SIZE: usize = 100;

const LARGE_KEY_THRESHOLD: usize = 1000;
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
    on_connection_error: EventHandler<()>,
    on_refresh: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
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

    let stream_status_message = use_signal(String::new);
    let stream_status_error = use_signal(|| false);
    let stream_search = use_signal(String::new);
    let deleting_stream_entry = use_signal(|| None::<String>);
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
                toast_manager.write().error(&error);
                hash_status_message.set(error);
                hash_status_error.set(true);
                on_connection_error.call(());
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
                                                            title: i18n.read().t("Copy path"),
                                                            aria_label: i18n.read().t("Copy path"),
                                                            onclick: {
                                                                let key = display_key.clone();
                                                                move |_| match copy_value_to_clipboard(&key) {
                                                                    Ok(_) => {
                                                                        toast_manager.write().success(&i18n.read().t("Key path copied"));
                                                                    }
                                                                    Err(error) => {
                                                                        toast_manager.write().error(&format!("{}{}", i18n.read().t("Copy failed: "), error));
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
                                                            title: i18n.read().t("More actions"),
                                                            aria_label: i18n.read().t("More actions"),
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

                                                        {format!("{} {}", i18n.read().t("Memory"), memory_badge)}
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
                                                                placeholder: i18n.read().t("Seconds"),
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
                                                                title: i18n.read().t("Apply TTL"),
                                                                onclick: {
                                                                    let pool = connection_pool.clone();
                                                                    let key = display_key.clone();
                                                                    move |_| {
                                                                        let ttl_text = ttl_input().trim().to_string();
                                                                        if ttl_text.is_empty() {
                                                                            toast_manager.write().error(&i18n.read().t("Please enter a TTL"));
                                                                            return;
                                                                        }

                                                                        let ttl = match ttl_text.parse::<i64>() {
                                                                            Ok(ttl) if ttl > 0 || ttl == -1 => ttl,
                                                                            _ => {
                                                                                toast_manager.write().error(&i18n.read().t("TTL must be greater than 0 or -1 for permanent"));
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
                                                                                        toast_manager.write().success(&i18n.read().t("Permanently persisted"));
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
                                                                                            toast_manager.write().error(&error);
                                                                                            on_connection_error.call(());
                                                                                        } else {
                                                                                            on_refresh.call(());
                                                                                        }
                                                                                    }
                                                                                    Err(error) => {
                                                                                        toast_manager.write().error(&format!("{}{}", i18n.read().t("Set failed: "), error));
                                                                                    }
                                                                                }
                                                                            } else {
                                                                                match pool.set_ttl(&key, ttl).await {
                                                                                    Ok(_) => {
                                                                                        toast_manager.write().success(&i18n.read().t("TTL updated"));
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
                                                                                            toast_manager.write().error(&error);
                                                                                            on_connection_error.call(());
                                                                                        } else {
                                                                                            on_refresh.call(());
                                                                                        }
                                                                                    }
                                                                                    Err(error) => {
                                                                                        toast_manager.write().error(&format!("{}{}", i18n.read().t("TTL update failed: "), error));
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
                                                                title: i18n.read().t("Cancel TTL edit"),
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

                                        {i18n.read().t("Select a key to view and edit details")}
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

                                    {i18n.read().t("Loading key content...")}
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
                                                                            Some(SerializationFormat::Java) => "Java serialized object",
                                                                            Some(SerializationFormat::Php) => "PHP serialized data",
                                                                            Some(SerializationFormat::MsgPack) => "MessagePack data",
                                                                            Some(SerializationFormat::Pickle) => "Python Pickle data",
                                                                            Some(SerializationFormat::Kryo) => "Kryo serialized data",
                                                                            Some(SerializationFormat::Fst) => "FST serialized data",
                                                                            Some(SerializationFormat::Bson) => "BSON data",
                                                                            Some(SerializationFormat::Cbor) => "CBOR data",
                                                                            Some(SerializationFormat::Protobuf) => "Protobuf data",
                                                                            _ => "Serialized data",
                                                                        }
                                                                    }
                                                                } else {
                                                                    span {
                                                                        color: COLOR_WARNING,
                                                                        font_size: "12px",

                                                                        {i18n.read().t("Binary data")}
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

                                                                            {i18n.read().t("Image")}
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
                                                                                        toast_manager.write().error(&format!("{}{}", i18n.read().t("Load bitmap failed: "), e));
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    },

                                                                    "Bitmap"
                                                                }

                                                                button {
                                                                    style: "{secondary_action_button_style()}",
                                                                    title: i18n.read().t("Copy"),
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
                                                                                toast_manager.write().success(&i18n.read().t("Copied"));
                                                                            }
                                                                            Err(e) => {
                                                                                toast_manager.write().error(&format!("{}{}", i18n.read().t("Copy failed: "), e));
                                                                            }
                                                                        }
                                                                    },

                                                                    IconCopy { size: Some(14) }
                                                                    {i18n.read().t("Copy")}
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

                                                                                {i18n.read().t("Parse failed")}
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

                                                                                    {format!("{}{}", i18n.read().t("PHP parse error: "), e)}
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

                                                                                {i18n.read().t("Parse failed")}
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

                                                                                    {format!("{}{}", i18n.read().t("MsgPack parse error: "), e)}
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

                                                                                {i18n.read().t("Parse failed")}
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

                                                                                    {format!("{}{}", i18n.read().t("Pickle parse error: "), e)}
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

                                                                                {i18n.read().t("Parse failed")}
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

                                                                                         {format!("{}{}", i18n.read().t("Kryo/FST parse error: "), e)}
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

                                                                                     {i18n.read().t("Not Kryo/FST data")}
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

                                                                                 {i18n.read().t("Parse failed")}
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

                                                                                     {format!("{}{}", i18n.read().t("BSON parse error: "), e)}
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

                                                                                 {i18n.read().t("Parse failed")}
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

                                                                                     {format!("{}{}", i18n.read().t("CBOR parse error: "), e)}
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

                                                                                 {i18n.read().t("Parse failed")}
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

                                                                                    {format!("{} {} - {}", format, i18n.read().t("Image"), file_size_formatted)}
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

                                                                                    {i18n.read().t("Open with system image viewer")}
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

                                                                                {i18n.read().t("Not image data")}
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

                                                                                {i18n.read().t("Not Protobuf data")}
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

                                                                                {i18n.read().t("Click the Bitmap button to load visualization data")}
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
                                                                            label: i18n.read().t("Value"),
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
                                        KeyType::Hash => rsx! {
                                            HashPanel {
                                                connection_pool: connection_pool.clone(),
                                                display_key: display_key.clone(),
                                                on_refresh: on_refresh.clone(),
                                                toast_manager,
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
                                                hash_loading_more,
                                                list_has_more,
                                                list_total,
                                                set_cursor,
                                                set_total,
                                                set_has_more,
                                                zset_cursor,
                                                zset_total,
                                                zset_has_more,
                                                hash_search,
                                                hash_status_message,
                                                hash_status_error,
                                                editing_hash_field,
                                                editing_hash_key,
                                                editing_hash_value,
                                                creating_hash_row,
                                                new_hash_key,
                                                new_hash_value,
                                                deleting_hash_field,
                                                deleting_hash_field_exiting,
                                                hash_action,
                                            }
                                        },
                                        KeyType::List => rsx! {
                                            ListPanel {
                                                connection_pool: connection_pool.clone(),
                                                display_key: display_key.clone(),
                                                on_refresh: on_refresh.clone(),
                                                toast_manager,
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
                                                list_loading_more,
                                                list_status_message,
                                                list_status_error,
                                                new_list_value,
                                                list_action,
                                                editing_list_index,
                                                editing_list_value,
                                            }
                                        },
                                        KeyType::Set => rsx! {
                                            SetPanel {
                                                connection_pool: connection_pool.clone(),
                                                display_key: display_key.clone(),
                                                on_refresh: on_refresh.clone(),
                                                toast_manager,
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
                                                set_loading_more,
                                                set_status_message,
                                                set_status_error,
                                                new_set_member,
                                                set_action,
                                                set_search,
                                                editing_set_member,
                                                editing_set_member_value,
                                            }
                                        },
                                        KeyType::ZSet => rsx! {
                                            ZSetPanel {
                                                connection_pool: connection_pool.clone(),
                                                display_key: display_key.clone(),
                                                on_refresh: on_refresh.clone(),
                                                toast_manager,
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
                                                zset_loading_more,
                                                zset_status_message,
                                                zset_status_error,
                                                new_zset_member,
                                                new_zset_score,
                                                zset_action,
                                                zset_search,
                                                deleting_zset_member,
                                                editing_zset_member,
                                                editing_zset_score,
                                            }
                                        },
                                        KeyType::Stream => rsx! {
                                            StreamPanel {
                                                connection_pool: connection_pool.clone(),
                                                display_key: display_key.clone(),
                                                on_refresh: on_refresh.clone(),
                                                toast_manager,
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
                                                stream_status_message,
                                                stream_status_error,
                                                stream_search,
                                                deleting_stream_entry,
                                                deleting_stream_entry_exiting,
                                            }
                                        },
                                        _ => {
                                                        rsx! {
                                                            div {
                                                                color: COLOR_TEXT_SECONDARY,

                                                                {i18n.read().t("Editing this type is not supported yet")}
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

                                    {i18n.read().t("Failed to load key data")}
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
                                                i18n.read().t("Collapse TTL editor")
                                            } else {
                                                i18n.read().t("Edit TTL")
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
                                            label: i18n.read().t("Delete key"),
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

                                        {i18n.read().t("Confirm delete")}
                                    }

                                    p {
                                        style: "{overlay_modal_body_style()}",

                                        {format!("{} \"{}\". {}", i18n.read().t("Delete current key?"), display_key, i18n.read().t("This action cannot be undone."))}
                                    }

                                    div {
                                        style: "{overlay_modal_actions_style()}",

                                        button {
                                            style: "{secondary_action_button_style()}",
                                            disabled: delete_key_processing(),
                                            onclick: move |_| delete_key_confirm.set(false),

                                            {i18n.read().t("Cancel")}
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
                                                                toast_manager.write().success(&i18n.read().t("Delete key succeeded"));
                                                                selected_key.set(String::new());
                                                                on_refresh.call(());
                                                            }
                                                            Err(error) => {
                                                                toast_manager.write().error(&format!("{}{}", i18n.read().t("Delete failed: "), error));
                                                            }
                                                        }

                                                        delete_key_processing.set(false);
                                                    });
                                                }
                                            },

                                            {if delete_key_processing() { i18n.read().t("Deleting...") } else { i18n.read().t("Confirm delete") }}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
}
