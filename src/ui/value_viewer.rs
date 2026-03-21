use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType};
use crate::serialization::is_java_serialization;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_ERROR,
    COLOR_PRIMARY, COLOR_SUCCESS, COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
    COLOR_TEXT_SOFT, COLOR_WARNING,
};
use crate::ui::editable_field::EditableField;
use crate::ui::icons::{IconCopy, IconEdit, IconTrash};
use crate::ui::java_viewer::JavaSerializedViewer;
use crate::ui::json_viewer::{is_json_content, JsonViewer};
use crate::ui::pagination::LargeKeyWarning;
use arboard::Clipboard;
use dioxus::prelude::*;
use std::collections::HashMap;

const LARGE_KEY_THRESHOLD: usize = 1000;
const STATUS_SUCCESS_BG: &str = "rgba(16, 124, 16, 0.12)";
const STATUS_ERROR_BG: &str = "rgba(209, 52, 56, 0.12)";
const ROW_CREATE_BG: &str = "rgba(15, 108, 189, 0.08)";
const ROW_EDIT_BG: &str = "rgba(15, 108, 189, 0.12)";

#[derive(Clone, Copy, PartialEq, Default)]
pub enum BinaryFormat {
    #[default]
    Hex,
    Base64,
    JavaSerialized,
}

#[derive(Clone, PartialEq)]
struct HashDeleteTarget {
    field: String,
}

fn is_binary_data(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    if is_java_serialization(data) {
        return true;
    }

    let non_printable_count = data
        .iter()
        .filter(|&&b| b < 0x20 && b != 0x09 && b != 0x0A && b != 0x0D)
        .count();

    non_printable_count > data.len() / 10
}

fn format_bytes(data: &[u8], format: BinaryFormat) -> String {
    match format {
        BinaryFormat::Hex => data
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" "),
        BinaryFormat::Base64 => {
            use base64::{engine::general_purpose, Engine as _};
            general_purpose::STANDARD.encode(data)
        }
        BinaryFormat::JavaSerialized => {
            if is_java_serialization(data) {
                format!(
                    "Java 序列化对象 ({} 字节)\n\n请切换到 Java 视图查看解析结果",
                    data.len()
                )
            } else {
                "非 Java 序列化数据".to_string()
            }
        }
    }
}

fn copy_value_to_clipboard(value: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(value.to_string())
        .map_err(|e| e.to_string())
}

fn sorted_hash_entries(fields: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut entries: Vec<_> = fields
        .iter()
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

async fn load_key_data(
    pool: ConnectionPool,
    key: String,
    mut key_info: Signal<Option<KeyInfo>>,
    mut string_value: Signal<String>,
    mut hash_value: Signal<HashMap<String, String>>,
    mut list_value: Signal<Vec<String>>,
    mut set_value: Signal<Vec<String>>,
    mut zset_value: Signal<Vec<(String, f64)>>,
    mut is_binary: Signal<bool>,
    mut binary_format: Signal<BinaryFormat>,
    mut java_serialization_info: Signal<Option<Vec<u8>>>,
    mut loading: Signal<bool>,
) -> Result<(), String> {
    if key.is_empty() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        list_value.set(Vec::new());
        set_value.set(Vec::new());
        zset_value.set(Vec::new());
        is_binary.set(false);
        java_serialization_info.set(None);
        loading.set(false);
        return Ok(());
    }

    loading.set(true);

    let load_result = async {
        let info = pool
            .get_key_info(&key)
            .await
            .map_err(|e| format!("读取 key 信息失败: {e}"))?;

        tracing::info!("Key info loaded: {:?}", info.key_type);
        key_info.set(Some(info.clone()));

        match info.key_type {
            KeyType::String => {
                let bytes = pool
                    .get_string_bytes(&key)
                    .await
                    .map_err(|e| format!("读取字符串值失败: {e}"))?;

                tracing::info!("String value loaded: {} bytes", bytes.len());

                if bytes.len() >= 4 {
                    tracing::info!("First 10 bytes: {:02x?}", &bytes[..10.min(bytes.len())]);
                }

                if is_binary_data(&bytes) {
                    is_binary.set(true);

                    if is_java_serialization(&bytes) {
                        tracing::info!("Java serialization detected");
                        java_serialization_info.set(Some(bytes.clone()));
                        binary_format.set(BinaryFormat::JavaSerialized);
                    } else {
                        java_serialization_info.set(None);
                    }

                    let formatted = format_bytes(&bytes, binary_format());
                    string_value.set(formatted);
                } else {
                    is_binary.set(false);
                    java_serialization_info.set(None);
                    match String::from_utf8(bytes) {
                        Ok(s) => string_value.set(s),
                        Err(_) => {
                            is_binary.set(true);
                            let bytes = pool
                                .get_string_bytes(&key)
                                .await
                                .map_err(|e| format!("读取字符串值失败: {e}"))?;
                            string_value.set(format_bytes(&bytes, binary_format()));
                        }
                    }
                }
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
            }
            KeyType::Hash => {
                let fields = pool
                    .get_hash_all(&key)
                    .await
                    .map_err(|e| format!("读取 hash 数据失败: {e}"))?;
                tracing::info!("Hash loaded: {} fields", fields.len());
                hash_value.set(fields);
                string_value.set(String::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                is_binary.set(false);
                java_serialization_info.set(None);
            }
            KeyType::List => {
                let items = pool
                    .get_list_range(&key, 0, -1)
                    .await
                    .map_err(|e| format!("读取 list 数据失败: {e}"))?;
                tracing::info!("List loaded: {} items", items.len());
                list_value.set(items);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                is_binary.set(false);
                java_serialization_info.set(None);
            }
            KeyType::Set => {
                let members = pool
                    .get_set_members(&key)
                    .await
                    .map_err(|e| format!("读取 set 数据失败: {e}"))?;
                tracing::info!("Set loaded: {} members", members.len());
                set_value.set(members);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                zset_value.set(Vec::new());
                is_binary.set(false);
                java_serialization_info.set(None);
            }
            KeyType::ZSet => {
                let members = pool
                    .get_zset_range(&key, 0, -1)
                    .await
                    .map_err(|e| format!("读取 zset 数据失败: {e}"))?;
                tracing::info!("ZSet loaded: {} members", members.len());
                zset_value.set(members);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                is_binary.set(false);
                java_serialization_info.set(None);
            }
            _ => {
                tracing::info!("Type: {:?}", info.key_type);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                list_value.set(Vec::new());
                set_value.set(Vec::new());
                zset_value.set(Vec::new());
                is_binary.set(false);
                java_serialization_info.set(None);
            }
        }

        Ok::<(), String>(())
    }
    .await;

    if load_result.is_err() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        list_value.set(Vec::new());
        set_value.set(Vec::new());
        zset_value.set(Vec::new());
        is_binary.set(false);
        java_serialization_info.set(None);
    }

    loading.set(false);
    load_result
}

#[component]
pub fn ValueViewer(
    connection_pool: ConnectionPool,
    selected_key: Signal<String>,
    on_refresh: EventHandler<()>,
) -> Element {
    let key_info = use_signal(|| None::<KeyInfo>);
    let mut string_value = use_signal(String::new);
    let hash_value = use_signal(HashMap::new);
    let list_value = use_signal(Vec::new);
    let set_value = use_signal(Vec::new);
    let zset_value = use_signal(Vec::new);
    let loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut is_binary = use_signal(|| false);
    let mut binary_format = use_signal(BinaryFormat::default);
    let mut java_serialization_info = use_signal(|| None::<Vec<u8>>);

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
    let mut set_page = use_signal(|| 0usize);
    let mut set_total = use_signal(|| 0usize);
    let mut zset_page = use_signal(|| 0usize);
    let mut zset_total = use_signal(|| 0usize);
    let mut show_large_key_warning = use_signal(|| false);

    let pool = connection_pool.clone();
    let pool_for_edit = connection_pool.clone();
    let pool_for_reload = connection_pool.clone();

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
        java_serialization_info.set(None);
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

        let pool = pool.clone();

        spawn(async move {
            tracing::info!("Loading key: {}", key);

            if let Err(error) = load_key_data(
                pool,
                key,
                key_info,
                string_value,
                hash_value,
                list_value,
                set_value,
                zset_value,
                is_binary,
                binary_format,
                java_serialization_info,
                loading,
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

    let key_for_edit = selected_key;

    let info = key_info();
    let is_loading = loading();
    let str_val = string_value();
    let hash_val = hash_value();
    let list_val = list_value();
    let set_val = set_value();
    let zset_val = zset_value();
    let display_key = selected_key.read().clone();

    let status_message = hash_status_message();
    let status_color = if hash_status_error() {
        COLOR_ERROR
    } else {
        COLOR_ACCENT
    };
    let active_hash_action = hash_action();
    let editing_field_name = editing_hash_field();
    let delete_target = deleting_hash_field();

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

                div {
                    padding: "12px 16px",
                    border_bottom: "1px solid {COLOR_BORDER}",
                    background: COLOR_BG_SECONDARY,

                    if let Some(ref info) = info {
                        div {
                            display: "flex",
                            justify_content: "space_between",
                            align_items: "center",

                            div {
                                span {
                                    color: COLOR_TEXT_SECONDARY,
                                    font_size: "12px",
                                    margin_right: "8px",

                                    "Key:"
                                }

                                span {
                                    color: COLOR_ACCENT,
                                    font_size: "14px",
                                    font_weight: "bold",

                                    "{display_key}"
                                }
                            }

                            div {
                                display: "flex",
                                gap: "16px",
                                font_size: "12px",
                                color: COLOR_TEXT_SECONDARY,

                                span {
                                    "Type: {info.key_type}"
                                }

                                if let Some(ttl) = info.ttl {
                                    span {
                                        "TTL: {ttl}s"
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            color: COLOR_TEXT_SECONDARY,

                            "Select a key to view"
                        }
                    }
                }

                div {
                    flex: "1",
                    overflow_y: "auto",
                    padding: "16px",

                    if is_loading {
                        div {
                            color: COLOR_TEXT_SECONDARY,
                            text_align: "center",
                            padding: "20px",

                            "Loading..."
                        }
                    } else if display_key.is_empty() {
                        div {
                            color: COLOR_TEXT_SECONDARY,
                            text_align: "center",
                            padding: "20px",

                            "No key selected"
                        }
                    } else if let Some(info) = info {
                        match info.key_type {
                            KeyType::String => {
                                let is_json = !is_binary() && is_json_content(&str_val);
                                let java_info_val = java_serialization_info();
                                let is_java = java_info_val.is_some();

                                rsx! {
                                    div {
                                        if is_binary() {
                                            div {
                                                display: "flex",
                                                gap: "8px",
                                                align_items: "center",
                                                margin_bottom: "12px",
                                                flex_wrap: "wrap",

                                                if is_java {
                                                    span {
                                                        color: COLOR_SUCCESS,
                                                        font_size: "12px",

                                                        "Java 序列化对象"
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

                                                if is_java {
                                                    button {
                                                        padding: "4px 8px",
                                                        background: if binary_format() == BinaryFormat::JavaSerialized { COLOR_PRIMARY } else { COLOR_BG_TERTIARY },
                                                        color: if binary_format() == BinaryFormat::JavaSerialized { COLOR_TEXT_CONTRAST } else { COLOR_TEXT },
                                                        border: "none",
                                                        border_radius: "4px",
                                                        cursor: "pointer",
                                                        font_size: "12px",
                                                        onclick: move |_| binary_format.set(BinaryFormat::JavaSerialized),

                                                        "Java解析"
                                                    }
                                                }
                                            }
                                        }

                                        if is_binary() && binary_format() == BinaryFormat::JavaSerialized {
                                            if let Some(ref data) = java_info_val {
                                                JavaSerializedViewer {
                                                    data: data.clone(),
                                                }
                                            } else {
                                                div {
                                                    padding: "16px",
                                                    background: COLOR_BG_TERTIARY,
                                                    border_radius: "8px",
                                                    color: COLOR_TEXT_SECONDARY,

                                                    "解析失败"
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
                                        justify_content: "space_between",
                                        align_items: "center",
                                        gap: "12px",
                                        flex_wrap: "wrap",
                                        margin_bottom: "12px",

                                        div {
                                            display: "flex",
                                            gap: "8px",
                                            align_items: "center",
                                            flex_wrap: "wrap",

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
                                                oninput: move |event| hash_search.set(event.value()),
                                            }

                                            button {
                                                padding: "8px 12px",
                                                background: "#0e639c",
                                                color: COLOR_TEXT_CONTRAST,
                                                border: "none",
                                                border_radius: "6px",
                                                cursor: "pointer",
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
                                        }

                                        div {
                                            text_align: "right",

                                            div {
                                                color: COLOR_TEXT_SECONDARY,
                                                font_size: "13px",

                                                "Hash Fields ({filtered_entries.len()}/{hash_val.len()})"
                                            }

                                            if !status_message.is_empty() {
                                                div {
                                                    margin_top: "4px",
                                                    color: "{status_color}",
                                                    font_size: "12px",

                                                    "{status_message}"
                                                }
                                            }
                                        }
                                    }

                                    div {
                                        overflow_x: "auto",
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "8px",
                                        background: COLOR_BG_SECONDARY,

                                        table {
                                            width: "100%",
                                            min_width: "920px",
                                            border_collapse: "collapse",

                                            thead {
                                                tr {
                                                    background: COLOR_BG_TERTIARY,
                                                    border_bottom: "1px solid {COLOR_BORDER}",

                                                    th {
                                                        width: "72px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "ID"
                                                    }

                                                    th {
                                                        width: "32%",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "key"
                                                    }

                                                    th {
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "value"
                                                    }

                                                    th {
                                                        width: "156px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

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
                                                                    padding: "6px 10px",
                                                                    background: "#38a169",
                                                                    color: COLOR_TEXT_CONTRAST,
                                                                    border: "none",
                                                                    border_radius: "6px",
                                                                    cursor: "pointer",
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
                                                                                        if let Err(error) = load_key_data(
                                                                                            pool.clone(),
                                                                                            key.clone(),
                                                                                            key_info,
                                                                                            string_value,
                                                                                            hash_value,
                                                                                            list_value,
                                                                                            set_value,
                                                                                            zset_value,
                                                                                            is_binary,
                                                                                            binary_format,
                                                                                            java_serialization_info,
                                                                                            loading,
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
                                                                        padding: "6px 10px",
                                                                        background: "#38a169",
                                                                        color: COLOR_TEXT_CONTRAST,
                                                                        border: "none",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                                            if let Err(error) = load_key_data(
                                                                                                pool.clone(),
                                                                                                key.clone(),
                                                                                                key_info,
                                                                                                string_value,
                                                                                                hash_value,
                                                                                                list_value,
                                                                                                set_value,
                                                                                                zset_value,
                                                                                                is_binary,
                                                                                                binary_format,
                                                                                                java_serialization_info,
                                                                                                loading,
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(47, 133, 90, 0.16)",
                                                                        color: "#68d391",
                                                                        border: "1px solid rgba(104, 211, 145, 0.28)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
                                                                        disabled: active_hash_action.is_some(),
                                                                        title: "复制值",
                                                                        aria_label: "复制值",
                                                                        onclick: {
                                                                            let value = value.clone();
                                                                            move |_| {
                                                                                match copy_value_to_clipboard(&value) {
                                                                                    Ok(_) => {
                                                                                        hash_status_message.set("复制成功".to_string());
                                                                                        hash_status_error.set(false);
                                                                                    }
                                                                                    Err(error) => {
                                                                                        tracing::error!("Failed to copy hash value: {}", error);
                                                                                        hash_status_message.set(format!("复制失败：{error}"));
                                                                                        hash_status_error.set(true);
                                                                                    }
                                                                                }
                                                                            }
                                                                        },

                                                                        IconCopy { size: Some(15) }
                                                                    }

                                                                    button {
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(49, 130, 206, 0.18)",
                                                                        color: "#63b3ed",
                                                                        border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(197, 48, 48, 0.18)",
                                                                        color: "#f87171",
                                                                        border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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

                                    if let Some(target) = delete_target {
                                        div {
                                            position: "fixed",
                                            top: "0",
                                            left: "0",
                                            right: "0",
                                            bottom: "0",
                                            background: "rgba(0, 0, 0, 0.7)",
                                            display: "flex",
                                            align_items: "center",
                                            justify_content: "center",
                                            z_index: "1000",

                                            div {
                                                width: "420px",
                                                max_width: "calc(100vw - 32px)",
                                                background: COLOR_BG_SECONDARY,
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "10px",
                                                padding: "20px",

                                                h3 {
                                                    color: COLOR_TEXT,
                                                    margin: "0 0 12px 0",
                                                    font_size: "18px",

                                                    "确认删除"
                                                }

                                                p {
                                                    color: COLOR_TEXT_SOFT,
                                                    margin: "0 0 18px 0",
                                                    line_height: "1.6",
                                                    word_break: "break-all",

                                                    "确定删除 hash field '{target.field}' 吗？"
                                                }

                                                div {
                                                    display: "flex",
                                                    justify_content: "flex_end",
                                                    gap: "8px",

                                                    button {
                                                        padding: "8px 12px",
                                                        background: COLOR_BG_TERTIARY,
                                                        color: COLOR_TEXT,
                                                        border: "none",
                                                        border_radius: "6px",
                                                        cursor: "pointer",
                                                        disabled: active_hash_action.is_some(),
                                                        onclick: move |_| deleting_hash_field.set(None),

                                                        "取消"
                                                    }

                                                    button {
                                                        padding: "8px 12px",
                                                        background: "#c53030",
                                                        color: COLOR_TEXT_CONTRAST,
                                                        border: "none",
                                                        border_radius: "6px",
                                                        cursor: "pointer",
                                                        disabled: active_hash_action.is_some(),
                                                        onclick: {
                                                            let pool = connection_pool.clone();
                                                            let key = display_key.clone();
                                                            let field = target.field.clone();
                                                            let delete_action = format!("delete:{}", target.field);
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
                                                                            deleting_hash_field.set(None);
                                                                            if editing_hash_field().as_deref() == Some(field.as_str()) {
                                                                                editing_hash_field.set(None);
                                                                                editing_hash_key.set(String::new());
                                                                                editing_hash_value.set(String::new());
                                                                            }
                                                                            hash_status_message.set("删除成功".to_string());
                                                                            hash_status_error.set(false);
    if let Err(error) = load_key_data(
                                                                                            pool.clone(),
                                                                                            key.clone(),
                                                                                            key_info,
                                                                                            string_value,
                                                                                            hash_value,
                                                                                            list_value,
                                                                                            set_value,
                                                                                            zset_value,
                                                                                            is_binary,
                                                                                            binary_format,
                                                                                            java_serialization_info,
                                                                                            loading,
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
                                }
                            }
                            KeyType::List => {
                                rsx! {
                                    div {
                                        display: "flex",
                                        justify_content: "space_between",
                                        align_items: "center",
                                        gap: "12px",
                                        margin_bottom: "12px",

                                        div {
                                            display: "flex",
                                            gap: "8px",
                                            align_items: "center",

                                            input {
                                                width: "300px",
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
                                                padding: "8px 12px",
                                                background: "#38a169",
                                                color: COLOR_TEXT_CONTRAST,
                                                border: "none",
                                                border_radius: "6px",
                                                cursor: "pointer",
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
                                                                    if let Err(error) = load_key_data(
                                                                        pool.clone(),
                                                                        key.clone(),
                                                                        key_info,
                                                                        string_value,
                                                                        hash_value,
                                                                        list_value,
                                                                        set_value,
                                                                        zset_value,
                                                                        is_binary,
                                                                        binary_format,
                                                                        java_serialization_info,
                                                                        loading,
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
                                                padding: "8px 12px",
                                                background: "#0e639c",
                                                color: COLOR_TEXT_CONTRAST,
                                                border: "none",
                                                border_radius: "6px",
                                                cursor: "pointer",
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
                                                                    if let Err(error) = load_key_data(
                                                                        pool.clone(),
                                                                        key.clone(),
                                                                        key_info,
                                                                        string_value,
                                                                        hash_value,
                                                                        list_value,
                                                                        set_value,
                                                                        zset_value,
                                                                        is_binary,
                                                                        binary_format,
                                                                        java_serialization_info,
                                                                        loading,
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
                                        }

                                        div {
                                            color: COLOR_TEXT_SECONDARY,
                                            font_size: "13px",

                                            "List Items ({list_val.len()})"
                                        }
                                    }

                                    if !list_status_message.read().is_empty() {
                                        div {
                                            margin_bottom: "12px",
                                            padding: "8px 12px",
                                            background: if list_status_error() { STATUS_ERROR_BG } else { STATUS_SUCCESS_BG },
                                            border_radius: "6px",
                                            color: if list_status_error() { COLOR_ERROR } else { COLOR_SUCCESS },
                                            font_size: "13px",

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
                                        overflow_x: "auto",
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "8px",
                                        background: COLOR_BG_SECONDARY,

                                        table {
                                            width: "100%",
                                            border_collapse: "collapse",

                                            thead {
                                                tr {
                                                    background: COLOR_BG_TERTIARY,
                                                    border_bottom: "1px solid {COLOR_BORDER}",

                                                    th {
                                                        width: "72px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "Index"
                                                    }

                                                    th {
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "Value"
                                                    }

                                                    th {
                                                        width: "156px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "Action"
                                                    }
                                                }
                                            }

                                            tbody {
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
                                                                        padding: "6px 10px",
                                                                        background: "#38a169",
                                                                        color: COLOR_TEXT_CONTRAST,
                                                                        border: "none",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                                            if let Err(error) = load_key_data(
                                                                                                pool.clone(),
                                                                                                key.clone(),
                                                                                                key_info,
                                                                                                string_value,
                                                                                                hash_value,
                                                                                                list_value,
                                                                                                set_value,
                                                                                                zset_value,
                                                                                                is_binary,
                                                                                                binary_format,
                                                                                                java_serialization_info,
                                                                                                loading,
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(49, 130, 206, 0.18)",
                                                                        color: "#63b3ed",
                                                                        border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(197, 48, 48, 0.18)",
                                                                        color: "#f87171",
                                                                        border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                                            if let Err(error) = load_key_data(
                                                                                                pool.clone(),
                                                                                                key.clone(),
                                                                                                key_info,
                                                                                                string_value,
                                                                                                hash_value,
                                                                                                list_value,
                                                                                                set_value,
                                                                                                zset_value,
                                                                                                is_binary,
                                                                                                binary_format,
                                                                                                java_serialization_info,
                                                                                                loading,
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
                            KeyType::Set => {
                                rsx! {
                                    div {
                                        display: "flex",
                                        justify_content: "space_between",
                                        align_items: "center",
                                        gap: "12px",
                                        margin_bottom: "12px",

                                        div {
                                            display: "flex",
                                            gap: "8px",
                                            align_items: "center",

                                            input {
                                                width: "200px",
                                                padding: "8px 10px",
                                                background: COLOR_BG_TERTIARY,
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "6px",
                                                color: COLOR_TEXT,
                                                value: "{set_search}",
                                                placeholder: "搜索成员",
                                                oninput: move |event| set_search.set(event.value()),
                                            }

                                            input {
                                                width: "200px",
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
                                                padding: "8px 12px",
                                                background: "#38a169",
                                                color: COLOR_TEXT_CONTRAST,
                                                border: "none",
                                                border_radius: "6px",
                                                cursor: "pointer",
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
                                                                    if let Err(error) = load_key_data(
                                                                        pool.clone(),
                                                                        key.clone(),
                                                                        key_info,
                                                                        string_value,
                                                                        hash_value,
                                                                        list_value,
                                                                        set_value,
                                                                        zset_value,
                                                                        is_binary,
                                                                        binary_format,
                                                                        java_serialization_info,
                                                                        loading,
                                                                    ).await {
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
                                        }

                                        div {
                                            text_align: "right",

                                            div {
                                                color: COLOR_TEXT_SECONDARY,
                                                font_size: "13px",

                                                "Set Members ({filtered_set_members.len()}/{set_val.len()})"
                                            }
                                        }
                                    }

                                    if !set_status_message.read().is_empty() {
                                        div {
                                            margin_bottom: "12px",
                                            padding: "8px 12px",
                                            background: if set_status_error() { STATUS_ERROR_BG } else { STATUS_SUCCESS_BG },
                                            border_radius: "6px",
                                            color: if set_status_error() { COLOR_ERROR } else { COLOR_SUCCESS },
                                            font_size: "13px",

                                            "{set_status_message}"
                                        }
                                    }

                                    if set_val.len() > LARGE_KEY_THRESHOLD {
                                        LargeKeyWarning {
                                            key_type: "Set".to_string(),
                                            size: set_val.len(),
                                            threshold: LARGE_KEY_THRESHOLD,
                                        }
                                    }

                                    div {
                                        overflow_x: "auto",
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "8px",
                                        background: COLOR_BG_SECONDARY,
                                        max_height: "500px",
                                        overflow_y: "auto",

                                        table {
                                            width: "100%",
                                            border_collapse: "collapse",

                                            thead {
                                                tr {
                                                    background: COLOR_BG_TERTIARY,
                                                    position: "sticky",
                                                    top: "0",

                                                    th {
                                                        width: "72px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",
                                                        border_bottom: "1px solid {COLOR_BORDER}",

                                                        "#"
                                                    }

                                                    th {
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",
                                                        border_bottom: "1px solid {COLOR_BORDER}",

                                                        "Member"
                                                    }

                                                    th {
                                                        width: "100px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",
                                                        border_bottom: "1px solid {COLOR_BORDER}",

                                                        "Action"
                                                    }
                                                }
                                            }

                                            tbody {
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
                                                                        padding: "6px 10px",
                                                                        background: "#38a169",
                                                                        color: COLOR_TEXT_CONTRAST,
                                                                        border: "none",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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

                                                                                    set_action.set(Some(format!("edit:{}", old_member)));
                                                                                    set_status_message.set(String::new());
                                                                                    set_status_error.set(false);

                                                                                    let edit_result = if new_member == old_member {
                                                                                        Ok(())
                                                                                    } else {
                                                                                        match pool.set_remove(&key, &old_member).await {
                                                                                            Ok(_) => pool.set_add(&key, &new_member).await.map(|_| ()),
                                                                                            Err(e) => Err(e),
                                                                                        }
                                                                                    };

                                                                                    match edit_result {
                                                                                        Ok(_) => {
                                                                                            editing_set_member.set(None);
                                                                                            editing_set_member_value.set(String::new());
                                                                                            set_status_message.set("修改成功".to_string());
                                                                                            set_status_error.set(false);
                                                                                            if let Err(error) = load_key_data(
                                                                                                pool.clone(),
                                                                                                key.clone(),
                                                                                                key_info,
                                                                                                string_value,
                                                                                                hash_value,
                                                                                                list_value,
                                                                                                set_value,
                                                                                                zset_value,
                                                                                                is_binary,
                                                                                                binary_format,
                                                                                                java_serialization_info,
                                                                                                loading,
                                                                                            ).await {
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

                                                                        if set_action().as_deref() == Some(format!("edit:{}", member).as_str()) { "保存中..." } else { "保存" }
                                                                    }

                                                                    button {
                                                                        padding: "6px 10px",
                                                                        background: COLOR_BG_TERTIARY,
                                                                        color: COLOR_TEXT,
                                                                        border: "none",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(47, 133, 90, 0.16)",
                                                                        color: "#68d391",
                                                                        border: "1px solid rgba(104, 211, 145, 0.28)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
                                                                        title: "复制",
                                                                        onclick: {
                                                                            let member = member.clone();
                                                                            move |_| {
                                                                                match copy_value_to_clipboard(&member) {
                                                                                    Ok(_) => {
                                                                                        set_status_message.set("复制成功".to_string());
                                                                                        set_status_error.set(false);
                                                                                    }
                                                                                    Err(error) => {
                                                                                        set_status_message.set(format!("复制失败：{error}"));
                                                                                        set_status_error.set(true);
                                                                                    }
                                                                                }
                                                                            }
                                                                        },

                                                                        IconCopy { size: Some(15) }
                                                                    }

                                                                    button {
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(49, 130, 206, 0.18)",
                                                                        color: "#63b3ed",
                                                                        border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(197, 48, 48, 0.18)",
                                                                        color: "#f87171",
                                                                        border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                                    set_action.set(Some(format!("delete:{}", member)));
                                                                                    match pool.set_remove(&key, &member).await {
                                                                                        Ok(_) => {
                                                                                            set_status_message.set("删除成功".to_string());
                                                                                            set_status_error.set(false);
                                                                                            if let Err(error) = load_key_data(
                                                                                                pool.clone(),
                                                                                                key.clone(),
                                                                                                key_info,
                                                                                                string_value,
                                                                                                hash_value,
                                                                                                list_value,
                                                                                                set_value,
                                                                                                zset_value,
                                                                                                is_binary,
                                                                                                binary_format,
                                                                                                java_serialization_info,
                                                                                                loading,
                                                                                            ).await {
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
                            }
                            KeyType::ZSet => {
                                rsx! {
                                    div {
                                        display: "flex",
                                        justify_content: "space_between",
                                        align_items: "center",
                                        gap: "12px",
                                        margin_bottom: "12px",
                                        flex_wrap: "wrap",

                                        div {
                                            display: "flex",
                                            gap: "8px",
                                            align_items: "center",
                                            flex_wrap: "wrap",

                                            input {
                                                width: "160px",
                                                padding: "8px 10px",
                                                background: COLOR_BG_TERTIARY,
                                                border: "1px solid {COLOR_BORDER}",
                                                border_radius: "6px",
                                                color: COLOR_TEXT,
                                                value: "{zset_search}",
                                                placeholder: "搜索成员",
                                                oninput: move |event| zset_search.set(event.value()),
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
                                                padding: "8px 12px",
                                                background: "#38a169",
                                                color: COLOR_TEXT_CONTRAST,
                                                border: "none",
                                                border_radius: "6px",
                                                cursor: "pointer",
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
                                                                    if let Err(error) = load_key_data(
                                                                        pool.clone(),
                                                                        key.clone(),
                                                                        key_info,
                                                                        string_value,
                                                                        hash_value,
                                                                        list_value,
                                                                        set_value,
                                                                        zset_value,
                                                                        is_binary,
                                                                        binary_format,
                                                                        java_serialization_info,
                                                                        loading,
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
                                        }

                                        div {
                                            text_align: "right",

                                            div {
                                                color: COLOR_TEXT_SECONDARY,
                                                font_size: "13px",

                                                "ZSet Members ({filtered_zset_members.len()}/{zset_val.len()})"
                                            }
                                        }
                                    }

                                    if !zset_status_message.read().is_empty() {
                                        div {
                                            margin_bottom: "12px",
                                            padding: "8px 12px",
                                            background: if zset_status_error() { STATUS_ERROR_BG } else { STATUS_SUCCESS_BG },
                                            border_radius: "6px",
                                            color: if zset_status_error() { COLOR_ERROR } else { COLOR_SUCCESS },
                                            font_size: "13px",

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
                                        overflow_x: "auto",
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "8px",
                                        background: COLOR_BG_SECONDARY,

                                        table {
                                            width: "100%",
                                            border_collapse: "collapse",

                                            thead {
                                                tr {
                                                    background: COLOR_BG_TERTIARY,
                                                    border_bottom: "1px solid {COLOR_BORDER}",

                                                    th {
                                                        width: "72px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "#"
                                                    }

                                                    th {
                                                        width: "100px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "Score"
                                                    }

                                                    th {
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

                                                        "Member"
                                                    }

                                                    th {
                                                        width: "100px",
                                                        padding: "12px",
                                                        color: COLOR_TEXT_SECONDARY,
                                                        font_size: "12px",
                                                        font_weight: "600",
                                                        text_align: "left",

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
                                                                        padding: "6px 10px",
                                                                        background: "#38a169",
                                                                        color: COLOR_TEXT_CONTRAST,
                                                                        border: "none",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                                            if let Err(error) = load_key_data(
                                                                                                pool.clone(),
                                                                                                key.clone(),
                                                                                                key_info,
                                                                                                string_value,
                                                                                                hash_value,
                                                                                                list_value,
                                                                                                set_value,
                                                                                                zset_value,
                                                                                                is_binary,
                                                                                                binary_format,
                                                                                                java_serialization_info,
                                                                                                loading,
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(49, 130, 206, 0.18)",
                                                                        color: "#63b3ed",
                                                                        border: "1px solid rgba(99, 179, 237, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                        width: "32px",
                                                                        height: "32px",
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        justify_content: "center",
                                                                        background: "rgba(197, 48, 48, 0.18)",
                                                                        color: "#f87171",
                                                                        border: "1px solid rgba(248, 113, 113, 0.30)",
                                                                        border_radius: "6px",
                                                                        cursor: "pointer",
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
                                                                                            if let Err(error) = load_key_data(
                                                                                                pool.clone(),
                                                                                                key.clone(),
                                                                                                key_info,
                                                                                                string_value,
                                                                                                hash_value,
                                                                                                list_value,
                                                                                                set_value,
                                                                                                zset_value,
                                                                                                is_binary,
                                                                                                binary_format,
                                                                                                java_serialization_info,
                                                                                                loading,
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
                                }
                            }
                            _ => {
                                rsx! {
                                    div {
                                        color: COLOR_TEXT_SECONDARY,

                                        "Unsupported type"
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            color: COLOR_TEXT_SECONDARY,
                            text_align: "center",
                            padding: "20px",

                            "No data"
                        }
                    }
                }
            }
        }
}
