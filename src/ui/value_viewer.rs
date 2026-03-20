use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType};
use crate::ui::editable_field::EditableField;
use crate::ui::json_viewer::{is_json_content, JsonViewer};
use arboard::Clipboard;
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum BinaryFormat {
    #[default]
    Hex,
    Base64,
}

#[derive(Clone, PartialEq)]
struct HashDeleteTarget {
    field: String,
}

fn is_binary_data(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    if data.len() >= 2 && data[0] == 0xAC && data[1] == 0xED {
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

#[component]
fn CopyIcon() -> Element {
    rsx! {
        svg {
            width: "15",
            height: "15",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            rect {
                x: "9",
                y: "9",
                width: "13",
                height: "13",
                rx: "2",
                ry: "2",
            }

            path {
                d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1",
            }
        }
    }
}

#[component]
fn EditIcon() -> Element {
    rsx! {
        svg {
            width: "15",
            height: "15",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7",
            }

            path {
                d: "M18.375 2.625a1.5 1.5 0 1 1 3 3L12 15l-4 1 1-4Z",
            }
        }
    }
}

#[component]
fn DeleteIcon() -> Element {
    rsx! {
        svg {
            width: "15",
            height: "15",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path { d: "M3 6h18" }
            path { d: "M8 6V4h8v2" }
            path { d: "M19 6l-1 14H6L5 6" }
            path { d: "M10 11v6" }
            path { d: "M14 11v6" }
        }
    }
}

async fn load_key_data(
    pool: ConnectionPool,
    key: String,
    mut key_info: Signal<Option<KeyInfo>>,
    mut string_value: Signal<String>,
    mut hash_value: Signal<HashMap<String, String>>,
    mut is_binary: Signal<bool>,
    binary_format: Signal<BinaryFormat>,
    mut loading: Signal<bool>,
) -> Result<(), String> {
    if key.is_empty() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        is_binary.set(false);
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

                if is_binary_data(&bytes) {
                    is_binary.set(true);
                    let formatted = format_bytes(&bytes, binary_format());
                    string_value.set(formatted);
                } else {
                    is_binary.set(false);
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
            }
            KeyType::Hash => {
                let fields = pool
                    .get_hash_all(&key)
                    .await
                    .map_err(|e| format!("读取 hash 数据失败: {e}"))?;
                tracing::info!("Hash loaded: {} fields", fields.len());
                hash_value.set(fields);
                string_value.set(String::new());
                is_binary.set(false);
            }
            _ => {
                tracing::info!("Type: {:?}", info.key_type);
                string_value.set(String::new());
                hash_value.set(HashMap::new());
                is_binary.set(false);
            }
        }

        Ok::<(), String>(())
    }
    .await;

    if load_result.is_err() {
        key_info.set(None);
        string_value.set(String::new());
        hash_value.set(HashMap::new());
        is_binary.set(false);
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
    let loading = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut is_binary = use_signal(|| false);
    let mut binary_format = use_signal(BinaryFormat::default);

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

        let pool = pool.clone();

        spawn(async move {
            tracing::info!("Loading key: {}", key);

            if let Err(error) = load_key_data(
                pool,
                key,
                key_info,
                string_value,
                hash_value,
                is_binary,
                binary_format,
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
    let display_key = selected_key.read().clone();

    let status_message = hash_status_message();
    let status_color = if hash_status_error() {
        "#f87171"
    } else {
        "#4ec9b0"
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

    rsx! {
        div {
            flex: "1",
            height: "100%",
            background: "#1e1e1e",
            display: "flex",
            flex_direction: "column",

            div {
                padding: "12px 16px",
                border_bottom: "1px solid #3c3c3c",
                background: "#252526",

                if let Some(ref info) = info {
                    div {
                        display: "flex",
                        justify_content: "space_between",
                        align_items: "center",

                        div {
                            span {
                                color: "#888",
                                font_size: "12px",
                                margin_right: "8px",

                                "Key:"
                            }

                            span {
                                color: "#4ec9b0",
                                font_size: "14px",
                                font_weight: "bold",

                                "{display_key}"
                            }
                        }

                        div {
                            display: "flex",
                            gap: "16px",
                            font_size: "12px",
                            color: "#888",

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
                        color: "#888",

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
                        color: "#888",
                        text_align: "center",
                        padding: "20px",

                        "Loading..."
                    }
                } else if display_key.is_empty() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",

                        "No key selected"
                    }
                } else if let Some(info) = info {
                    match info.key_type {
                        KeyType::String => {
                            let is_json = !is_binary() && is_json_content(&str_val);
                            
                            rsx! {
                                div {
                                    if is_binary() {
                                        div {
                                            display: "flex",
                                            gap: "8px",
                                            align_items: "center",
                                            margin_bottom: "12px",

                                            span {
                                                color: "#f59e0b",
                                                font_size: "12px",

                                                "二进制数据 (Java序列化或其他)"
                                            }

                                            button {
                                                padding: "4px 8px",
                                                background: if binary_format() == BinaryFormat::Hex { "#0e639c" } else { "#3c3c3c" },
                                                color: "white",
                                                border: "none",
                                                border_radius: "4px",
                                                cursor: "pointer",
                                                font_size: "12px",
                                                onclick: move |_| binary_format.set(BinaryFormat::Hex),

                                                "Hex"
                                            }

                                            button {
                                                padding: "4px 8px",
                                                background: if binary_format() == BinaryFormat::Base64 { "#0e639c" } else { "#3c3c3c" },
                                                color: "white",
                                                border: "none",
                                                border_radius: "4px",
                                                cursor: "pointer",
                                                font_size: "12px",
                                                onclick: move |_| binary_format.set(BinaryFormat::Base64),

                                                "Base64"
                                            }
                                        }
                                    }

                                    if is_json {
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
                                            background: "#2d2d2d",
                                            border: "1px solid #3c3c3c",
                                            border_radius: "6px",
                                            color: "white",
                                            value: "{search_value}",
                                            placeholder: "搜索 key 或 value",
                                            oninput: move |event| hash_search.set(event.value()),
                                        }

                                        button {
                                            padding: "8px 12px",
                                            background: "#0e639c",
                                            color: "white",
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
                                            color: "#888",
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
                                    border: "1px solid #3c3c3c",
                                    border_radius: "8px",
                                    background: "#252526",

                                    table {
                                        width: "100%",
                                        min_width: "920px",
                                        border_collapse: "collapse",

                                        thead {
                                            tr {
                                                background: "#2d2d2d",
                                                border_bottom: "1px solid #3c3c3c",

                                                th {
                                                    width: "72px",
                                                    padding: "12px",
                                                    color: "#888",
                                                    font_size: "12px",
                                                    font_weight: "600",
                                                    text_align: "left",

                                                    "ID"
                                                }

                                                th {
                                                    width: "32%",
                                                    padding: "12px",
                                                    color: "#888",
                                                    font_size: "12px",
                                                    font_weight: "600",
                                                    text_align: "left",

                                                    "key"
                                                }

                                                th {
                                                    padding: "12px",
                                                    color: "#888",
                                                    font_size: "12px",
                                                    font_weight: "600",
                                                    text_align: "left",

                                                    "value"
                                                }

                                                th {
                                                    width: "156px",
                                                    padding: "12px",
                                                    color: "#888",
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
                                                    background: "#202a33",
                                                    border_bottom: "1px solid #3c3c3c",

                                                    td {
                                                        padding: "12px",
                                                        color: "#4ec9b0",
                                                        vertical_align: "top",

                                                        "+"
                                                    }

                                                    td {
                                                        padding: "12px",
                                                        vertical_align: "top",

                                                        input {
                                                            width: "100%",
                                                            padding: "8px 10px",
                                                            background: "#1e1e1e",
                                                            border: "1px solid #555",
                                                            border_radius: "6px",
                                                            color: "white",
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
                                                            background: "#1e1e1e",
                                                            border: "1px solid #555",
                                                            border_radius: "6px",
                                                            color: "white",
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
                                                                color: "white",
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
                                                                                        is_binary,
                                                                                        binary_format,
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
                                                                background: "#5a5a5a",
                                                                color: "white",
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
                                                        background: "#1f2937",
                                                        border_bottom: "1px solid #3c3c3c",

                                                        td {
                                                            padding: "12px",
                                                            color: "#4ec9b0",
                                                            vertical_align: "top",

                                                            "{index + 1}"
                                                        }

                                                        td {
                                                            padding: "12px",
                                                            vertical_align: "top",

                                                            input {
                                                                width: "100%",
                                                                padding: "8px 10px",
                                                                background: "#1e1e1e",
                                                                border: "1px solid #555",
                                                                border_radius: "6px",
                                                                color: "white",
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
                                                                background: "#1e1e1e",
                                                                border: "1px solid #555",
                                                                border_radius: "6px",
                                                                color: "white",
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
                                                                    color: "white",
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
                                                                                            is_binary,
                                                                                            binary_format,
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
                                                                    background: "#5a5a5a",
                                                                    color: "white",
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
                                                        border_bottom: "1px solid #333",
                                                        background: "#252526",

                                                        td {
                                                            padding: "12px",
                                                            color: "#888",
                                                            vertical_align: "top",

                                                            "{index + 1}"
                                                        }

                                                        td {
                                                            padding: "12px",
                                                            vertical_align: "top",

                                                            div {
                                                                color: "#4ec9b0",
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
                                                                color: "white",
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

                                                                    CopyIcon {}
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

                                                                    EditIcon {}
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

                                                                    DeleteIcon {}
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
                                                        color: "#888",
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
                                            background: "#252526",
                                            border: "1px solid #3c3c3c",
                                            border_radius: "10px",
                                            padding: "20px",

                                            h3 {
                                                color: "white",
                                                margin: "0 0 12px 0",
                                                font_size: "18px",

                                                "确认删除"
                                            }

                                            p {
                                                color: "#bbb",
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
                                                    background: "#5a5a5a",
                                                    color: "white",
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
                                                    color: "white",
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
                                                                            is_binary,
                                                                            binary_format,
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
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",

                                    "List Items (use CLI to view)"
                                }
                            }
                        }
                        KeyType::Set => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",

                                    "Set Members (use CLI to view)"
                                }
                            }
                        }
                        KeyType::ZSet => {
                            rsx! {
                                div {
                                    color: "#888",
                                    margin_bottom: "12px",
                                    font_size: "14px",

                                    "Sorted Set Members (use CLI to view)"
                                }
                            }
                        }
                        _ => {
                            rsx! {
                                div {
                                    color: "#888",

                                    "Unsupported type"
                                }
                            }
                        }
                    }
                } else {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",

                        "No data"
                    }
                }
            }
        }
    }
}
