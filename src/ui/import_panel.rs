use crate::connection::ConnectionPool;
use crate::theme::{
    COLOR_BG, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_PRIMARY, COLOR_SUCCESS,
    COLOR_TEXT, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use dioxus::prelude::*;

#[component]
pub fn ImportPanel(connection_pool: ConnectionPool, on_close: EventHandler<()>) -> Element {
    let mut import_data = use_signal(String::new);
    let mut status_message = use_signal(|| None::<String>);
    let mut is_importing = use_signal(|| false);
    let mut imported_count = use_signal(|| 0usize);

    let do_import = {
        let pool = connection_pool.clone();
        move |_| {
            let data = import_data();
            if data.is_empty() {
                status_message.set(Some("请输入要导入的数据".to_string()));
                return;
            }

            is_importing.set(true);
            status_message.set(None);

            let pool = pool.clone();
            let data = data.clone();
            let mut status_message = status_message.clone();
            let mut imported_count = imported_count.clone();
            let mut is_importing = is_importing.clone();

            spawn(async move {
                match pool.import_json_data(&data).await {
                    Ok(count) => {
                        imported_count.set(count);
                        status_message.set(Some(format!("成功导入 {} 个键", count)));
                        import_data.set(String::new());
                    }
                    Err(e) => {
                        status_message.set(Some(format!("导入失败: {}", e)));
                    }
                }
                is_importing.set(false);
            });
        }
    };

    let load_sample = move |_| {
        import_data.set(
            r#"[
  {
    "key": "user:1",
    "type": "string",
    "value": "John Doe",
    "ttl": null
  },
  {
    "key": "user:2",
    "type": "hash",
    "fields": {
      "name": "Jane Doe",
      "email": "jane@example.com",
      "age": "30"
    },
    "ttl": 3600
  },
  {
    "key": "tags",
    "type": "set",
    "members": ["redis", "database", "cache"],
    "ttl": null
  },
  {
    "key": "queue",
    "type": "list",
    "elements": ["task1", "task2", "task3"],
    "ttl": null
  },
  {
    "key": "scores",
    "type": "zset",
    "scored_members": [["player1", "100"], ["player2", "200"], ["player3", "150"]],
    "ttl": null
  }
]"#
            .to_string(),
        );
    };

    rsx! {
        div {
            width: "600px",
            max_height: "80vh",
            display: "flex",
            flex_direction: "column",
            background: COLOR_BG,
            border_radius: "8px",
            overflow: "hidden",
            animation: "modalFadeIn 0.2s ease-out",

            style {
                r#"
                @keyframes modalFadeIn {{
                    from {{ opacity: 0; transform: scale(0.95); }}
                    to {{ opacity: 1; transform: scale(1); }}
                }}
                "#
            }

            div {
                display: "flex",
                justify_content: "space-between",
                align_items: "center",
                padding: "16px",
                border_bottom: "1px solid {COLOR_BORDER}",

                h3 {
                    margin: "0",
                    color: COLOR_TEXT,
                    font_size: "16px",
                    "导入数据"
                }

                button {
                    padding: "4px 8px",
                    background: "transparent",
                    border: "none",
                    color: COLOR_TEXT_SECONDARY,
                    cursor: "pointer",
                    font_size: "18px",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            div {
                flex: "1",
                padding: "16px",
                overflow_y: "auto",

                div {
                    margin_bottom: "12px",

                    label {
                        display: "block",
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",
                        margin_bottom: "8px",
                        "JSON 数据格式"
                    }

                    textarea {
                        width: "100%",
                        height: "250px",
                        padding: "12px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        font_family: "monospace",
                        box_sizing: "border_box",
                        resize: "vertical",
                        value: "{import_data}",
                        oninput: move |e| import_data.set(e.value()),
                        placeholder: "粘贴 JSON 数据...",
                    }
                }

                div {
                    margin_bottom: "12px",

                    button {
                        padding: "6px 12px",
                        background: COLOR_BG_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT_SECONDARY,
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: load_sample,
                        "加载示例数据"
                    }
                }

                if let Some(msg) = status_message() {
                    div {
                        padding: "10px 12px",
                        background: if imported_count() > 0 { COLOR_SUCCESS } else { COLOR_BG_SECONDARY },
                        border_radius: "4px",
                        color: if imported_count() > 0 { "white" } else { COLOR_TEXT_SECONDARY },
                        font_size: "13px",
                        "{msg}"
                    }
                }
            }

            div {
                display: "flex",
                justify_content: "flex_end",
                gap: "8px",
                padding: "16px",
                border_top: "1px solid {COLOR_BORDER}",

                button {
                    padding: "8px 16px",
                    background: COLOR_BG_TERTIARY,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "4px",
                    color: COLOR_TEXT_SECONDARY,
                    cursor: "pointer",
                    font_size: "13px",
                    onclick: move |_| on_close.call(()),
                    "取消"
                }

                button {
                    padding: "8px 16px",
                    background: COLOR_PRIMARY,
                    border: "none",
                    border_radius: "4px",
                    color: "white",
                    cursor: "pointer",
                    font_size: "13px",
                    disabled: is_importing(),
                    onclick: do_import,
                    if is_importing() { "导入中..." } else { "导入" }
                }
            }
        }
    }
}
