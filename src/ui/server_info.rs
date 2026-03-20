use crate::connection::ConnectionPool;
use crate::redis::ServerInfo;
use dioxus::prelude::*;
use std::time::Duration;

fn format_uptime(seconds: u64) -> String {
    if seconds == 0 {
        return "0秒".to_string();
    }
    
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}天"));
    }
    if hours > 0 {
        parts.push(format!("{hours}小时"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}分钟"));
    }
    if secs > 0 && parts.is_empty() {
        parts.push(format!("{secs}秒"));
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
                section.items.push((key.trim().to_string(), value.trim().to_string()));
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
            background: "#252526",
            border: "1px solid #3c3c3c",
            border_radius: "8px",
            padding: "12px 16px",
            
            div {
                color: "#888",
                font_size: "12px",
                margin_bottom: "4px",
                
                "{title}"
            }
            
            div {
                color: "white",
                font_size: "16px",
                font_weight: "bold",
                
                "{value}"
            }
            
            if let Some(sub) = subtitle {
                div {
                    color: "#666",
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
                        key.to_lowercase().contains(&keyword) || value.to_lowercase().contains(&keyword)
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
                    color: "#888",
                    font_size: "13px",
                    padding_left: "4px",
                    
                    "详细信息"
                }
                
                if has_search {
                    span {
                        color: "#4ec9b0",
                        font_size: "12px",
                        background: "rgba(78, 201, 176, 0.1)",
                        padding: "2px 8px",
                        border_radius: "4px",
                        
                        "找到 {filtered_sections.iter().map(|s| s.items.len()).sum::<usize>()} 条匹配"
                    }
                }
            }
            
            if filtered_sections.is_empty() {
                div {
                    padding: "40px 20px",
                    text_align: "center",
                    color: "#666",
                    font_size: "14px",
                    
                    if has_search {
                        "未找到匹配 \"{search_keyword}\" 的信息"
                    } else {
                        "暂无信息"
                    }
                }
            } else {
                for section in filtered_sections {
                    div {
                        margin_bottom: "16px",
                        border: "1px solid #3c3c3c",
                        border_radius: "8px",
                        overflow: "hidden",
                        
                        div {
                            background: "#2d2d2d",
                            padding: "10px 16px",
                            border_bottom: "1px solid #3c3c3c",
                            color: "#4ec9b0",
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
                                        background: if idx % 2 == 0 { "#252526" } else { "#1e1e1e" },
                                        
                                        td {
                                            width: "35%",
                                            padding: "8px 16px",
                                            color: "#888",
                                            font_size: "12px",
                                            border_bottom: "1px solid #3c3c3c",
                                            
                                            "{key}"
                                        }
                                        
                                        td {
                                            padding: "8px 16px",
                                            color: "white",
                                            font_size: "12px",
                                            font_family: "Consolas, 'Courier New', monospace",
                                            border_bottom: "1px solid #3c3c3c",
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
    
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "20px",
            
            div {
                background: "#252526",
                border: "1px solid #3c3c3c",
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
                        background: "#4ade80",
                        border_radius: "50%",
                    }
                    
                    span {
                        color: "#4ec9b0",
                        font_size: "14px",
                        
                        "Redis 服务器已连接"
                    }
                }
                
                div {
                    color: "white",
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
                        color: "#888",
                        font_size: "14px",
                        margin_top: "4px",
                        
                        "{mode} 模式"
                    }
                }
            }
            
            div {
                display: "grid",
                grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                gap: "12px",
                
                if let Some(pid) = info.process_id {
                    StatCard {
                        title: "进程 ID".to_string(),
                        value: pid.to_string(),
                        subtitle: None,
                    }
                }
                
                if let Some(port) = info.tcp_port {
                    StatCard {
                        title: "端口".to_string(),
                        value: port.to_string(),
                        subtitle: None,
                    }
                }
                
                if let Some(ref os) = info.os {
                    StatCard {
                        title: "操作系统".to_string(),
                        value: os.clone(),
                        subtitle: info.arch_bits.clone().map(|b| format!("{b} 位")),
                    }
                }
                
                if let Some(uptime) = info.uptime_in_seconds {
                    StatCard {
                        title: "运行时间".to_string(),
                        value: format_uptime(uptime),
                        subtitle: None,
                    }
                }
            }
            
            div {
                margin_top: "8px",
                
                div {
                    color: "#888",
                    font_size: "13px",
                    margin_bottom: "12px",
                    padding_left: "4px",
                    
                    "内存信息"
                }
                
                div {
                    display: "grid",
                    grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                    gap: "12px",
                    
                    if let Some(ref mem) = info.used_memory_human {
                        StatCard {
                            title: "已用内存".to_string(),
                            value: mem.clone(),
                            subtitle: info.used_memory.map(|b| format!("{} bytes", b)),
                        }
                    }
                    
                    if let Some(ref peak) = info.used_memory_peak_human {
                        StatCard {
                            title: "峰值内存".to_string(),
                            value: peak.clone(),
                            subtitle: None,
                        }
                    }
                    
                    if let Some(ratio) = info.mem_fragmentation_ratio {
                        StatCard {
                            title: "内存碎片率".to_string(),
                            value: format!("{ratio:.2}"),
                            subtitle: None,
                        }
                    }
                    
                    if let Some(ref allocator) = info.mem_allocator {
                        StatCard {
                            title: "内存分配器".to_string(),
                            value: allocator.clone(),
                            subtitle: None,
                        }
                    }
                }
            }
            
            div {
                margin_top: "8px",
                
                div {
                    color: "#888",
                    font_size: "13px",
                    margin_bottom: "12px",
                    padding_left: "4px",
                    
                    "连接与统计"
                }
                
                div {
                    display: "grid",
                    grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                    gap: "12px",
                    
                    if let Some(clients) = info.connected_clients {
                        StatCard {
                            title: "当前连接数".to_string(),
                            value: clients.to_string(),
                            subtitle: info.max_clients.map(|m| format!("最大: {m}")),
                        }
                    }
                    
                    if info.keys_total > 0 {
                        StatCard {
                            title: "Key 总数".to_string(),
                            value: info.keys_total.to_string(),
                            subtitle: if info.expires_total > 0 {
                                Some(format!("{} 个设置了过期时间", info.expires_total))
                            } else {
                                None
                            },
                        }
                    }
                    
                    if let Some(ops) = info.instantaneous_ops_per_sec {
                        StatCard {
                            title: "每秒操作数".to_string(),
                            value: ops.to_string(),
                            subtitle: None,
                        }
                    }
                    
                    if let Some(cmds) = info.total_commands_processed {
                        StatCard {
                            title: "总命令数".to_string(),
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
                        color: "#888",
                        font_size: "13px",
                        margin_bottom: "12px",
                        padding_left: "4px",
                        
                        "持久化状态"
                    }
                    
                    div {
                        display: "grid",
                        grid_template_columns: "repeat(auto-fit, minmax(200px, 1fr))",
                        gap: "12px",
                        
                        if info.aof_enabled == Some(1) {
                            StatCard {
                                title: "AOF".to_string(),
                                value: "已启用".to_string(),
                                subtitle: if info.aof_rewrite_in_progress == Some(1) {
                                    Some("正在重写...".to_string())
                                } else {
                                    None
                                },
                            }
                        }
                        
                        if let Some(changes) = info.rdb_changes_since_last_save {
                            StatCard {
                                title: "RDB 待保存变更".to_string(),
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
                background: "#252526",
                border: "1px solid #3c3c3c",
                border_radius: "8px",
                
                div {
                    display: "flex",
                    gap: "8px",
                    align_items: "center",
                    
                    span {
                        color: "#888",
                        font_size: "13px",
                        white_space: "nowrap",
                        
                        "搜索"
                    }
                    
                    input {
                        flex: "1",
                        padding: "8px 12px",
                        background: "#1e1e1e",
                        border: "1px solid #3c3c3c",
                        border_radius: "6px",
                        color: "white",
                        font_size: "13px",
                        value: "{search_keyword}",
                        placeholder: "输入关键字搜索（支持 key 和 value）",
                        oninput: move |e| search_keyword.set(e.value()),
                    }
                    
                    if !search_keyword.read().is_empty() {
                        button {
                            padding: "6px 12px",
                            background: "#3c3c3c",
                            color: "#888",
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| search_keyword.set(String::new()),
                            
                            "清除"
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
    let raw = pool.get_raw_info()
        .await
        .map_err(|e| format!("获取服务器信息失败: {e}"))?;
    
    let info = pool.get_server_info()
        .await
        .map_err(|e| format!("解析服务器信息失败: {e}"))?;
    
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
    
    rsx! {
        div {
            flex: "1",
            height: "100%",
            background: "#1e1e1e",
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
                        background: "#3c3c3c",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),
                        
                        if is_loading { "刷新中..." } else { "🔄 刷新" }
                    }
                }
                
                if is_loading {
                    div {
                        display: "flex",
                        justify_content: "center",
                        align_items: "center",
                        height: "200px",
                        color: "#888",
                        
                        "加载中..."
                    }
                } else if !error.is_empty() {
                    div {
                        display: "flex",
                        justify_content: "center",
                        align_items: "center",
                        height: "200px",
                        color: "#f87171",
                        
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
                        color: "#888",
                        
                        "无法获取服务器信息"
                    }
                }
            }
        }
    }
}