use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::redis::{parse_command_stats, CommandStat};
use crate::theme::{COLOR_BG, COLOR_BG_SECONDARY, COLOR_BORDER, COLOR_TEXT, COLOR_TEXT_SECONDARY};
use dioxus::prelude::*;

#[component]
pub fn CommandStatsPanel(connection_pool: ConnectionPool) -> Element {
    let i18n = use_i18n();
    let mut stats = use_signal(Vec::<CommandStat>::new);
    let mut loading = use_signal(|| false);

    let refresh = {
        let pool = connection_pool.clone();
        move || {
            let pool = pool.clone();
            spawn(async move {
                loading.set(true);
                match pool.get_raw_info().await {
                    Ok(raw) => stats.set(parse_command_stats(&raw)),
                    Err(error) => tracing::error!("Failed to load command stats: {}", error),
                }
                loading.set(false);
            });
        }
    };

    let initial_refresh = refresh.clone();
    use_effect(move || initial_refresh());

    {
        let refresh = refresh.clone();
        use_future(move || {
            let refresh = refresh.clone();
            async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    refresh();
                }
            }
        });
    }

    rsx! {
        div {
            height: "100%",
            overflow_y: "auto",
            background: COLOR_BG,
            padding: "16px",

            div {
                display: "flex",
                justify_content: "space-between",
                align_items: "center",
                margin_bottom: "16px",

                h2 { color: COLOR_TEXT, font_size: "18px", "Command stats" }
                button {
                    padding: "7px 12px",
                    background: COLOR_BG_SECONDARY,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "6px",
                    color: COLOR_TEXT,
                    cursor: "pointer",
                    disabled: loading(),
                    onclick: move |_| refresh(),
                    {i18n.read().t("Refresh")}
                }
            }

            if loading() && stats().is_empty() {
                div { color: COLOR_TEXT_SECONDARY, {i18n.read().t("Loading...")} }
            } else {
                table {
                    width: "100%",
                    border_collapse: "collapse",
                    thead {
                        tr {
                            th { "Command" }
                            th { "Calls" }
                            th { "Total usec" }
                            th { "usec/call" }
                            th { "Rejected" }
                            th { "Failed" }
                        }
                    }
                    tbody {
                        for stat in stats() {
                            tr {
                                key: "{stat.command}",
                                border_bottom: "1px solid {COLOR_BORDER}",
                                td { padding: "8px", color: COLOR_TEXT, "{stat.command}" }
                                td { padding: "8px", color: COLOR_TEXT_SECONDARY, "{stat.calls}" }
                                td { padding: "8px", color: COLOR_TEXT_SECONDARY, "{stat.usec}" }
                                td { padding: "8px", color: COLOR_TEXT_SECONDARY, "{stat.usec_per_call:.3}" }
                                td { padding: "8px", color: COLOR_TEXT_SECONDARY, "{stat.rejected_calls}" }
                                td { padding: "8px", color: COLOR_TEXT_SECONDARY, "{stat.failed_calls}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
