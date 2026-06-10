use crate::config::{ConfigStorage, HistoryEntry};
use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::redis::{find_command, find_commands, RedisCommand};
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_CONTROL_BG,
    COLOR_CONTROL_BORDER, COLOR_PRIMARY, COLOR_SELECTION_BG, COLOR_TEXT, COLOR_TEXT_CONTRAST,
    COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE, COLOR_WARNING,
};
use chrono::Utc;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::time::Duration;

#[derive(Clone, PartialEq)]
pub struct TerminalHistoryEntry {
    pub command: String,
    pub result: String,
    pub timestamp: String,
}

/// 建议面板中的条目：内置命令 或 历史命令
#[derive(Clone, PartialEq)]
enum SuggestionItem {
    Builtin(&'static RedisCommand),
    History(String),
}

impl SuggestionItem {
    fn display_name(&self) -> &str {
        match self {
            SuggestionItem::Builtin(cmd) => cmd.name,
            SuggestionItem::History(cmd) => cmd.as_str(),
        }
    }
}

/// 子序列模糊匹配，返回 None（不匹配）或 Some(score)
/// score 越高匹配越好：连续匹配加权，首字符匹配加权
fn fuzzy_score(query: &str, candidate: &str) -> Option<u32> {
    if query.is_empty() {
        return Some(0);
    }
    let q_lower: Vec<char> = query.to_lowercase().chars().collect();
    let c_lower: Vec<char> = candidate.to_lowercase().chars().collect();
    let mut score = 0u32;
    let mut qi = 0usize;
    let mut last_match_pos: Option<usize> = None;
    for (ci, &cc) in c_lower.iter().enumerate() {
        if qi < q_lower.len() && cc == q_lower[qi] {
            // 连续匹配加分
            score += if last_match_pos == Some(ci - 1) { 3 } else { 1 };
            // 首字符匹配额外加分
            if ci == 0 {
                score += 2;
            }
            last_match_pos = Some(ci);
            qi += 1;
        }
    }
    if qi == q_lower.len() {
        Some(score)
    } else {
        None
    }
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
fn CommandHelp(cmd: &'static RedisCommand, on_close: EventHandler<()>) -> Element {
    let i18n = use_i18n();
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

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

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

                button {
                    padding: "4px 8px",
                    background: COLOR_CONTROL_BG,
                    border: "1px solid {COLOR_CONTROL_BORDER}",
                    border_radius: "4px",
                    color: COLOR_TEXT,
                    font_size: "12px",
                    cursor: "pointer",

                    onclick: move |_| on_close.call(()),

                    {i18n.read().t("Close")}
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
    let history = use_signal(Vec::<TerminalHistoryEntry>::new);
    let executing = use_signal(|| false);
    let mut show_suggestions = use_signal(|| false);
    let show_help = use_signal(|| None::<String>);
    let mut selected_suggestion_index = use_signal(|| 0usize);
    let config_storage = use_signal(|| ConfigStorage::new().ok());
    let i18n = use_i18n();

    // 从磁盘加载持久化历史命令（最近使用的在前，去重）
    let command_history = use_signal(|| {
        let entries = ConfigStorage::new()
            .ok()
            .and_then(|s| s.load_command_history().ok())
            .map(|h| {
                h.entries
                    .iter()
                    .rev()
                    .map(|e| e.command.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        // 去重，保留最新
        let mut seen: HashSet<String> = HashSet::new();
        let mut deduped: Vec<String> = Vec::new();
        for cmd in entries {
            let upper = cmd.to_uppercase();
            if !seen.contains(&upper) {
                seen.insert(upper);
                deduped.push(cmd);
            }
        }
        deduped
    });

    // 历史导航游标：None = 当前输入，Some(i) = 指向 command_history[i]
    let mut history_cursor = use_signal(|| None::<usize>);
    // 暂存用户开始导航前的原始输入
    let mut saved_input = use_signal(String::new);

    // Auto-scroll terminal output to bottom when history changes
    use_effect(move || {
        let _len = history.read().len();
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = document::eval(
                r#"
                let el = document.getElementById('terminal-output');
                if (el) { el.scrollTop = el.scrollHeight; }
                "#,
            );
        });
    });

    // 合并内置命令（前缀）+ 历史命令（模糊匹配）的建议闭包
    let suggestions = {
        let input = input.clone();
        let command_history = command_history.clone();
        move || {
            let raw = input();
            let query = raw.trim();
            if query.is_empty() {
                return Vec::new();
            }

            let cmd_word = query
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_uppercase();
            if cmd_word.is_empty() {
                return Vec::new();
            }

            let mut items: Vec<SuggestionItem> = Vec::new();
            let mut seen_upper: HashSet<String> = HashSet::new();

            // 1. 内置命令（前缀匹配，优先展示）
            for cmd in find_commands(&cmd_word) {
                seen_upper.insert(cmd.name.to_string());
                items.push(SuggestionItem::Builtin(cmd));
            }

            // 2. 历史命令（子序列模糊匹配，按 score 降序，去重，最多5条）
            let hist = command_history.read();
            let mut scored: Vec<(String, u32)> = hist
                .iter()
                .filter_map(|cmd| {
                    let upper = cmd.to_uppercase();
                    if seen_upper.contains(&upper) {
                        return None;
                    }
                    fuzzy_score(&cmd_word, cmd).map(|s| (cmd.clone(), s))
                })
                .collect();
            // 分数高的排前面
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            for (cmd, _) in scored.into_iter().take(5) {
                seen_upper.insert(cmd.to_uppercase());
                items.push(SuggestionItem::History(cmd));
            }

            items
        }
    };

    let execute_command = {
        let pool = connection_pool.clone();
        let config_storage = config_storage.clone();
        move || {
            let cmd = input().trim().to_string();
            if cmd.is_empty() {
                return;
            }

            let pool = pool.clone();
            let config_storage = config_storage.clone();
            let mut show_suggestions = show_suggestions.clone();
            let mut history = history.clone();
            let mut executing = executing.clone();
            let mut input = input.clone();
            let mut show_help = show_help.clone();
            let mut history_cursor = history_cursor.clone();
            let mut command_history = command_history.clone();
            spawn(async move {
                executing.set(true);
                show_suggestions.set(false);
                // 重置历史导航游标
                history_cursor.set(None);

                let upper_cmd = cmd.to_uppercase();
                if upper_cmd == "HELP" || upper_cmd.starts_with("HELP ") {
                    let cmd_name = cmd.split_whitespace().nth(1).unwrap_or("").to_uppercase();
                    if !cmd_name.is_empty() {
                        if find_command(&cmd_name).is_some() {
                            show_help.set(Some(cmd_name));
                        } else {
                            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                            history.write().push(TerminalHistoryEntry {
                                command: cmd.clone(),
                                result: format!("ERROR: Unknown command '{}'", cmd_name),
                                timestamp,
                            });
                        }
                    } else {
                        show_help.set(None);
                    }
                    input.set(String::new());
                    executing.set(false);
                    return;
                }

                let start = std::time::Instant::now();
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

                let result = match pool.execute_raw_command(&cmd).await {
                    Ok(res) => res,
                    Err(e) => format!("ERROR: {}", e),
                };

                let execution_time_ms = start.elapsed().as_millis() as u64;

                history.write().push(TerminalHistoryEntry {
                    command: cmd.clone(),
                    result,
                    timestamp,
                });

                // 将执行的命令追加到内存中的历史列表（去重，最新在前）
                {
                    let mut hist = command_history.write();
                    hist.retain(|c| c.to_uppercase() != cmd.to_uppercase());
                    hist.insert(0, cmd.clone());
                }

                if let Some(storage) = config_storage.read().as_ref() {
                    let _ = storage.add_command_history(HistoryEntry {
                        command: cmd,
                        timestamp: Utc::now(),
                        execution_time_ms: Some(execution_time_ms),
                    });
                }

                input.set(String::new());
                executing.set(false);
            });
        }
    };

    let current_suggestions = suggestions();
    let submit_button_label = if executing() {
        "...".to_string()
    } else {
        i18n.read().t("Run")
    };

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

                    {i18n.read().t("Enter to execute, Tab to fill suggestion, ↑/↓ to browse history, HELP <cmd> for docs")}
                }
            }

            if let Some(ref cmd_name) = show_help() {
                if let Some(cmd) = find_command(cmd_name) {
                    CommandHelp {
                        cmd: cmd,
                        on_close: {
                            let mut show_help = show_help.clone();
                            move |_| show_help.set(None)
                        },
                    }
                }
            }

            div {
                id: "terminal-output",
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
                    max_height: "240px",
                    overflow_y: "auto",
                    background: COLOR_BG,

                    for (idx, item) in current_suggestions.iter().enumerate() {
                        {
                            let is_selected = idx == selected_suggestion_index();
                            match item {
                                SuggestionItem::Builtin(cmd) => {
                                    rsx! {
                                        div {
                                            key: "builtin-{cmd.name}",
                                            padding: "6px 12px",
                                            cursor: "pointer",
                                            background: if is_selected { COLOR_SELECTION_BG } else { "transparent" },
                                            onclick: {
                                                let cmd_name = cmd.name;
                                                move |_| {
                                                    input.set(cmd_name.to_string() + " ");
                                                    show_suggestions.set(false);
                                                    history_cursor.set(None);
                                                }
                                            },
                                            div {
                                                display: "flex",
                                                justify_content: "space_between",
                                                align_items: "center",

                                                span {
                                                    color: COLOR_ACCENT,
                                                    font_family: "Consolas, monospace",
                                                    font_size: "12px",
                                                    font_weight: "bold",

                                                    "{cmd.name}"
                                                }

                                                div {
                                                    display: "flex",
                                                    gap: "8px",
                                                    align_items: "center",

                                                    span {
                                                        color: COLOR_TEXT_SUBTLE,
                                                        font_size: "10px",

                                                        "{cmd.group}"
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
                                SuggestionItem::History(cmd) => {
                                    let cmd_clone = cmd.clone();
                                    rsx! {
                                        div {
                                            key: "hist-{cmd_clone}",
                                            padding: "6px 12px",
                                            cursor: "pointer",
                                            background: if is_selected { COLOR_SELECTION_BG } else { "transparent" },
                                            onclick: {
                                                let cmd_for_click = cmd_clone.clone();
                                                move |_| {
                                                    input.set(cmd_for_click.clone());
                                                    show_suggestions.set(false);
                                                    history_cursor.set(None);
                                                }
                                            },
                                            div {
                                                display: "flex",
                                                justify_content: "space_between",
                                                align_items: "center",

                                                span {
                                                    color: COLOR_TEXT,
                                                    font_family: "Consolas, monospace",
                                                    font_size: "12px",

                                                    "{cmd_clone}"
                                                }

                                                span {
                                                    color: COLOR_TEXT_SUBTLE,
                                                    font_size: "10px",

                                                    "history"
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
                            // 输入时自动弹出建议面板
                            let has_text = !e.value().trim().is_empty();
                            show_suggestions.set(has_text);
                            selected_suggestion_index.set(0);
                            // 重置历史游标（用户主动输入时）
                            history_cursor.set(None);
                        },
                        onkeydown: {
                            let execute_command = execute_command.clone();
                            move |e| {
                                let key = e.data().key();
                                if key == Key::Enter {
                                    // Enter 始终执行命令，与 zsh/bash/redis-cli 行为一致
                                    show_suggestions.set(false);
                                    execute_command();
                                } else if key == Key::Tab {
                                    e.prevent_default();
                                    if show_suggestions() && !current_suggestions.is_empty() {
                                        // Tab：将选中建议填充到输入框
                                        if let Some(item) = current_suggestions.get(selected_suggestion_index()) {
                                            input.set(item.display_name().to_string() + " ");
                                            show_suggestions.set(false);
                                            history_cursor.set(None);
                                        }
                                    } else {
                                        // Tab：切换建议面板显隐
                                        show_suggestions.set(!show_suggestions());
                                        selected_suggestion_index.set(0);
                                    }
                                } else if key == Key::ArrowUp {
                                    e.prevent_default();
                                    if show_suggestions() && !current_suggestions.is_empty() {
                                        // 在建议面板中向上导航
                                        let idx = if selected_suggestion_index() > 0 {
                                            selected_suggestion_index() - 1
                                        } else {
                                            current_suggestions.len() - 1
                                        };
                                        selected_suggestion_index.set(idx);
                                    } else {
                                        // 无面板时：历史命令回退
                                        let hist = command_history.read();
                                        if hist.is_empty() { return; }
                                        let cursor = history_cursor();
                                        let new_cursor = match cursor {
                                            None => {
                                                saved_input.set(input());
                                                0usize
                                            }
                                            Some(i) if i + 1 < hist.len() => i + 1,
                                            Some(i) => i,
                                        };
                                        history_cursor.set(Some(new_cursor));
                                        input.set(hist[new_cursor].clone());
                                    }
                                } else if key == Key::ArrowDown {
                                    e.prevent_default();
                                    if show_suggestions() && !current_suggestions.is_empty() {
                                        // 在建议面板中向下导航
                                        let idx = (selected_suggestion_index() + 1) % current_suggestions.len();
                                        selected_suggestion_index.set(idx);
                                    } else {
                                        // 无面板时：历史命令前进
                                        let hist = command_history.read();
                                        match history_cursor() {
                                            Some(i) => {
                                                if i > 0 {
                                                    let new_cursor = i - 1;
                                                    history_cursor.set(Some(new_cursor));
                                                    input.set(hist[new_cursor].clone());
                                                } else {
                                                    // 到达最新，恢复原始输入
                                                    history_cursor.set(None);
                                                    input.set(saved_input());
                                                }
                                            }
                                            None => {}
                                        }
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

                        {submit_button_label}
                    }
                }
            }
        }
    }
}
