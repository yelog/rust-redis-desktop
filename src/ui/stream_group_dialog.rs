use crate::connection::ConnectionPool;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::icons::{IconPlus, IconTrash, IconUsers};
use dioxus::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamGroupInfo {
    pub name: String,
    pub consumers: i64,
    pub pending: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamConsumerInfo {
    pub name: String,
    pub pending: i64,
    pub idle: i64,
}

fn parse_groups(value: &redis::Value) -> Vec<StreamGroupInfo> {
    let mut groups = Vec::new();
    if let redis::Value::Array(arr) = value {
        for group_data in arr {
            if let redis::Value::Array(fields) = group_data {
                let mut info = StreamGroupInfo::default();
                let mut i = 0;
                while i + 1 < fields.len() {
                    if let (redis::Value::BulkString(key), redis::Value::BulkString(val)) =
                        (&fields[i], &fields[i + 1])
                    {
                        match String::from_utf8_lossy(key).as_ref() {
                            "name" => info.name = String::from_utf8_lossy(val).to_string(),
                            "consumers" => {
                                info.consumers = String::from_utf8_lossy(val).parse().unwrap_or(0)
                            }
                            "pending" => {
                                info.pending = String::from_utf8_lossy(val).parse().unwrap_or(0)
                            }
                            _ => {}
                        }
                    }
                    i += 2;
                }
                if !info.name.is_empty() {
                    groups.push(info);
                }
            }
        }
    }
    groups
}

fn parse_consumers(value: &redis::Value) -> Vec<StreamConsumerInfo> {
    let mut consumers = Vec::new();
    if let redis::Value::Array(arr) = value {
        for consumer_data in arr {
            if let redis::Value::Array(fields) = consumer_data {
                let mut info = StreamConsumerInfo::default();
                let mut i = 0;
                while i + 1 < fields.len() {
                    if let (redis::Value::BulkString(key), redis::Value::BulkString(val)) =
                        (&fields[i], &fields[i + 1])
                    {
                        match String::from_utf8_lossy(key).as_ref() {
                            "name" => info.name = String::from_utf8_lossy(val).to_string(),
                            "pending" => {
                                info.pending = String::from_utf8_lossy(val).parse().unwrap_or(0)
                            }
                            "idle" => info.idle = String::from_utf8_lossy(val).parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                    i += 2;
                }
                if !info.name.is_empty() {
                    consumers.push(info);
                }
            }
        }
    }
    consumers
}

fn format_idle(ms: i64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", ms as f64 / 60000.0)
    }
}

#[component]
pub fn StreamGroupDialog(
    connection_pool: ConnectionPool,
    stream_key: String,
    colors: ThemeColors,
    on_close: EventHandler<()>,
) -> Element {
    let mut groups = use_signal(Vec::<StreamGroupInfo>::new);
    let mut consumers = use_signal(Vec::<StreamConsumerInfo>::new);
    let mut selected_group = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut show_create = use_signal(|| false);
    let mut new_name = use_signal(String::new);
    let mut new_id = use_signal(|| "$".to_string());
    let mut mkstream = use_signal(|| false);
    let mut action = use_signal(|| None::<StreamAction>);

    #[derive(Clone)]
    enum StreamAction {
        DeleteGroup(String),
        DeleteConsumer { group: String, consumer: String },
        CreateGroup { name: String, id: String, mkstream: bool },
    }

    let pool_clone = connection_pool.clone();
    let key_clone = stream_key.clone();

    use_effect(move || {
        let pool = pool_clone.clone();
        let key = key_clone.clone();
        loading.set(true);
        spawn(async move {
            if let Ok(value) = pool.stream_get_groups_raw(&key).await {
                groups.set(parse_groups(&value));
            }
            loading.set(false);
        });
    });

    let grp = selected_group();
    let pool_for_consumers = connection_pool.clone();
    let key_for_consumers = stream_key.clone();
    use_effect(move || {
        if let Some(g) = grp.clone() {
            let pool = pool_for_consumers.clone();
            let key = key_for_consumers.clone();
            spawn(async move {
                if let Ok(value) = pool.stream_get_consumers_raw(&key, &g).await {
                    consumers.set(parse_consumers(&value));
                }
            });
        }
    });

    let act = action();
    let pool_for_action = connection_pool.clone();
    let key_for_action = stream_key.clone();
    let sel_grp = selected_group();
    use_effect(move || {
        if let Some(a) = act.clone() {
            let pool = pool_for_action.clone();
            let key = key_for_action.clone();
            let sel = sel_grp.clone();
            spawn(async move {
                match a {
                    StreamAction::DeleteGroup(name) => {
                        let _ = pool.stream_destroy_group(&key, &name).await;
                        if let Ok(value) = pool.stream_get_groups_raw(&key).await {
                            groups.set(parse_groups(&value));
                        }
                        if selected_group() == Some(name) {
                            selected_group.set(None);
                            consumers.set(Vec::new());
                        }
                    }
                    StreamAction::DeleteConsumer { group, consumer } => {
                        let _ = pool.stream_delete_consumer(&key, &group, &consumer).await;
                        if let Ok(value) = pool.stream_get_consumers_raw(&key, &group).await {
                            consumers.set(parse_consumers(&value));
                        }
                    }
                    StreamAction::CreateGroup { name, id, mkstream: mks } => {
                        let _ = pool.stream_create_group(&key, &name, &id, mks).await;
                        if let Ok(value) = pool.stream_get_groups_raw(&key).await {
                            groups.set(parse_groups(&value));
                        }
                    }
                }
                action.set(None);
            });
        }
    });

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "650px".to_string(),
            max_height: "80vh".to_string(),

            h3 {
                color: "{colors.text}",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "8px",
                font_size: "18px",

                IconUsers { size: Some(20) }
                "Consumer Groups"
            }

            div {
                margin_bottom: "8px",
                color: "{colors.text_secondary}",
                font_size: "12px",
                font_family: "monospace",

                "Stream: {stream_key}"
            }

            if loading() {
                div {
                    padding: "20px",
                    text_align: "center",
                    color: "{colors.text_secondary}",
                    "Loading..."
                }
            } else {
                div {
                    display: "flex",
                    gap: "16px",
                    height: "350px",

                    div {
                        flex: "1",
                        display: "flex",
                        flex_direction: "column",
                        border: "1px solid {colors.border}",
                        border_radius: "6px",
                        overflow: "hidden",

                        div {
                            padding: "10px 12px",
                            background: "{colors.background_tertiary}",
                            border_bottom: "1px solid {colors.border}",
                            display: "flex",
                            justify_content: "space_between",
                            align_items: "center",

                            span {
                                color: "{colors.text}",
                                font_size: "13px",
                                font_weight: "600",
                                "Groups ({groups.read().len()})"
                            }

                            button {
                                padding: "4px 8px",
                                background: "{colors.primary}",
                                color: "{colors.primary_text}",
                                border: "none",
                                border_radius: "4px",
                                cursor: "pointer",
                                font_size: "11px",
                                onclick: move |_| show_create.set(true),

                                IconPlus { size: Some(12) }
                                "New"
                            }
                        }

                        div {
                            flex: "1",
                            overflow_y: "auto",

                            if groups.read().is_empty() {
                                div {
                                    padding: "20px",
                                    text_align: "center",
                                    color: "{colors.text_secondary}",
                                    font_size: "13px",
                                    "No consumer groups"
                                }
                            } else {
                                for group in groups.read().iter() {
                                    {
                                        let is_sel = selected_group() == Some(group.name.clone());
                                        let name = group.name.clone();
                                        let del_name = group.name.clone();
                                        rsx! {
                                            div {
                                                padding: "10px 12px",
                                                border_bottom: "1px solid {colors.border}",
                                                cursor: "pointer",
                                                background: if is_sel { "{colors.background_tertiary}" } else { "transparent" },
                                                onclick: move |_| selected_group.set(Some(name.clone())),

                                                div {
                                                    display: "flex",
                                                    justify_content: "space_between",
                                                    align_items: "center",

                                                    span {
                                                        color: "{colors.text}",
                                                        font_size: "13px",
                                                        font_weight: "500",
                                                        "{group.name}"
                                                    }

                                                    button {
                                                        padding: "2px 6px",
                                                        background: "transparent",
                                                        color: "{colors.error}",
                                                        border: "1px solid {colors.error}",
                                                        border_radius: "3px",
                                                        cursor: "pointer",
                                                        font_size: "10px",
                                                        onclick: move |e| {
                                                            e.stop_propagation();
                                                            action.set(Some(StreamAction::DeleteGroup(del_name.clone())));
                                                        },
                                                        "Delete"
                                                    }
                                                }

                                                div {
                                                    margin_top: "6px",
                                                    display: "flex",
                                                    gap: "16px",

                                                    span {
                                                        color: "{colors.text_secondary}",
                                                        font_size: "11px",
                                                        "Consumers: {group.consumers}"
                                                    }
                                                    span {
                                                        color: "{colors.text_secondary}",
                                                        font_size: "11px",
                                                        "Pending: {group.pending}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        flex: "1",
                        display: "flex",
                        flex_direction: "column",
                        border: "1px solid {colors.border}",
                        border_radius: "6px",
                        overflow: "hidden",

                        div {
                            padding: "10px 12px",
                            background: "{colors.background_tertiary}",
                            border_bottom: "1px solid {colors.border}",

                            span {
                                color: "{colors.text}",
                                font_size: "13px",
                                font_weight: "600",
                                if let Some(g) = selected_group() {
                                    "Consumers - {g}"
                                } else {
                                    "Consumers"
                                }
                            }
                        }

                        div {
                            flex: "1",
                            overflow_y: "auto",

                            if selected_group().is_none() {
                                div {
                                    padding: "20px",
                                    text_align: "center",
                                    color: "{colors.text_secondary}",
                                    font_size: "13px",
                                    "Select a group"
                                }
                            } else if consumers.read().is_empty() {
                                div {
                                    padding: "20px",
                                    text_align: "center",
                                    color: "{colors.text_secondary}",
                                    font_size: "13px",
                                    "No consumers"
                                }
                            } else {
                                for consumer in consumers.read().iter() {
                                    {
                                        let cn = consumer.name.clone();
                                        let grp_name = selected_group().clone().unwrap_or_default();
                                        rsx! {
                                            div {
                                                padding: "10px 12px",
                                                border_bottom: "1px solid {colors.border}",

                                                div {
                                                    display: "flex",
                                                    justify_content: "space_between",
                                                    align_items: "center",

                                                    span {
                                                        color: "{colors.text}",
                                                        font_size: "13px",
                                                        "{consumer.name}"
                                                    }

                                                    button {
                                                        padding: "2px 6px",
                                                        background: "transparent",
                                                        color: "{colors.error}",
                                                        border: "1px solid {colors.error}",
                                                        border_radius: "3px",
                                                        cursor: "pointer",
                                                        font_size: "10px",
                                                        onclick: move |_| {
                                                            action.set(Some(StreamAction::DeleteConsumer {
                                                                group: grp_name.clone(),
                                                                consumer: cn.clone(),
                                                            }));
                                                        },
                                                        "Delete"
                                                    }
                                                }

                                                div {
                                                    margin_top: "6px",
                                                    display: "flex",
                                                    gap: "16px",

                                                    span {
                                                        color: "{colors.text_secondary}",
                                                        font_size: "11px",
                                                        "Pending: {consumer.pending}"
                                                    }
                                                    span {
                                                        color: "{colors.text_secondary}",
                                                        font_size: "11px",
                                                        "Idle: {format_idle(consumer.idle)}"
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

        if show_create() {
            AnimatedDialog {
                is_open: true,
                on_close: move |_| show_create.set(false),
                colors,
                width: "400px".to_string(),
                max_height: "auto".to_string(),

                h3 {
                    color: "{colors.text}",
                    margin_bottom: "16px",
                    font_size: "16px",
                    "Create Consumer Group"
                }

                div {
                    margin_bottom: "12px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "4px",
                        "Group Name"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        box_sizing: "border_box",
                        value: "{new_name}",
                        oninput: move |e| new_name.set(e.value()),
                    }
                }

                div {
                    margin_bottom: "12px",

                    label {
                        display: "block",
                        color: "{colors.text_secondary}",
                        font_size: "12px",
                        margin_bottom: "4px",
                        "Start ID ($ = last, 0 = beginning)"
                    }

                    input {
                        width: "100%",
                        padding: "8px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "13px",
                        font_family: "monospace",
                        box_sizing: "border_box",
                        value: "{new_id}",
                        oninput: move |e| new_id.set(e.value()),
                    }
                }

                div {
                    margin_bottom: "16px",

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        color: "{colors.text}",
                        font_size: "13px",
                        cursor: "pointer",

                        input {
                            r#type: "checkbox",
                            checked: mkstream(),
                            onchange: move |e| mkstream.set(e.checked()),
                        }
                        "Create stream if not exists (MKSTREAM)"
                    }
                }

                div {
                    display: "flex",
                    gap: "8px",

                    button {
                        flex: "1",
                        padding: "10px",
                        background: "{colors.primary}",
                        color: "{colors.primary_text}",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        disabled: new_name.read().is_empty(),
                        onclick: move |_| {
                            show_create.set(false);
                            action.set(Some(StreamAction::CreateGroup {
                                name: new_name(),
                                id: new_id(),
                                mkstream: mkstream(),
                            }));
                            new_name.set(String::new());
                            new_id.set("$".to_string());
                        },
                        "Create"
                    }

                    button {
                        flex: "1",
                        padding: "10px",
                        background: "{colors.background_tertiary}",
                        color: "{colors.text}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| show_create.set(false),
                        "Cancel"
                    }
                }
            }
        }
    }
}