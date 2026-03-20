use dioxus::prelude::*;
use crate::connection::ConnectionPool;

#[derive(Clone, PartialEq)]
pub struct CommandHistory {
    pub command: String,
    pub result: String,
    pub timestamp: String,
}

#[component]
pub fn Terminal(
    connection_pool: ConnectionPool,
) -> Element {
    let mut input = use_signal(String::new);
    let mut history = use_signal(Vec::<CommandHistory>::new);
    let mut executing = use_signal(|| false);
    
    let execute_command = {
        let pool = connection_pool.clone();
        move || {
            let cmd = input().trim().to_string();
            if cmd.is_empty() {
                return;
            }
            
            let pool = pool.clone();
            spawn(async move {
                executing.set(true);
                
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                
                // Execute command using raw Redis connection
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
    
    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",
            background: "#1e1e1e",
            
            // History area
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
            
            // Input area
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
                        oninput: move |e| input.set(e.value()),
                        onkeydown: {
                            let execute_command = execute_command.clone();
                            move |e| {
                                if e.data().key() == Key::Enter {
                                    execute_command();
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