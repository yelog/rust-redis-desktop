use crate::connection::ConnectionPool;
use crate::theme::{COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_PRIMARY, COLOR_TEXT, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE, COLOR_SUCCESS, COLOR_ERROR};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SavedScript {
    pub name: String,
    pub script: String,
}

fn format_redis_value(value: &redis::Value) -> String {
    match value {
        redis::Value::Nil => "(nil)".to_string(),
        redis::Value::Int(i) => format!("(integer) {}", i),
        redis::Value::BulkString(data) => match String::from_utf8(data.clone()) {
            Ok(s) => format!("\"{}\"", s),
            Err(_) => format!("{:?}", data),
        },
        redis::Value::Array(items) => {
            if items.is_empty() {
                "(empty array)".to_string()
            } else {
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| format!("{}) {}", i + 1, format_redis_value(item)))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        redis::Value::SimpleString(s) => s.clone(),
        redis::Value::Okay => "OK".to_string(),
        _ => format!("{:?}", value),
    }
}

#[component]
pub fn ScriptPanel(connection_pool: ConnectionPool) -> Element {
    let mut script_content = use_signal(String::new);
    let mut keys_input = use_signal(String::new);
    let mut args_input = use_signal(String::new);
    let mut result_output = use_signal(String::new);
    let mut status_message = use_signal(|| None::<String>);
    let mut is_executing = use_signal(|| false);
    let mut saved_scripts = use_signal(|| {
        let saved: HashMap<String, SavedScript> = HashMap::new();
        saved
    });
    let mut script_name = use_signal(String::new);

    let execute_script = {
        let pool = connection_pool.clone();
        move |_| {
            let script = script_content();
            if script.is_empty() {
                status_message.set(Some("请输入脚本内容".to_string()));
                return;
            }

            is_executing.set(true);
            status_message.set(None);

            let pool = pool.clone();
            let script = script.clone();
            let keys: Vec<String> = keys_input().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            let args: Vec<String> = args_input().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            let mut result_output = result_output.clone();
            let mut status_message = status_message.clone();
            let mut is_executing = is_executing.clone();

            spawn(async move {
                match pool.eval_script(&script, &keys, &args).await {
                    Ok(value) => {
                        result_output.set(format_redis_value(&value));
                        status_message.set(Some("执行成功".to_string()));
                    }
                    Err(e) => {
                        result_output.set(String::new());
                        status_message.set(Some(format!("执行失败: {}", e)));
                    }
                }
                is_executing.set(false);
            });
        }
    };

    let load_script = {
        let pool = connection_pool.clone();
        move |_| {
            let script = script_content();
            if script.is_empty() {
                status_message.set(Some("请输入脚本内容".to_string()));
                return;
            }

            let pool = pool.clone();
            let script = script.clone();
            let mut result_output = result_output.clone();
            let mut status_message = status_message.clone();

            spawn(async move {
                match pool.script_load(&script).await {
                    Ok(sha) => {
                        result_output.set(format!("脚本已加载，SHA: {}", sha));
                        status_message.set(Some("脚本加载成功".to_string()));
                    }
                    Err(e) => {
                        status_message.set(Some(format!("加载失败: {}", e)));
                    }
                }
            });
        }
    };

    let flush_scripts = {
        let pool = connection_pool.clone();
        move |_| {
            let pool = pool.clone();
            let mut status_message = status_message.clone();

            spawn(async move {
                match pool.script_flush().await {
                    Ok(_) => {
                        status_message.set(Some("脚本缓存已清空".to_string()));
                    }
                    Err(e) => {
                        status_message.set(Some(format!("清空失败: {}", e)));
                    }
                }
            });
        }
    };

    let save_script = move |_| {
        let name = script_name();
        let script = script_content();
        
        if name.is_empty() || script.is_empty() {
            status_message.set(Some("请输入脚本名称和内容".to_string()));
            return;
        }

        saved_scripts.write().insert(name.clone(), SavedScript {
            name: name.clone(),
            script,
        });
        script_name.set(String::new());
        status_message.set(Some(format!("脚本 '{}' 已保存", name)));
    };

    let mut load_saved_script = move |name: String| {
        if let Some(saved) = saved_scripts.read().get(&name) {
            script_content.set(saved.script.clone());
            script_name.set(name);
        }
    };

    let mut delete_saved_script = move |name: String| {
        saved_scripts.write().remove(&name);
    };

    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background: COLOR_BG,
            padding: "16px",
            gap: "12px",

            div {
                display: "flex",
                gap: "12px",

                div {
                    flex: "2",
                    display: "flex",
                    flex_direction: "column",
                    gap: "8px",

                    label {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        font_weight: "500",
                        "Lua 脚本"
                    }

                    textarea {
                        width: "100%",
                        height: "200px",
                        padding: "12px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        font_family: "monospace",
                        box_sizing: "border_box",
                        resize: "vertical",
                        value: "{script_content}",
                        oninput: move |e| script_content.set(e.value()),
                        placeholder: "return redis.call('GET', KEYS[1])",
                    }
                }

                div {
                    flex: "1",
                    display: "flex",
                    flex_direction: "column",
                    gap: "8px",

                    label {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        font_weight: "500",
                        "已保存脚本"
                    }

                    div {
                        flex: "1",
                        overflow_y: "auto",
                        background: COLOR_BG_SECONDARY,
                        border_radius: "4px",
                        padding: "8px",

                        if saved_scripts.read().is_empty() {
                            div {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "12px",
                                text_align: "center",
                                padding: "20px",
                                "暂无保存的脚本"
                            }
                        } else {
                            for (name, _) in saved_scripts.read().clone() {
                                div {
                                    display: "flex",
                                    align_items: "center",
                                    gap: "8px",
                                    padding: "6px 8px",
                                    margin_bottom: "4px",
                                    background: COLOR_BG_TERTIARY,
                                    border_radius: "4px",
                                    font_size: "12px",

                                    span {
                                        flex: "1",
                                        color: COLOR_TEXT,
                                        overflow: "hidden",
                                        text_overflow: "ellipsis",
                                        white_space: "nowrap",
                                        "{name}"
                                    }

                                    button {
                                        padding: "2px 6px",
                                        background: "transparent",
                                        border: "none",
                                        color: COLOR_TEXT_SECONDARY,
                                        cursor: "pointer",
                                        font_size: "11px",
                                        onclick: {
                                            let name = name.clone();
                                            move |_| load_saved_script(name.clone())
                                        },
                                        "加载"
                                    }

                                    button {
                                        padding: "2px 6px",
                                        background: "transparent",
                                        border: "none",
                                        color: COLOR_ERROR,
                                        cursor: "pointer",
                                        font_size: "11px",
                                        onclick: {
                                            let name = name.clone();
                                            move |_| delete_saved_script(name.clone())
                                        },
                                        "删除"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                display: "flex",
                gap: "8px",

                div {
                    flex: "1",

                    label {
                        display: "block",
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "11px",
                        margin_bottom: "4px",
                        "KEYS (逗号分隔)"
                    }

                    input {
                        width: "100%",
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        box_sizing: "border_box",
                        value: "{keys_input}",
                        oninput: move |e| keys_input.set(e.value()),
                        placeholder: "key1,key2,key3",
                    }
                }

                div {
                    flex: "1",

                    label {
                        display: "block",
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "11px",
                        margin_bottom: "4px",
                        "ARGV (逗号分隔)"
                    }

                    input {
                        width: "100%",
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        box_sizing: "border_box",
                        value: "{args_input}",
                        oninput: move |e| args_input.set(e.value()),
                        placeholder: "arg1,arg2,arg3",
                    }
                }

                div {
                    flex: "1",

                    label {
                        display: "block",
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "11px",
                        margin_bottom: "4px",
                        "脚本名称"
                    }

                    input {
                        width: "100%",
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        box_sizing: "border_box",
                        value: "{script_name}",
                        oninput: move |e| script_name.set(e.value()),
                        placeholder: "保存脚本名称",
                    }
                }
            }

            div {
                display: "flex",
                gap: "8px",

                button {
                    padding: "8px 16px",
                    background: COLOR_PRIMARY,
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    disabled: is_executing(),
                    onclick: execute_script,
                    if is_executing() { "执行中..." } else { "执行脚本" }
                }

                button {
                    padding: "8px 16px",
                    background: COLOR_BG_TERTIARY,
                    color: COLOR_TEXT,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    onclick: load_script,
                    "加载脚本 (SCRIPT LOAD)"
                }

                button {
                    padding: "8px 16px",
                    background: COLOR_BG_TERTIARY,
                    color: COLOR_TEXT,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    onclick: save_script,
                    "保存脚本"
                }

                button {
                    padding: "8px 16px",
                    background: COLOR_ERROR,
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    onclick: flush_scripts,
                    "清空缓存 (SCRIPT FLUSH)"
                }
            }

            if let Some(msg) = status_message() {
                div {
                    padding: "8px 12px",
                    background: if msg.contains("成功") { COLOR_SUCCESS } else { COLOR_ERROR },
                    border_radius: "4px",
                    color: "white",
                    font_size: "13px",
                    "{msg}"
                }
            }

            div {
                flex: "1",
                display: "flex",
                flex_direction: "column",
                gap: "8px",
                overflow: "hidden",

                label {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",
                    font_weight: "500",
                    "执行结果"
                }

                div {
                    flex: "1",
                    overflow_y: "auto",
                    background: COLOR_BG_SECONDARY,
                    border_radius: "4px",
                    padding: "12px",
                    font_family: "monospace",
                    font_size: "12px",
                    color: COLOR_TEXT,
                    white_space: "pre_wrap",
                    word_break: "break_all",

                    if result_output().is_empty() {
                        span {
                            color: COLOR_TEXT_SUBTLE,
                            "执行结果将显示在这里"
                        }
                    } else {
                        "{result_output}"
                    }
                }
            }
        }
    }
}