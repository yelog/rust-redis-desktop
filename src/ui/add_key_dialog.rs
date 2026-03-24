use crate::connection::ConnectionPool;
use crate::redis::KeyType;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
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
    colors: ThemeColors,
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

    let input_style = format!(
        "width: 100%; padding: 8px 12px; background: {}; border: 1px solid {}; border_radius: 4px; color: {}; font_size: 13px; box_sizing: border-box;",
        colors.background_tertiary, colors.border, colors.text
    );

    let button_primary_style = format!(
        "padding: 8px; background: {}; color: {}; border: none; border_radius: 4px; cursor: pointer;",
        colors.primary, colors.primary_text
    );

    let button_danger_style = "padding: 4px 8px; background: #c53030; color: white; border: none; border_radius: 4px; cursor: pointer; font_size: 12px;";

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "500px".to_string(),
            max_height: "80vh".to_string(),

            div {
                position: "relative",

                button {
                    position: "absolute",
                    top: "-8px",
                    right: "-8px",
                    z_index: "10",
                    padding: "4px",
                    background: "transparent",
                    border: "none",
                    cursor: "pointer",
                    color: "{colors.text_secondary}",
                    onclick: move |_| on_cancel.call(()),

                    IconX { size: Some(18) }
                }

                h3 {
                    color: "{colors.accent}",
                    margin_bottom: "16px",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    font_size: "18px",

                    "➕ 新增 Key"
                }

            if let Some(err) = error_msg() {
                div {
                    color: "{colors.error}",
                    background: "{colors.error_bg}",
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
                    color: "{colors.text_secondary}",
                    font_size: "12px",
                    margin_bottom: "4px",

                    "Key 名称"
                }

                input {
                    width: "100%",
                    padding: "8px 12px",
                    background: "{colors.background_tertiary}",
                    border: "1px solid {colors.border}",
                    border_radius: "4px",
                    color: "{colors.text}",
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
                    color: "{colors.text_secondary}",
                    font_size: "12px",
                    margin_bottom: "8px",

                    "类型"
                }

                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "8px",

                    for type_name in ["String", "Hash", "List", "Set", "ZSet", "Stream"] {
                        {
                            let kt = KeyType::from(type_name.to_string());
                            let is_selected = key_type() == kt;
                            let bg = if is_selected { colors.primary.clone() } else { colors.background_tertiary.clone() };
                            let border_color = if is_selected { colors.primary.clone() } else { colors.border.clone() };
                            let text_color = if is_selected { colors.primary_text.clone() } else { colors.text.clone() };
                            rsx! {
                                div {
                                    key: "{type_name}",
                                    padding: "6px 14px",
                                    background: "{bg}",
                                    border: "1px solid {border_color}",
                                    border_radius: "16px",
                                    color: "{text_color}",
                                    font_size: "13px",
                                    cursor: "pointer",
                                    user_select: "none",
                                    onclick: {
                                        let kt = kt.clone();
                                        move |_| key_type.set(kt.clone())
                                    },

                                    "{type_name}"
                                }
                            }
                        }
                    }
                }
            }

            div {
                margin_bottom: "16px",
                flex: "1",
                width: "100%",
                box_sizing: "border-box",
                overflow_y: "auto",
                overflow_x: "hidden",

                label {
                    display: "block",
                    color: "{colors.text_secondary}",
                    font_size: "12px",
                    margin_bottom: "4px",

                    "Value"
                }

                match key_type() {
                    KeyType::String => rsx! {
                        textarea {
                            width: "100%",
                            min_width: "0",
                            height: "150px",
                            padding: "8px 12px",
                            background: "{colors.background_tertiary}",
                            border: "1px solid {colors.border}",
                            border_radius: "4px",
                            color: "{colors.text}",
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
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
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
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
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
                                        background: "{colors.error}",
                                        color: "{colors.primary_text}",
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
                                background: "{colors.primary}",
                                color: "{colors.primary_text}",
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
                                        color: "{colors.text_secondary}",
                                        font_size: "12px",
                                        width: "24px",

                                        "{idx}"
                                    }

                                    input {
                                        flex: "1",
                                        padding: "6px 8px",
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
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
                                        background: "{colors.error}",
                                        color: "{colors.primary_text}",
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
                                background: "{colors.primary}",
                                color: "{colors.primary_text}",
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
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
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
                                        background: "{colors.error}",
                                        color: "{colors.primary_text}",
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
                                background: "{colors.primary}",
                                color: "{colors.primary_text}",
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
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
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
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
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
                                        background: "{colors.error}",
                                        color: "{colors.primary_text}",
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
                                background: "{colors.primary}",
                                color: "{colors.primary_text}",
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
                            width: "100%",
                            box_sizing: "border-box",

                            for (idx, entry) in stream_entries.read().iter().enumerate() {
                                div {
                                    key: "{idx}",
                                    display: "flex",
                                    gap: "8px",
                                    align_items: "center",
                                    min_width: "0",

                                    input {
                                        width: "100px",
                                        min_width: "0",
                                        padding: "6px 8px",
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
                                        font_size: "12px",
                                        placeholder: "ID (*)",
                                        box_sizing: "border-box",
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
                                        min_width: "0",
                                        padding: "6px 8px",
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
                                        font_size: "12px",
                                        placeholder: "Field",
                                        box_sizing: "border-box",
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
                                        min_width: "0",
                                        padding: "6px 8px",
                                        background: "{colors.background_tertiary}",
                                        border: "1px solid {colors.border}",
                                        border_radius: "4px",
                                        color: "{colors.text}",
                                        font_size: "12px",
                                        placeholder: "Value",
                                        box_sizing: "border-box",
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
                                        background: "{colors.error}",
                                        color: "{colors.primary_text}",
                                        border: "none",
                                        border_radius: "4px",
                                        cursor: "pointer",
                                        font_size: "12px",
                                        flex_shrink: "0",
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
                                background: "{colors.primary}",
                                color: "{colors.primary_text}",
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
                    color: "{colors.text_secondary}",
                    font_size: "12px",
                    margin_bottom: "4px",

                    "TTL (秒, -1 表示永不过期)"
                }

                input {
                    width: "100%",
                    padding: "8px 12px",
                    background: "{colors.background_tertiary}",
                    border: "1px solid {colors.border}",
                    border_radius: "4px",
                    color: "{colors.text}",
                    font_size: "13px",
                    box_sizing: "border-box",
                    value: "{ttl}",
                    oninput: move |e| ttl.set(e.value()),
                }
            }

            div {
                button {
                    width: "100%",
                    padding: "8px",
                    background: "{colors.primary}",
                    color: "{colors.primary_text}",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
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
