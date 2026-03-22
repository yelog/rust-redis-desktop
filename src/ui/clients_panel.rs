use crate::connection::ConnectionPool;
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER,
    COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SOFT, COLOR_TEXT_SUBTLE,
    COLOR_WARNING,
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ClientInfo {
    pub id: u64,
    pub addr: String,
    pub fd: u64,
    pub name: String,
    pub age: u64,
    pub idle: u64,
    pub flags: String,
    pub db: u64,
    pub sub: u64,
    pub psub: u64,
    pub multi: u64,
    pub qbuf: u64,
    pub obuf: u64,
    pub cmd: String,
}

async fn get_client_list(pool: &ConnectionPool) -> Result<Vec<ClientInfo>, String> {
    let mut connection = pool.connection.lock().await;

    if let Some(ref mut conn) = *connection {
        let result: String = redis::cmd("CLIENT")
            .arg("LIST")
            .query_async(conn)
            .await
            .map_err(|e| format!("Failed to get client list: {}", e))?;

        let mut clients = Vec::new();
        for line in result.lines() {
            if line.is_empty() {
                continue;
            }

            let mut client = ClientInfo {
                id: 0,
                addr: String::new(),
                fd: 0,
                name: String::new(),
                age: 0,
                idle: 0,
                flags: String::new(),
                db: 0,
                sub: 0,
                psub: 0,
                multi: 0,
                qbuf: 0,
                obuf: 0,
                cmd: String::new(),
            };

            for part in line.split_whitespace() {
                if let Some((key, value)) = part.split_once('=') {
                    match key {
                        "id" => client.id = value.parse().unwrap_or(0),
                        "addr" => client.addr = value.to_string(),
                        "fd" => client.fd = value.parse().unwrap_or(0),
                        "name" => client.name = value.to_string(),
                        "age" => client.age = value.parse().unwrap_or(0),
                        "idle" => client.idle = value.parse().unwrap_or(0),
                        "flags" => client.flags = value.to_string(),
                        "db" => client.db = value.parse().unwrap_or(0),
                        "sub" => client.sub = value.parse().unwrap_or(0),
                        "psub" => client.psub = value.parse().unwrap_or(0),
                        "multi" => client.multi = value.parse().unwrap_or(0),
                        "qbuf" => client.qbuf = value.parse().unwrap_or(0),
                        "obuf" => client.obuf = value.parse().unwrap_or(0),
                        "cmd" => client.cmd = value.to_string(),
                        _ => {}
                    }
                }
            }
            clients.push(client);
        }

        Ok(clients)
    } else {
        Err("Connection closed".to_string())
    }
}

async fn kill_client(pool: &ConnectionPool, addr: &str) -> Result<bool, String> {
    let mut connection = pool.connection.lock().await;

    if let Some(ref mut conn) = *connection {
        let result: i32 = redis::cmd("CLIENT")
            .arg("KILL")
            .arg("ADDR")
            .arg(addr)
            .query_async(conn)
            .await
            .map_err(|e| format!("Failed to kill client: {}", e))?;

        Ok(result == 1)
    } else {
        Err("Connection closed".to_string())
    }
}

fn format_age(seconds: u64) -> String {
    if seconds >= 86400 {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    } else if seconds >= 3600 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else if seconds >= 60 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

#[component]
pub fn ClientsPanel(connection_pool: ConnectionPool) -> Element {
    let clients = use_signal(Vec::<ClientInfo>::new);
    let loading = use_signal(|| false);
    let mut refresh_trigger = use_signal(|| 0u32);
    let mut killing_client = use_signal(|| None::<String>);
    let mut status_message = use_signal(String::new);

    use_effect({
        let pool = connection_pool.clone();
        move || {
            let _ = refresh_trigger();
            let pool = pool.clone();
            let mut clients = clients.clone();
            let mut loading = loading.clone();
            spawn(async move {
                loading.set(true);
                match get_client_list(&pool).await {
                    Ok(client_list) => {
                        clients.set(client_list);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load clients: {}", e);
                    }
                }
                loading.set(false);
            });
        }
    });

    let client_list = clients();

    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background: COLOR_BG,
            overflow_y: "auto",

            div {
                padding: "16px",
                border_bottom: "1px solid {COLOR_BORDER}",
                display: "flex",
                justify_content: "space_between",
                align_items: "center",

                h2 {
                    color: COLOR_TEXT,
                    font_size: "18px",
                    margin: "0",

                    "👥 客户端连接 ({client_list.len()})"
                }

                button {
                    padding: "6px 12px",
                    background: COLOR_BG_TERTIARY,
                    color: COLOR_TEXT,
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "12px",
                    onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),

                    "🔄 刷新"
                }
            }

            if !status_message.read().is_empty() {
                div {
                    padding: "8px 16px",
                    background: "rgba(0, 122, 204, 0.12)",
                    color: COLOR_ACCENT,
                    font_size: "13px",

                    "{status_message}"
                }
            }

            div {
                flex: "1",
                overflow: "auto",
                padding: "16px",

                if loading() && client_list.is_empty() {
                    div {
                        color: COLOR_TEXT_SECONDARY,
                        text_align: "center",
                        padding: "40px",

                        "加载中..."
                    }
                } else if client_list.is_empty() {
                    div {
                        color: COLOR_TEXT_SECONDARY,
                        text_align: "center",
                        padding: "40px",

                        "暂无客户端连接"
                    }
                } else {
                    div {
                        overflow_x: "auto",
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "8px",
                        background: COLOR_BG_SECONDARY,

                        table {
                            width: "100%",
                            border_collapse: "collapse",

                            thead {
                                tr {
                                    background: COLOR_BG_TERTIARY,
                                    border_bottom: "1px solid {COLOR_BORDER}",

                                    th {
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",
                                        white_space: "nowrap",

                                        "ID"
                                    }

                                    th {
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "地址"
                                    }

                                    th {
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "名称"
                                    }

                                    th {
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "DB"
                                    }

                                    th {
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "连接时间"
                                    }

                                    th {
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "空闲"
                                    }

                                    th {
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "命令"
                                    }

                                    th {
                                        width: "80px",
                                        padding: "10px 8px",
                                        color: COLOR_TEXT_SECONDARY,
                                        font_size: "11px",
                                        font_weight: "600",
                                        text_align: "left",

                                        "操作"
                                    }
                                }
                            }

                            tbody {
                                for (idx, client) in client_list.iter().enumerate() {
                                    tr {
                                        key: "{client.id}",
                                        background: if idx % 2 == 0 { COLOR_BG_SECONDARY } else { COLOR_BG },
                                        border_bottom: "1px solid {COLOR_BORDER}",

                                        td {
                                            padding: "8px",
                                            color: COLOR_TEXT_SECONDARY,
                                            font_size: "11px",

                                            "{client.id}"
                                        }

                                        td {
                                            padding: "8px",
                                            color: COLOR_ACCENT,
                                            font_size: "11px",
                                            font_family: "Consolas, monospace",

                                            "{client.addr}"
                                        }

                                        td {
                                            padding: "8px",
                                            color: if client.name.is_empty() { COLOR_TEXT_SUBTLE } else { COLOR_TEXT_SOFT },
                                            font_size: "11px",

                                            if client.name.is_empty() { "-" } else { "{client.name}" }
                                        }

                                        td {
                                            padding: "8px",
                                            color: COLOR_WARNING,
                                            font_size: "11px",

                                            "{client.db}"
                                        }

                                        td {
                                            padding: "8px",
                                            color: COLOR_TEXT_SOFT,
                                            font_size: "11px",

                                            "{format_age(client.age)}"
                                        }

                                        td {
                                            padding: "8px",
                                            color: if client.idle > 300 { "var(--theme-error, #d13438)" } else { COLOR_TEXT_SOFT },
                                            font_size: "11px",

                                            "{format_age(client.idle)}"
                                        }

                                        td {
                                            padding: "8px",
                                            color: COLOR_ACCENT,
                                            font_size: "11px",
                                            font_family: "Consolas, monospace",

                                            "{client.cmd}"
                                        }

                                        td {
                                            padding: "8px",

                                            button {
                                                padding: "4px 8px",
                                                background: "#c53030",
                                                color: COLOR_TEXT_CONTRAST,
                                                border: "none",
                                                border_radius: "4px",
                                                cursor: "pointer",
                                                font_size: "10px",
                                                disabled: killing_client() == Some(client.addr.clone()),
                                                onclick: {
                                                    let pool = connection_pool.clone();
                                                    let addr = client.addr.clone();
                                                    move |_| {
                                                        let pool = pool.clone();
                                                        let addr = addr.clone();
                                                        spawn(async move {
                                                            killing_client.set(Some(addr.clone()));
                                                            match kill_client(&pool, &addr).await {
                                                                Ok(_) => {
                                                                    status_message.set(format!("已断开客户端 {}", addr));
                                                                }
                                                                Err(e) => {
                                                                    status_message.set(format!("断开失败: {}", e));
                                                                }
                                                            }
                                                            killing_client.set(None);
                                                        });
                                                    }
                                                },

                                                if killing_client() == Some(client.addr.clone()) {
                                                    "..."
                                                } else {
                                                    "断开"
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
}
