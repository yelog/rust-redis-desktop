use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::redis::ServerInfo;
use dioxus::prelude::*;
use std::time::Duration;

const COLOR_BG: &str = "var(--theme-bg)";
const COLOR_BG_SECONDARY: &str = "var(--theme-bg-secondary)";
const COLOR_BG_TERTIARY: &str = "var(--theme-bg-tertiary)";
const COLOR_BORDER: &str = "var(--theme-border)";
const COLOR_TEXT: &str = "var(--theme-text)";
const COLOR_TEXT_SECONDARY: &str = "var(--theme-text-secondary)";
const COLOR_TEXT_SUBTLE: &str = "var(--theme-text-subtle, #808080)";
const COLOR_PRIMARY: &str = "var(--theme-primary)";
const COLOR_ACCENT: &str = "var(--theme-accent)";
const COLOR_SUCCESS: &str = "var(--theme-success, #107c10)";

fn format_uptime(seconds: u64, i18n: &crate::i18n::I18n) -> String {
    if seconds == 0 {
        return format!("0 {}", i18n.t("seconds"));
    }

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days} {}", i18n.t("days")));
    }
    if hours > 0 {
        parts.push(format!("{hours} {}", i18n.t("hours")));
    }
    if minutes > 0 {
        parts.push(format!("{minutes} {}", i18n.t("minutes")));
    }
    if secs > 0 && parts.is_empty() {
        parts.push(format!("{secs} {}", i18n.t("seconds")));
    }

    parts.join(" ")
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

#[derive(Clone, PartialEq)]
struct InfoSection {
    name: String,
    items: Vec<(String, String)>,
}

fn parse_raw_info(raw: &str) -> Vec<InfoSection> {
    let mut sections: Vec<InfoSection> = Vec::new();
    let mut current_section: Option<InfoSection> = None;

    for line in raw.lines() {
        let line = line.trim();

        if line.starts_with("# ") {
            if let Some(section) = current_section.take() {
                if !section.items.is_empty() {
                    sections.push(section);
                }
            }
            current_section = Some(InfoSection {
                name: line[2..].to_string(),
                items: Vec::new(),
            });
        } else if let Some((key, value)) = line.split_once(':') {
            if let Some(ref mut section) = current_section {
                section
                    .items
                    .push((key.trim().to_string(), value.trim().to_string()));
            }
        }
    }

    if let Some(section) = current_section {
        if !section.items.is_empty() {
            sections.push(section);
        }
    }

    sections
}

#[component]
fn StatCard(title: String, value: String, subtitle: Option<String>) -> Element {
    rsx! {
        div {
            background: COLOR_BG_SECONDARY,
            border: "1px solid {COLOR_BORDER}",
            border_radius: "8px",
            padding: "12px 16px",

            div {
                color: COLOR_TEXT_SECONDARY,
                font_size: "12px",
                margin_bottom: "4px",

                "{title}"
            }

            div {
                color: COLOR_TEXT,
                font_size: "16px",
                font_weight: "bold",

                "{value}"
            }

            if let Some(sub) = subtitle {
                div {
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "11px",
                    margin_top: "2px",

                    "{sub}"
                }
            }
        }
    }
}

#[component]
fn InfoTable(sections: Vec<InfoSection>, search_keyword: String) -> Element {
    let i18n = use_i18n();
    let keyword = search_keyword.trim().to_lowercase();
    let has_search = !keyword.is_empty();

    let filtered_sections: Vec<InfoSection> = if has_search {
        sections
            .into_iter()
            .map(|section| {
                let filtered_items: Vec<(String, String)> = section
                    .items
                    .into_iter()
                    .filter(|(key, value)| {
                        key.to_lowercase().contains(&keyword)
                            || value.to_lowercase().contains(&keyword)
                    })
                    .collect();
                InfoSection {
                    name: section.name,
                    items: filtered_items,
                }
            })
            .filter(|section| !section.items.is_empty())
            .collect()
    } else {
        sections
    };

    rsx! {
        div {
            margin_top: "24px",

            div {
                display: "flex",
                justify_content: "space_between",
                align_items: "center",
                margin_bottom: "12px",
                gap: "12px",

                div {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",
                    padding_left: "4px",

                    {i18n.read().t("Details")}
                }

                if has_search {
                    span {
                        color: COLOR_ACCENT,
                        font_size: "12px",
                        background: "rgba(0, 122, 204, 0.12)",
                        padding: "2px 8px",
                        border_radius: "4px",

                        {format!(
                            "{} {}",
                            filtered_sections.iter().map(|s| s.items.len()).sum::<usize>(),
                            i18n.read().t("matches found")
                        )}
                    }
                }
            }

            if filtered_sections.is_empty() {
                div {
                    padding: "40px 20px",
                    text_align: "center",
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "14px",

                    if has_search {
                        {format!("{} \"{}\"", i18n.read().t("No information matched"), search_keyword)}
                    } else {
                        {i18n.read().t("No information")}
                    }
                }
            } else {
                for section in filtered_sections {
                    div {
                        margin_bottom: "16px",
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "8px",
                        overflow: "hidden",

                        div {
                            background: COLOR_BG_TERTIARY,
                            padding: "10px 16px",
                            border_bottom: "1px solid {COLOR_BORDER}",
                            color: COLOR_PRIMARY,
                            font_size: "13px",
                            font_weight: "bold",

                            "{section.name}"
                        }

                        table {
                            width: "100%",
                            border_collapse: "collapse",

                            tbody {
                                for (idx, (key, value)) in section.items.iter().enumerate() {
                                    tr {
                                        background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },

                                        td {
                                            width: "35%",
                                            padding: "8px 16px",
                                            color: COLOR_TEXT_SECONDARY,
                                            font_size: "12px",
                                            border_bottom: "1px solid {COLOR_BORDER}",

                                            "{key}"
                                        }

                                        td {
                                            padding: "8px 16px",
                                            color: COLOR_TEXT,
                                            font_size: "12px",
                                            font_family: "Consolas, 'Courier New', monospace",
                                            border_bottom: "1px solid {COLOR_BORDER}",
                                            word_break: "break_all",

                                            "{value}"
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

#[component]
fn ServerInfoContent(info: ServerInfo, raw_info: String) -> Element {
    let mut search_keyword = use_signal(String::new);
    let i18n = use_i18n();

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "20px",

            div {
                background: COLOR_BG_SECONDARY,
                border: "1px solid {COLOR_BORDER}",
                border_radius: "12px",
                padding: "24px",

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "12px",
                    margin_bottom: "8px",

                    div {
                        width: "12px",
                        height: "12px",
                        background: COLOR_SUCCESS,
                        border_radius: "50%",
                    }

                    span {
                        color: COLOR_ACCENT,
                        font_size: "14px",

                        {i18n.read().t("Redis server connected")}
                    }
                }

                div {
                    color: COLOR_TEXT,
                    font_size: "28px",
                    font_weight: "bold",

                    if let Some(ref version) = info.redis_version {
                        "Redis {version}"
                    } else {
                        "Redis"
                    }
                }

                if let Some(ref mode) = info.redis_mode {
                    div {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "14px",
                        margin_top: "4px",

                        {format!("{} {}", mode, i18n.read().t("mode"))}
                    }
                }
            }

            div {
                display: "grid",
                grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                gap: "12px",

                if let Some(pid) = info.process_id {
                    StatCard {
                        title: i18n.read().t("Process ID"),
                        value: pid.to_string(),
                        subtitle: None,
                    }
                }

                if let Some(port) = info.tcp_port {
                    StatCard {
                        title: i18n.read().t("Port"),
                        value: port.to_string(),
                        subtitle: None,
                    }
                }

                if let Some(ref os) = info.os {
                    StatCard {
                        title: i18n.read().t("Operating System"),
                        value: os.clone(),
                        subtitle: info.arch_bits.clone().map(|b| format!("{b}-{}", i18n.read().t("bit"))),
                    }
                }

                if let Some(uptime) = info.uptime_in_seconds {
                    StatCard {
                        title: i18n.read().t("Uptime"),
                        value: format_uptime(uptime, &i18n.read()),
                        subtitle: None,
                    }
                }
            }

            div {
                margin_top: "8px",

                div {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",
                    margin_bottom: "12px",
                    padding_left: "4px",

                    {i18n.read().t("Memory Information")}
                }

                div {
                    display: "grid",
                    grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                    gap: "12px",

                    if let Some(ref mem) = info.used_memory_human {
                        StatCard {
                            title: i18n.read().t("Used Memory"),
                            value: mem.clone(),
                            subtitle: info.used_memory.map(|b| format!("{} bytes", b)),
                        }
                    }

                    if let Some(ref peak) = info.used_memory_peak_human {
                        StatCard {
                            title: i18n.read().t("Peak Memory"),
                            value: peak.clone(),
                            subtitle: None,
                        }
                    }

                    if let Some(ratio) = info.mem_fragmentation_ratio {
                        StatCard {
                            title: i18n.read().t("Memory Fragmentation Ratio"),
                            value: format!("{ratio:.2}"),
                            subtitle: None,
                        }
                    }

                    if let Some(ref allocator) = info.mem_allocator {
                        StatCard {
                            title: i18n.read().t("Memory Allocator"),
                            value: allocator.clone(),
                            subtitle: None,
                        }
                    }
                }
            }

            div {
                margin_top: "8px",

                div {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "13px",
                    margin_bottom: "12px",
                    padding_left: "4px",

                    {i18n.read().t("Connections and Stats")}
                }

                div {
                    display: "grid",
                    grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                    gap: "12px",

                    if let Some(clients) = info.connected_clients {
                        StatCard {
                            title: i18n.read().t("Current Connections"),
                            value: clients.to_string(),
                            subtitle: info.max_clients.map(|m| format!("{}: {m}", i18n.read().t("Max"))),
                        }
                    }

                    if info.keys_total > 0 {
                        StatCard {
                            title: i18n.read().t("Total Keys"),
                            value: info.keys_total.to_string(),
                            subtitle: if info.expires_total > 0 {
                                Some(format!("{} {}", info.expires_total, i18n.read().t("with expiration")))
                            } else {
                                None
                            },
                        }
                    }

                    if let Some(ops) = info.instantaneous_ops_per_sec {
                        StatCard {
                            title: i18n.read().t("Operations Per Second"),
                            value: ops.to_string(),
                            subtitle: None,
                        }
                    }

                    if let Some(cmds) = info.total_commands_processed {
                        StatCard {
                            title: i18n.read().t("Total Commands"),
                            value: format_number(cmds),
                            subtitle: None,
                        }
                    }
                }
            }

            if info.aof_enabled == Some(1) || info.rdb_last_save_time.is_some() {
                div {
                    margin_top: "8px",

                    div {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        margin_bottom: "12px",
                        padding_left: "4px",

                        {i18n.read().t("Persistence Status")}
                    }

                    div {
                        display: "grid",
                        grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                        gap: "12px",

                        if info.aof_enabled == Some(1) {
                            StatCard {
                                title: "AOF".to_string(),
                                value: i18n.read().t("Enabled"),
                                subtitle: if info.aof_rewrite_in_progress == Some(1) {
                                    Some(i18n.read().t("Rewriting..."))
                                } else {
                                    None
                                },
                            }
                        }

                        if let Some(changes) = info.rdb_changes_since_last_save {
                            StatCard {
                                title: i18n.read().t("RDB Pending Save Changes"),
                                value: changes.to_string(),
                                subtitle: None,
                            }
                        }
                    }
                }
            }

            div {
                margin_top: "24px",
                padding: "12px",
                background: COLOR_BG_SECONDARY,
                border: "1px solid {COLOR_BORDER}",
                border_radius: "8px",

                div {
                    display: "flex",
                    gap: "8px",
                    align_items: "center",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        white_space: "nowrap",

                        {i18n.read().t("Search")}
                    }

                    input {
                        flex: "1",
                        padding: "8px 12px",
                        background: COLOR_BG,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        color: COLOR_TEXT,
                        font_size: "13px",
                        value: "{search_keyword}",
                        placeholder: i18n.read().t("Enter a keyword to search keys and values"),
                        oninput: move |e| search_keyword.set(e.value()),
                    }

                    if !search_keyword.read().is_empty() {
                        button {
                            padding: "6px 12px",
                            background: COLOR_BG_TERTIARY,
                            color: COLOR_TEXT_SECONDARY,
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| search_keyword.set(String::new()),

                            {i18n.read().t("Clear")}
                        }
                    }
                }
            }

            InfoTable {
                sections: parse_raw_info(&raw_info),
                search_keyword: search_keyword(),
            }
        }
    }
}

async fn load_server_info(pool: ConnectionPool) -> Result<(ServerInfo, String), String> {
    let raw = pool
        .get_raw_info()
        .await
        .map_err(|e| format!("Failed to load server info: {e}"))?;

    let info = pool
        .get_server_info()
        .await
        .map_err(|e| format!("Failed to parse server info: {e}"))?;

    Ok((info, raw))
}

#[component]
pub fn ServerInfoPanel(
    connection_pool: ConnectionPool,
    connection_version: u32,
    auto_refresh_interval: u32,
) -> Element {
    let server_info = use_signal(|| None::<ServerInfo>);
    let raw_info = use_signal(String::new);
    let loading = use_signal(|| true);
    let error_msg = use_signal(String::new);
    let mut refresh_trigger = use_signal(|| 0u32);
    let i18n = use_i18n();

    let pool = connection_pool.clone();

    let load_data = {
        let pool = pool.clone();
        move || {
            let pool = pool.clone();
            let mut server_info = server_info.clone();
            let mut raw_info = raw_info.clone();
            let mut loading = loading.clone();
            let mut error_msg = error_msg.clone();

            spawn(async move {
                loading.set(true);
                error_msg.set(String::new());

                match load_server_info(pool).await {
                    Ok((info, raw)) => {
                        server_info.set(Some(info));
                        raw_info.set(raw);
                    }
                    Err(e) => {
                        error_msg.set(e);
                    }
                }

                loading.set(false);
            });
        }
    };

    use_effect({
        let load_data = load_data.clone();
        move || {
            let _ = refresh_trigger();
            let _ = connection_version;
            load_data();
        }
    });

    use_effect(move || {
        let interval = auto_refresh_interval;
        if interval == 0 {
            return;
        }

        let mut refresh_trigger = refresh_trigger.clone();

        spawn(async move {
            let mut timer = tokio::time::interval(Duration::from_secs(interval as u64));
            loop {
                timer.tick().await;
                refresh_trigger.set(refresh_trigger() + 1);
            }
        });
    });

    let is_loading = loading();
    let error = error_msg();
    let info = server_info();
    let raw = raw_info();
    let refresh_label = if is_loading {
        i18n.read().t("Refreshing...")
    } else {
        format!("🔄 {}", i18n.read().t("Refresh"))
    };

    rsx! {
        div {
            flex: "1",
            height: "100%",
            background: COLOR_BG,
            display: "flex",
            flex_direction: "column",
            overflow_y: "auto",

            div {
                padding: "20px",

                div {
                    display: "flex",
                    justify_content: "flex_end",
                    margin_bottom: "16px",

                    button {
                        padding: "6px 12px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),

                        {refresh_label}
                    }
                }

                if is_loading {
                    div {
                        display: "flex",
                        justify_content: "center",
                        align_items: "center",
                        height: "200px",
                        color: COLOR_TEXT_SECONDARY,

                        {i18n.read().t("Loading...")}
                    }
                } else if !error.is_empty() {
                    div {
                        display: "flex",
                        justify_content: "center",
                        align_items: "center",
                        height: "200px",
                        color: "var(--theme-error, #d13438)",

                        "{error}"
                    }
                } else if let Some(info) = info {
                    ServerInfoContent {
                        info: info,
                        raw_info: raw,
                    }
                } else {
                    div {
                        display: "flex",
                        justify_content: "center",
                        align_items: "center",
                        height: "200px",
                        color: COLOR_TEXT_SECONDARY,

                        {i18n.read().t("Unable to load server info")}
                    }
                }
            }
        }
    }
}
