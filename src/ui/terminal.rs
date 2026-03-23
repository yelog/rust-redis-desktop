use crate::connection::ConnectionPool;
use crate::redis::{find_command, find_commands, RedisCommand};
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_CONTROL_BG,
    COLOR_CONTROL_BORDER, COLOR_PRIMARY, COLOR_SELECTION_BG, COLOR_TEXT, COLOR_TEXT_CONTRAST,
    COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE, COLOR_WARNING,
};
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
            background: COLOR_BG_TERTIARY,
            border_bottom: "1px solid {COLOR_BORDER}",
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
                    color: COLOR_ACCENT,
                    font_family: "Consolas, monospace",
                    font_size: "13px",
                    font_weight: "bold",

                    "{cmd.name}"
                }

                span {
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "10px",

                    "{cmd.group}"
                }
            }

            div {
                color: COLOR_TEXT_SECONDARY,
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
            background: COLOR_BG_TERTIARY,
            border_radius: "6px",
            margin_bottom: "12px",

            div {
                display: "flex",
                justify_content: "space_between",
                align_items: "center",
                margin_bottom: "8px",

                span {
                    color: COLOR_ACCENT,
                    font_family: "Consolas, monospace",
                    font_size: "16px",
                    font_weight: "bold",

                    "{cmd.name}"
                }

                span {
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "12px",

                    "{cmd.group}"
                }
            }

            div {
                color: COLOR_TEXT_SECONDARY,
                font_size: "12px",
                margin_bottom: "8px",

                "{cmd.description}"
            }

            div {
                background: COLOR_BG,
                padding: "8px",
                border_radius: "4px",

                code {
                    color: COLOR_WARNING,
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
            background: COLOR_BG,

            div {
                padding: "8px 12px",
                border_bottom: "1px solid {COLOR_BORDER}",
                background: COLOR_BG_SECONDARY,

                span {
                    color: COLOR_TEXT_SECONDARY,
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
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "11px",

                                "{entry.timestamp}"
                            }

                            span {
                                color: COLOR_ACCENT,
                                font_family: "Consolas, monospace",
                                font_size: "13px",

                                "> {entry.command}"
                            }
                        }

                        pre {
                            color: COLOR_TEXT,
                            font_family: "Consolas, monospace",
                            font_size: "12px",
                            margin: "0",
                            padding: "8px",
                            background: COLOR_BG_TERTIARY,
                            border_radius: "4px",
                            overflow_x: "auto",

                            "{entry.result}"
                        }
                    }
                }
            }

            if show_suggestions() && !current_suggestions.is_empty() {
                div {
                    border_top: "1px solid {COLOR_BORDER}",
                    max_height: "200px",
                    overflow_y: "auto",
                    background: COLOR_BG,

                    for (idx, cmd) in current_suggestions.iter().enumerate() {
                        div {
                            key: "{cmd.name}",
                            padding: "6px 12px",
                            cursor: "pointer",
                            background: if idx == selected_suggestion_index() { COLOR_SELECTION_BG } else { "transparent" },
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
                                    color: COLOR_ACCENT,
                                    font_family: "Consolas, monospace",
                                    font_size: "12px",

                                    "{cmd.name}"
                                }

                                span {
                                    color: COLOR_TEXT_SUBTLE,
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
                border_top: "1px solid {COLOR_BORDER}",

                div {
                    display: "flex",
                    gap: "8px",

                    span {
                        color: COLOR_ACCENT,
                        font_family: "Consolas, monospace",
                        line_height: "32px",

                        ">"
                    }

                    input {
                        flex: "1",
                        padding: "6px",
                        background: COLOR_CONTROL_BG,
                        border: "1px solid {COLOR_CONTROL_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
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
                        background: COLOR_PRIMARY,
                        color: COLOR_TEXT_CONTRAST,
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
