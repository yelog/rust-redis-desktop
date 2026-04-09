use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::theme::{
    COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_PRIMARY, COLOR_TEXT,
    COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq)]
pub struct PubSubMessage {
    pub channel: String,
    pub payload: String,
    pub timestamp: u64,
}

#[component]
pub fn PubSubPanel(connection_pool: ConnectionPool) -> Element {
    let mut messages = use_signal(Vec::<PubSubMessage>::new);
    let mut subscribed_channels = use_signal(Vec::<String>::new);
    let mut channel_input = use_signal(String::new);
    let mut message_input = use_signal(String::new);
    let mut publish_channel = use_signal(String::new);
    let mut is_subscribing = use_signal(|| false);
    let mut status_message = use_signal(|| None::<String>);
    let mut subscribe_handle: Signal<Option<Arc<AtomicBool>>> = use_signal(|| None);
    let i18n = use_i18n();
    let subscribe_button_label = if is_subscribing() {
        i18n.read().t("Subscribing...")
    } else {
        i18n.read().t("Subscribe")
    };

    let _stop_subscription = move || {
        if let Some(handle) = subscribe_handle() {
            handle.store(false, Ordering::SeqCst);
            subscribe_handle.set(None);
        }
        is_subscribing.set(false);
    };

    let add_subscription = {
        let pool = connection_pool.clone();
        let mut subscribed_channels = subscribed_channels.clone();
        let mut messages = messages.clone();
        let mut is_subscribing = is_subscribing.clone();
        let mut subscribe_handle = subscribe_handle.clone();
        move |_| {
            let channel = channel_input();
            if channel.is_empty() {
                return;
            }

            let running = Arc::new(AtomicBool::new(true));
            subscribe_handle.set(Some(running.clone()));
            is_subscribing.set(true);

            if !subscribed_channels.read().contains(&channel) {
                subscribed_channels.write().push(channel.clone());
            }

            let pool = pool.clone();
            let channel_clone = channel.clone();
            let mut messages = messages.clone();

            let i18n = i18n.clone();
            spawn(async move {
                let mut connection = pool.connection.lock().await;
                if let Some(ref mut conn) = *connection {
                    let mut subscribe_cmd = redis::cmd("SUBSCRIBE");
                    subscribe_cmd.arg(&channel_clone);

                    if let Err(e) = conn.execute_cmd::<redis::Value>(&mut subscribe_cmd).await {
                        messages.write().push(PubSubMessage {
                            channel: "_system".to_string(),
                            payload: format!("{}: {}", i18n.read().t("Subscribe failed"), e),
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        });
                        return;
                    }
                }
            });

            channel_input.set(String::new());
        }
    };

    let publish_message = {
        let pool = connection_pool.clone();
        let mut status_message = status_message.clone();
        move |_| {
            let channel = publish_channel();
            let message = message_input();

            if channel.is_empty() || message.is_empty() {
                status_message.set(Some(i18n.read().t("Enter a channel and message")));
                return;
            }

            let pool = pool.clone();
            let channel = channel.clone();
            let message = message.clone();
            let mut status_message = status_message.clone();
            let i18n = i18n.clone();

            spawn(async move {
                let mut connection = pool.connection.lock().await;
                if let Some(ref mut conn) = *connection {
                    match conn
                        .execute_cmd::<i32>(&mut redis::cmd("PUBLISH").arg(&channel).arg(&message))
                        .await
                    {
                        Ok(_) => {
                            status_message.set(Some(i18n.read().t("Message published")));
                        }
                        Err(e) => {
                            status_message.set(Some(format!(
                                "{}: {}",
                                i18n.read().t("Publish failed"),
                                e
                            )));
                        }
                    }
                }
            });

            message_input.set(String::new());
        }
    };

    let clear_messages = move |_| {
        messages.write().clear();
    };

    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background: COLOR_BG,
            padding: "16px",
            gap: "16px",

            div {
                display: "flex",
                gap: "16px",

                div {
                    flex: "1",
                    display: "flex",
                    flex_direction: "column",
                    gap: "8px",

                    label {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        font_weight: "500",
                        {i18n.read().t("Subscribe Channel")}
                    }

                    div {
                        display: "flex",
                        gap: "8px",

                        input {
                            flex: "1",
                            padding: "8px 12px",
                            background: COLOR_BG_TERTIARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "4px",
                            color: COLOR_TEXT,
                            font_size: "13px",
                            placeholder: i18n.read().t("Enter channel name"),
                            value: "{channel_input}",
                            oninput: move |e| channel_input.set(e.value()),
                        }

                        button {
                            padding: "8px 16px",
                            background: COLOR_PRIMARY,
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: add_subscription,
                            disabled: is_subscribing(),
                            {subscribe_button_label}
                        }
                    }
                }
            }

            div {
                display: "flex",
                gap: "16px",

                div {
                    flex: "1",
                    display: "flex",
                    flex_direction: "column",
                    gap: "8px",

                    label {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        font_weight: "500",
                        {i18n.read().t("Publish Message")}
                    }

                    div {
                        display: "flex",
                        gap: "8px",

                        input {
                            flex: "1",
                            padding: "8px 12px",
                            background: COLOR_BG_TERTIARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "4px",
                            color: COLOR_TEXT,
                            font_size: "13px",
                            placeholder: i18n.read().t("Channel name"),
                            value: "{publish_channel}",
                            oninput: move |e| publish_channel.set(e.value()),
                        }

                        input {
                            flex: "2",
                            padding: "8px 12px",
                            background: COLOR_BG_TERTIARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "4px",
                            color: COLOR_TEXT,
                            font_size: "13px",
                            placeholder: i18n.read().t("Message content"),
                            value: "{message_input}",
                            oninput: move |e| message_input.set(e.value()),
                        }

                        button {
                            padding: "8px 16px",
                            background: COLOR_PRIMARY,
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: publish_message,
                            {i18n.read().t("Publish")}
                        }
                    }
                }
            }

            if let Some(msg) = status_message() {
                div {
                    padding: "8px 12px",
                    background: COLOR_BG_SECONDARY,
                    border_radius: "4px",
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",
                    "{msg}"
                }
            }

            div {
                display: "flex",
                gap: "8px",
                align_items: "center",

                label {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",
                    font_weight: "500",
                    {i18n.read().t("Subscribed Channels:")}
                }

                for channel in subscribed_channels() {
                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "4px",
                        padding: "4px 8px",
                        background: COLOR_BG_TERTIARY,
                        border_radius: "4px",
                        font_size: "12px",

                        "{channel}"

                        button {
                            padding: "2px 6px",
                            background: "transparent",
                            border: "none",
                            color: COLOR_TEXT_SUBTLE,
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: {
                                let channel = channel.clone();
                                let pool = connection_pool.clone();
                                let mut subscribed_channels = subscribed_channels.clone();
                                move |_| {
                                    let pool = pool.clone();
                                    let channel = channel.clone();
                                    let mut subscribed_channels = subscribed_channels.clone();
                                    spawn(async move {
                                        let mut connection = pool.connection.lock().await;
                                        if let Some(ref mut conn) = *connection {
                                            let _ = conn.execute_cmd::<redis::Value>(&mut redis::cmd("UNSUBSCRIBE").arg(&channel)).await;
                                        }
                                        subscribed_channels.write().retain(|c| c != &channel);
                                    });
                                }
                            },
                            "×"
                        }
                    }
                }
            }

            div {
                flex: "1",
                display: "flex",
                flex_direction: "column",
                gap: "8px",
                overflow: "hidden",

                div {
                    display: "flex",
                    justify_content: "space-between",
                    align_items: "center",

                    label {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        font_weight: "500",
                        {i18n.read().t("Messages")}
                    }

                    button {
                        padding: "4px 12px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT_SECONDARY,
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: clear_messages,
                        {i18n.read().t("Clear")}
                    }
                }

                div {
                    flex: "1",
                    overflow_y: "auto",
                    background: COLOR_BG_SECONDARY,
                    border_radius: "4px",
                    padding: "8px",

                    if messages().is_empty() {
                        div {
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "13px",
                            text_align: "center",
                            padding: "20px",
                            {i18n.read().t("No messages")}
                        }
                    } else {
                        for msg in messages() {
                            div {
                                padding: "8px",
                                border_bottom: "1px solid {COLOR_BORDER}",
                                font_size: "13px",

                                div {
                                    display: "flex",
                                    gap: "8px",
                                    margin_bottom: "4px",

                                    span {
                                        color: COLOR_PRIMARY,
                                        font_weight: "500",
                                        "[{msg.channel}]"
                                    }

                                    span {
                                        color: COLOR_TEXT_SUBTLE,
                                        font_size: "11px",
                                        "{msg.timestamp}"
                                    }
                                }

                                div {
                                    color: COLOR_TEXT,
                                    "{msg.payload}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
