use crate::i18n::use_i18n;
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
    let i18n = use_i18n();
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

                {format!("⚠️ {}", i18n.read().t("Delete connection"))}
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

                    {i18n.read().t("Connection to delete:")}
                }

                div {
                    display: "flex",
                    justify_content: "space_between",
                    align_items: "center",
                    padding: "8px 0",

                    span {
                        color: "{colors.text}",
                        font_size: "14px",

                        {i18n.read().t("Connection name")}
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

                {format!("⚠️ {}", i18n.read().t("This will delete the saved connection configuration permanently."))}
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

                    {i18n.read().t("Delete")}
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

                    {i18n.read().t("Cancel")}
                }
            }
        }
    }
}
