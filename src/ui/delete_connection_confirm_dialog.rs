use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn DeleteConnectionConfirmDialog(
    connection_id: Uuid,
    connection_name: String,
    colors: ThemeColors,
    on_confirm: EventHandler<Uuid>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "450px".to_string(),

            h3 {
                color: "{colors.error}",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "8px",
                font_size: "18px",

                "⚠️ 确认删除连接"
            }

            div {
                background: "{colors.background_tertiary}",
                border: "1px solid {colors.border}",
                border_radius: "4px",
                padding: "16px",
                margin_bottom: "16px",

                div {
                    color: "{colors.text_secondary}",
                    font_size: "13px",
                    margin_bottom: "12px",

                    "将要删除的连接："
                }

                div {
                    display: "flex",
                    justify_content: "space_between",
                    align_items: "center",
                    padding: "8px 0",

                    span {
                        color: "{colors.text}",
                        font_size: "14px",

                        "连接名称"
                    }

                    span {
                        color: "{colors.accent}",
                        font_size: "14px",
                        font_weight: "bold",

                        "{connection_name}"
                    }
                }
            }

            div {
                color: "{colors.error}",
                font_size: "13px",
                margin_bottom: "16px",
                padding: "8px 12px",
                background: "{colors.error_bg}",
                border_radius: "4px",

                "⚠️ 此操作将删除该连接配置，且不可恢复！"
            }

            div {
                display: "flex",
                gap: "8px",

                button {
                    flex: "1",
                    padding: "10px",
                    background: "{colors.error}",
                    color: "{colors.primary_text}",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    onclick: {
                        let on_confirm = on_confirm.clone();
                        move |_| {
                            on_confirm.call(connection_id);
                        }
                    },

                    "确认删除"
                }

                button {
                    flex: "1",
                    padding: "10px",
                    background: "{colors.background_tertiary}",
                    color: "{colors.text}",
                    border: "1px solid {colors.border}",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "13px",
                    onclick: move |_| on_cancel.call(()),

                    "取消"
                }
            }
        }
    }
}
