use crate::connection::ConnectionPool;
use crate::redis::{find_command, find_commands, RedisCommand};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct CommandHistory {
    pub command: String,
    pub result: String,
    pub timestamp: String,
}

#[component]
fn CommandSuggestion(cmd: &'static RedisCommand, on_select: EventHandler<String>) -> Element {
    rsx! {
        div {
            padding: "6px 10px",
            cursor: "pointer",
            background: "#2d2d2d",
            border_bottom: "1px solid #3c3c3c",
            onmouseenter: |e| {
                let _ = e;
            },
            onclick: {
                let cmd_name = cmd.name;
                move |_| on_select.call(cmd_name.to_string())
            },

            div {
                display: "flex",
                justify_content: "space_between",
                align_items: "center",

                span {
                    color: "#4ec9b0",
                    font_family: "Consolas, monospace",
                    font_size: "13px",
                    font_weight: "bold",

                    "{cmd.name}"
                }

                span {
                    color: "#666",
                    font_size: "10px",

                    "{cmd.group}"
                }
            }

            div {
                color: "#888",
                font_size: "11px",
                margin_top: "2px",

                "{cmd.description}"
            }
        }
    }
}

#[component]
fn CommandHelp(cmd: &'static RedisCommand) -> Element {
    rsx! {
        div {
            padding: "12px",
            background: "#2d2d2d",
            border_radius: "6px",
            margin_bottom: "12px",

            div {
                display: "flex",
                justify_content: "space_between",
                align_items: "center",
                margin_bottom: "8px",

                span {
                    color: "#4ec9b0",
                    font_family: "Consolas, monospace",
                    font_size: "16px",
                    font_weight: "bold",

                    "{cmd.name}"
                }

                span {
                    color: "#666",
                    font_size: "12px",

                    "{cmd.group}"
                }
            }

            div {
                color: "#888",
                font_size: "12px",
                margin_bottom: "8px",

                "{cmd.description}"
            }

            div {
                background: "#1e1e1e",
                padding: "8px",
                border_radius: "4px",

                code {
                    color: "#f59e0b",
                    font_family: "Consolas, monospace",
                    font_size: "12px",
                    white_space: "pre-wrap",

                    "{cmd.syntax}"
                }
            }
        }
    }
}

#[component]
pub fn Terminal(connection_pool: ConnectionPool) -> Element {
    let mut input = use_signal(String::new);
    let mut history = use_signal(Vec::<CommandHistory>::new);
    let mut executing = use_signal(|| false);
    let mut show_suggestions = use_signal(|| false);
    let mut show_help = use_signal(|| None::<String>);
    let mut selected_suggestion_index = use_signal(|| 0usize);

    let suggestions = {
        let input = input.clone();
        move || {
            let cmd_part = input()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_uppercase();
            if cmd_part.is_empty() {
                Vec::new()
            } else {
                find_commands(&cmd_part)
            }
        }
    };

    let execute_command = {
        let pool = connection_pool.clone();
        move || {
            let cmd = input().trim().to_string();
            if cmd.is_empty() {
                return;
            }

            let pool = pool.clone();
            let mut show_suggestions = show_suggestions.clone();
            let mut history = history.clone();
            let mut executing = executing.clone();
            let mut input = input.clone();
            spawn(async move {
                executing.set(true);
                show_suggestions.set(false);

                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

                let result = match pool.execute_raw_command(&cmd).await {
                    Ok(res) => res,
                    Err(e) => format!("ERROR: {}", e),
                };

                history.write().push(CommandHistory {
                    command: cmd.clone(),
                    result,
                    timestamp,
                });

                input.set(String::new());
                executing.set(false);
            });
        }
    };

    let current_suggestions = suggestions();

    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background: "#1e1e1e",

            div {
                padding: "8px 12px",
                border_bottom: "1px solid #3c3c3c",
                background: "#252526",

                span {
                    color: "#888",
                    font_size: "12px",

                    "输入命令后按 TAB 查看补全建议，输入 HELP <command> 查看命令帮助"
                }
            }

            if let Some(ref cmd_name) = show_help() {
                if let Some(cmd) = find_command(cmd_name) {
                    CommandHelp {
                        cmd: cmd,
                    }
                }
            }

            div {
                flex: "1",
                overflow_y: "auto",
                padding: "12px",

                for entry in history.read().iter() {
                    div {
                        margin_bottom: "12px",

                        div {
                            display: "flex",
                            gap: "8px",
                            margin_bottom: "4px",

                            span {
                                color: "#888",
                                font_size: "11px",

                                "{entry.timestamp}"
                            }

                            span {
                                color: "#4ec9b0",
                                font_family: "Consolas, monospace",
                                font_size: "13px",

                                "> {entry.command}"
                            }
                        }

                        pre {
                            color: "#d4d4d4",
                            font_family: "Consolas, monospace",
                            font_size: "12px",
                            margin: "0",
                            padding: "8px",
                            background: "#2d2d2d",
                            border_radius: "4px",
                            overflow_x: "auto",

                            "{entry.result}"
                        }
                    }
                }
            }

            if show_suggestions() && !current_suggestions.is_empty() {
                div {
                    border_top: "1px solid #3c3c3c",
                    max_height: "200px",
                    overflow_y: "auto",
                    background: "#1e1e1e",

                    for (idx, cmd) in current_suggestions.iter().enumerate() {
                        div {
                            key: "{cmd.name}",
                            padding: "6px 12px",
                            cursor: "pointer",
                            background: if idx == selected_suggestion_index() { "#3c3c3c" } else { "transparent" },
                            onclick: {
                                let cmd_name = cmd.name;
                                move |_| {
                                    input.set(cmd_name.to_string() + " ");
                                    show_suggestions.set(false);
                                }
                            },

                            div {
                                display: "flex",
                                justify_content: "space_between",

                                span {
                                    color: "#4ec9b0",
                                    font_family: "Consolas, monospace",
                                    font_size: "12px",

                                    "{cmd.name}"
                                }

                                span {
                                    color: "#666",
                                    font_size: "10px",

                                    "{cmd.description}"
                                }
                            }
                        }
                    }
                }
            }

            div {
                padding: "12px",
                border_top: "1px solid #3c3c3c",

                div {
                    display: "flex",
                    gap: "8px",

                    span {
                        color: "#4ec9b0",
                        font_family: "Consolas, monospace",
                        line_height: "32px",

                        ">"
                    }

                    input {
                        flex: "1",
                        padding: "6px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        font_family: "Consolas, monospace",
                        font_size: "13px",
                        value: "{input}",
                        oninput: move |e| {
                            input.set(e.value());
                            show_suggestions.set(false);
                        },
                        onkeydown: {
                            let execute_command = execute_command.clone();
                            move |e| {
                                let key = e.data().key();
                                if key == Key::Enter {
                                    if show_suggestions() && !current_suggestions.is_empty() {
                                        if let Some(cmd) = current_suggestions.get(selected_suggestion_index()) {
                                            input.set(cmd.name.to_string() + " ");
                                            show_suggestions.set(false);
                                        }
                                    } else {
                                        execute_command();
                                    }
                                } else if key == Key::Tab {
                                    e.prevent_default();
                                    show_suggestions.set(!show_suggestions());
                                    selected_suggestion_index.set(0);
                                } else if key == Key::ArrowUp {
                                    if show_suggestions() && !current_suggestions.is_empty() {
                                        let idx = if selected_suggestion_index() > 0 {
                                            selected_suggestion_index() - 1
                                        } else {
                                            current_suggestions.len() - 1
                                        };
                                        selected_suggestion_index.set(idx);
                                    }
                                } else if key == Key::ArrowDown {
                                    if show_suggestions() && !current_suggestions.is_empty() {
                                        let idx = (selected_suggestion_index() + 1) % current_suggestions.len();
                                        selected_suggestion_index.set(idx);
                                    }
                                } else if key == Key::Escape {
                                    show_suggestions.set(false);
                                }
                            }
                        },
                    }

                    button {
                        padding: "6px 16px",
                        background: "#0e639c",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        disabled: executing(),
                        onclick: move |_| execute_command(),

                        if executing() { "..." } else { "Run" }
                    }
                }
            }
        }
    }
}
