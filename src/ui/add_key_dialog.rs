use crate::connection::ConnectionPool;
use crate::redis::KeyType;
use crate::ui::icons::*;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub struct HashField {
    pub field: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Default)]
pub struct ListValue {
    pub value: String,
}

#[derive(Clone, PartialEq, Default)]
pub struct SetValue {
    pub value: String,
}

#[derive(Clone, PartialEq, Default)]
pub struct ZSetMember {
    pub score: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Default)]
pub struct StreamEntry {
    pub id: String,
    pub field: String,
    pub value: String,
}

#[component]
pub fn AddKeyDialog(
    connection_pool: ConnectionPool,
    on_save: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut key_name = use_signal(String::new);
    let mut key_type = use_signal(|| KeyType::String);
    let mut ttl = use_signal(|| String::from("-1"));
    let mut processing = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);

    let mut string_value = use_signal(String::new);
    let mut hash_fields = use_signal(Vec::<HashField>::new);
    let mut list_values = use_signal(Vec::<ListValue>::new);
    let mut set_values = use_signal(Vec::<SetValue>::new);
    let mut zset_members = use_signal(Vec::<ZSetMember>::new);
    let mut stream_entries = use_signal(Vec::<StreamEntry>::new);

    rsx! {
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
            z_index: "1001",

            div {
                background: "#252526",
                padding: "24px",
                border_radius: "8px",
                width: "500px",
                max_height: "80vh",
                display: "flex",
                flex_direction: "column",

                h3 {
                    color: "#4ec9b0",
                    margin_bottom: "16px",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

                    "➕ 新增 Key"
                }

                if let Some(err) = error_msg() {
                    div {
                        color: "#f87171",
                        background: "rgba(248, 113, 113, 0.1)",
                        padding: "8px 12px",
                        border_radius: "4px",
                        margin_bottom: "16px",
                        font_size: "13px",

                        "{err}"
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        font_size: "12px",
                        margin_bottom: "4px",

                        "Key 名称"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        font_size: "13px",
                        box_sizing: "border-box",
                        value: "{key_name}",
                        oninput: move |e| key_name.set(e.value()),
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        font_size: "12px",
                        margin_bottom: "4px",

                        "类型"
                    }

                    select {
                        width: "100%",
                        padding: "8px 12px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        font_size: "13px",
                        box_sizing: "border-box",
                        value: "{key_type}",
                        onchange: move |e| {
                            let new_type = match e.value().as_str() {
                                "String" => KeyType::String,
                                "Hash" => KeyType::Hash,
                                "List" => KeyType::List,
                                "Set" => KeyType::Set,
                                "ZSet" => KeyType::ZSet,
                                "Stream" => KeyType::Stream,
                                _ => KeyType::String,
                            };
                            key_type.set(new_type);
                        },

                        for type_name in ["String", "Hash", "List", "Set", "ZSet", "Stream"] {
                            option {
                                value: type_name,
                                selected: key_type() == KeyType::from(type_name.to_string()),

                                "{type_name}"
                            }
                        }
                    }
                }

                div {
                    margin_bottom: "16px",
                    flex: "1",
                    overflow_y: "auto",

                    label {
                        display: "block",
                        color: "#888",
                        font_size: "12px",
                        margin_bottom: "4px",

                        "Value"
                    }

                    match key_type() {
                        KeyType::String => rsx! {
                            textarea {
                                width: "100%",
                                height: "150px",
                                padding: "8px 12px",
                                background: "#3c3c3c",
                                border: "1px solid #555",
                                border_radius: "4px",
                                color: "white",
                                font_size: "13px",
                                font_family: "monospace",
                                resize: "vertical",
                                box_sizing: "border-box",
                                value: "{string_value}",
                                oninput: move |e| string_value.set(e.value()),
                            }
                        },
                        KeyType::Hash => rsx! {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "8px",

                                for (idx, field) in hash_fields.read().iter().enumerate() {
                                    div {
                                        key: "{idx}",
                                        display: "flex",
                                        gap: "8px",
                                        align_items: "center",

                                        input {
                                            flex: "1",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            placeholder: "Field",
                                            value: "{field.field}",
                                            oninput: {
                                                let mut fields = hash_fields.clone();
                                                move |e| {
                                                    let val = e.value();
                                                    fields.write()[idx].field = val;
                                                }
                                            },
                                        }

                                        input {
                                            flex: "2",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            placeholder: "Value",
                                            value: "{field.value}",
                                            oninput: {
                                                let mut fields = hash_fields.clone();
                                                move |e| {
                                                    let val = e.value();
                                                    fields.write()[idx].value = val;
                                                }
                                            },
                                        }

                                        button {
                                            padding: "4px 8px",
                                            background: "#c53030",
                                            color: "white",
                                            border: "none",
                                            border_radius: "4px",
                                            cursor: "pointer",
                                            font_size: "12px",
                                            onclick: {
                                                let mut fields = hash_fields.clone();
                                                move |_| {
                                                    fields.write().remove(idx);
                                                }
                                            },

                                            IconX { size: Some(12) }
                                        }
                                    }
                                }

                                button {
                                    padding: "6px 12px",
                                    background: "#0e639c",
                                    color: "white",
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "12px",
                                    onclick: move |_| {
                                        hash_fields.write().push(HashField::default());
                                    },

                                    "+ 添加字段"
                                }
                            }
                        },
                        KeyType::List => rsx! {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "8px",

                                for (idx, val) in list_values.read().iter().enumerate() {
                                    div {
                                        key: "{idx}",
                                        display: "flex",
                                        gap: "8px",
                                        align_items: "center",

                                        span {
                                            color: "#888",
                                            font_size: "12px",
                                            width: "24px",

                                            "{idx}"
                                        }

                                        input {
                                            flex: "1",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            value: "{val.value}",
                                            oninput: {
                                                let mut values = list_values.clone();
                                                move |e| {
                                                    let v = e.value();
                                                    values.write()[idx].value = v;
                                                }
                                            },
                                        }

                                        button {
                                            padding: "4px 8px",
                                            background: "#c53030",
                                            color: "white",
                                            border: "none",
                                            border_radius: "4px",
                                            cursor: "pointer",
                                            font_size: "12px",
                                            onclick: {
                                                let mut values = list_values.clone();
                                                move |_| {
                                                    values.write().remove(idx);
                                                }
                                            },

                                            IconX { size: Some(12) }
                                        }
                                    }
                                }

                                button {
                                    padding: "6px 12px",
                                    background: "#0e639c",
                                    color: "white",
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "12px",
                                    onclick: move |_| {
                                        list_values.write().push(ListValue::default());
                                    },

                                    "+ 添加元素"
                                }
                            }
                        },
                        KeyType::Set => rsx! {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "8px",

                                for (idx, val) in set_values.read().iter().enumerate() {
                                    div {
                                        key: "{idx}",
                                        display: "flex",
                                        gap: "8px",
                                        align_items: "center",

                                        input {
                                            flex: "1",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            value: "{val.value}",
                                            oninput: {
                                                let mut values = set_values.clone();
                                                move |e| {
                                                    let v = e.value();
                                                    values.write()[idx].value = v;
                                                }
                                            },
                                        }

                                        button {
                                            padding: "4px 8px",
                                            background: "#c53030",
                                            color: "white",
                                            border: "none",
                                            border_radius: "4px",
                                            cursor: "pointer",
                                            font_size: "12px",
                                            onclick: {
                                                let mut values = set_values.clone();
                                                move |_| {
                                                    values.write().remove(idx);
                                                }
                                            },

                                            IconX { size: Some(12) }
                                        }
                                    }
                                }

                                button {
                                    padding: "6px 12px",
                                    background: "#0e639c",
                                    color: "white",
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "12px",
                                    onclick: move |_| {
                                        set_values.write().push(SetValue::default());
                                    },

                                    "+ 添加元素"
                                }
                            }
                        },
                        KeyType::ZSet => rsx! {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "8px",

                                for (idx, member) in zset_members.read().iter().enumerate() {
                                    div {
                                        key: "{idx}",
                                        display: "flex",
                                        gap: "8px",
                                        align_items: "center",

                                        input {
                                            width: "80px",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            placeholder: "Score",
                                            value: "{member.score}",
                                            oninput: {
                                                let mut members = zset_members.clone();
                                                move |e| {
                                                    let v = e.value();
                                                    members.write()[idx].score = v;
                                                }
                                            },
                                        }

                                        input {
                                            flex: "1",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            placeholder: "Value",
                                            value: "{member.value}",
                                            oninput: {
                                                let mut members = zset_members.clone();
                                                move |e| {
                                                    let v = e.value();
                                                    members.write()[idx].value = v;
                                                }
                                            },
                                        }

                                        button {
                                            padding: "4px 8px",
                                            background: "#c53030",
                                            color: "white",
                                            border: "none",
                                            border_radius: "4px",
                                            cursor: "pointer",
                                            font_size: "12px",
                                            onclick: {
                                                let mut members = zset_members.clone();
                                                move |_| {
                                                    members.write().remove(idx);
                                                }
                                            },

                                            IconX { size: Some(12) }
                                        }
                                    }
                                }

                                button {
                                    padding: "6px 12px",
                                    background: "#0e639c",
                                    color: "white",
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "12px",
                                    onclick: move |_| {
                                        zset_members.write().push(ZSetMember::default());
                                    },

                                    "+ 添加成员"
                                }
                            }
                        },
                        KeyType::Stream => rsx! {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                gap: "8px",

                                for (idx, entry) in stream_entries.read().iter().enumerate() {
                                    div {
                                        key: "{idx}",
                                        display: "flex",
                                        gap: "8px",
                                        align_items: "center",

                                        input {
                                            width: "120px",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            placeholder: "ID (*)",
                                            value: "{entry.id}",
                                            oninput: {
                                                let mut entries = stream_entries.clone();
                                                move |e| {
                                                    let v = e.value();
                                                    entries.write()[idx].id = v;
                                                }
                                            },
                                        }

                                        input {
                                            flex: "1",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            placeholder: "Field",
                                            value: "{entry.field}",
                                            oninput: {
                                                let mut entries = stream_entries.clone();
                                                move |e| {
                                                    let v = e.value();
                                                    entries.write()[idx].field = v;
                                                }
                                            },
                                        }

                                        input {
                                            flex: "1",
                                            padding: "6px 8px",
                                            background: "#3c3c3c",
                                            border: "1px solid #555",
                                            border_radius: "4px",
                                            color: "white",
                                            font_size: "12px",
                                            placeholder: "Value",
                                            value: "{entry.value}",
                                            oninput: {
                                                let mut entries = stream_entries.clone();
                                                move |e| {
                                                    let v = e.value();
                                                    entries.write()[idx].value = v;
                                                }
                                            },
                                        }

                                        button {
                                            padding: "4px 8px",
                                            background: "#c53030",
                                            color: "white",
                                            border: "none",
                                            border_radius: "4px",
                                            cursor: "pointer",
                                            font_size: "12px",
                                            onclick: {
                                                let mut entries = stream_entries.clone();
                                                move |_| {
                                                    entries.write().remove(idx);
                                                }
                                            },

                                            IconX { size: Some(12) }
                                        }
                                    }
                                }

                                button {
                                    padding: "6px 12px",
                                    background: "#0e639c",
                                    color: "white",
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "12px",
                                    onclick: move |_| {
                                        stream_entries.write().push(StreamEntry::default());
                                    },

                                    "+ 添加条目"
                                }
                            }
                        },
                        KeyType::None => rsx! { div {} },
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "block",
                        color: "#888",
                        font_size: "12px",
                        margin_bottom: "4px",

                        "TTL (秒, -1 表示永不过期)"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        font_size: "13px",
                        box_sizing: "border-box",
                        value: "{ttl}",
                        oninput: move |e| ttl.set(e.value()),
                    }
                }

                div {
                    display: "flex",
                    gap: "8px",

                    button {
                        flex: "1",
                        padding: "8px",
                        background: "#5a5a5a",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        disabled: processing(),
                        onclick: move |_| on_cancel.call(()),

                        "取消"
                    }

                    button {
                        flex: "1",
                        padding: "8px",
                        background: "#0e639c",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        disabled: processing(),
                        onclick: move |_| {
                            let key = key_name.read().trim().to_string();
                            if key.is_empty() {
                                error_msg.set(Some("Key 名称不能为空".to_string()));
                                return;
                            }

                            let pool = connection_pool.clone();
                            let on_save = on_save.clone();
                            let kt = key_type();
                            let ttl_val = ttl.read().parse::<i64>().unwrap_or(-1);
                            let string_val = string_value.read().clone();
                            let hash_f = hash_fields.read().clone();
                            let list_v = list_values.read().clone();
                            let set_v = set_values.read().clone();
                            let zset_m = zset_members.read().clone();
                            let stream_e = stream_entries.read().clone();

                            spawn(async move {
                                processing.set(true);
                                error_msg.set(None);

                                let result = match kt {
                                    KeyType::String => {
                                        pool.set_string_value(&key, &string_val).await
                                    }
                                    KeyType::Hash => {
                                        pool.set_hash_values(&key, hash_f).await
                                    }
                                    KeyType::List => {
                                        pool.set_list_values(&key, list_v).await
                                    }
                                    KeyType::Set => {
                                        pool.set_set_values(&key, set_v).await
                                    }
                                    KeyType::ZSet => {
                                        pool.set_zset_members(&key, zset_m).await
                                    }
                                    KeyType::Stream => {
                                        pool.add_stream_entries(&key, stream_e).await
                                    }
                                    KeyType::None => {
                                        Err(crate::connection::ConnectionError::ConnectionFailed(
                                            "Invalid key type".to_string(),
                                        ))
                                    }
                                };

                                match result {
                                    Ok(_) => {
                                        if ttl_val > 0 {
                                            let _ = pool.set_ttl(&key, ttl_val).await;
                                        }
                                        processing.set(false);
                                        on_save.call(key);
                                    }
                                    Err(e) => {
                                        error_msg.set(Some(e.to_string()));
                                        processing.set(false);
                                    }
                                }
                            });
                        },

                        if processing() {
                            "保存中..."
                        } else {
                            "✓ 保存"
                        }
                    }
                }
            }
        }
    }
}
