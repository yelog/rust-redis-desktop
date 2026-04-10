use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;

#[component]
pub fn FlushConfirmDialog(
    connection_pool: ConnectionPool,
    current_db: u8,
    colors: ThemeColors,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let mut processing = use_signal(|| false);
    let mut db_size = use_signal(|| None::<u64>);
    let mut loading = use_signal(|| true);

    use_effect({
        let pool = connection_pool.clone();
        move || {
            if db_size().is_none() {
                let pool = pool.clone();
                spawn(async move {
                    match pool.db_size().await {
                        Ok(size) => db_size.set(Some(size)),
                        Err(e) => tracing::error!("Failed to get db size: {}", e),
                    }
                    loading.set(false);
                });
            }
        }
    });

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

                {format!("⚠️ {}", i18n.read().t("Confirm flush database"))}
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

                    {i18n.read().t("Database to flush:")}
                }

                div {
                    display: "flex",
                    justify_content: "space_between",
                    align_items: "center",
                    padding: "8px 0",

                    span {
                        color: "{colors.text}",
                        font_size: "14px",

                        {i18n.read().t("Database")}
                    }

                    span {
                        color: "{colors.accent}",
                        font_size: "14px",
                        font_weight: "bold",

                        "DB {current_db}"
                    }
                }

                div {
                    display: "flex",
                    justify_content: "space_between",
                    align_items: "center",
                    padding: "8px 0",

                    span {
                        color: "{colors.text}",
                        font_size: "14px",

                        {i18n.read().t("Key count")}
                    }

                    span {
                        color: if loading() {
                            colors.text_secondary
                        } else if db_size().unwrap_or(0) > 0 {
                            colors.error
                        } else {
                            colors.success
                        },
                        font_size: "14px",
                        font_weight: "bold",

                        if loading() {
                            {i18n.read().t("Loading...")}
                        } else {
                            "{db_size().unwrap_or(0)}"
                        }
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

                {format!("⚠️ {}", i18n.read().t("This will permanently delete all keys in the selected database."))}
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
                    disabled: processing() || loading(),
                    onclick: {
                        let pool = connection_pool.clone();
                        let on_confirm = on_confirm.clone();
                        move |_| {
                            let pool = pool.clone();
                            let on_confirm = on_confirm.clone();
                            spawn(async move {
                                processing.set(true);

                                match pool.flush_db().await {
                                    Ok(_) => tracing::info!("Database flushed successfully"),
                                    Err(e) => tracing::error!("Failed to flush database: {}", e),
                                }

                                processing.set(false);
                                on_confirm.call(());
                            });
                        }
                    },

                    if processing() {
                        {i18n.read().t("Flushing...")}
                    } else {
                        {i18n.read().t("Flush database")}
                    }
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
                    disabled: processing(),
                    onclick: move |_| on_cancel.call(()),

                    {i18n.read().t("Cancel")}
                }
            }
        }
    }
}
