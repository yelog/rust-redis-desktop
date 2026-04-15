use super::state::Tab;
use crate::config::{AppSettings, ConfigStorage};
use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::theme::{
    ThemeColors, ThemeId, COLOR_ACCENT, COLOR_BG, COLOR_BG_SECONDARY, COLOR_BORDER, COLOR_ERROR,
    COLOR_SURFACE_LOW, COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
};
use crate::ui::{
    ClientsPanel, ConnectionExportDialog, ConnectionImportDialog, FlushConfirmDialog, ImportPanel,
    KeyBrowser, MonitorPanel, PubSubPanel, ScriptPanel, SettingsDialog, SlowLogPanel, Terminal,
};
use dioxus::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub(super) fn spinner_panel(message: String) -> Element {
    rsx! {
        div {
            flex: "1",
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            justify_content: "center",
            gap: "16px",
            background: "{COLOR_SURFACE_LOW}",

            style { {r#"
                @keyframes spin {
                    from { transform: rotate(0deg); }
                    to { transform: rotate(360deg); }
                }
            "#} }

            div {
                width: "40px",
                height: "40px",
                border: "3px solid {COLOR_ACCENT}",
                border_top_color: "transparent",
                border_radius: "50%",
                animation: "spin 0.8s linear infinite",
            }

            div {
                color: "{COLOR_TEXT_SECONDARY}",
                font_size: "14px",
                "{message}"
            }
        }
    }
}

pub(super) fn empty_connection_panel() -> Element {
    let i18n = use_i18n();
    rsx! {
        div {
            flex: "1",
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            justify_content: "center",
            gap: "10px",
            color: "{COLOR_TEXT_SECONDARY}",
            background: "{COLOR_SURFACE_LOW}",

            div {
                font_size: "28px",
                font_weight: "700",
                color: "{COLOR_TEXT}",
                {i18n.read().t("Redis Workspace")}
            }

            div {
                font_size: "14px",
                {i18n.read().t("Select a connection from the left, or create a new Redis connection first.")}
            }
        }
    }
}

pub(super) fn connection_error_panel(on_retry: EventHandler<MouseEvent>) -> Element {
    let i18n = use_i18n();
    rsx! {
        div {
            flex: "1",
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            justify_content: "center",
            gap: "16px",
            background: "{COLOR_SURFACE_LOW}",

            div {
                color: "{COLOR_ERROR}",
                font_size: "14px",
                {i18n.read().t("Connection failed. Check the configuration and try again.")}
            }

            button {
                padding: "10px 20px",
                background: "var(--theme-primary)",
                color: "var(--theme-text-contrast)",
                border: "none",
                border_radius: "6px",
                cursor: "pointer",
                font_size: "13px",
                onclick: move |evt| on_retry.call(evt),
                {i18n.read().t("Reconnect")}
            }
        }
    }
}

#[component]
pub(super) fn MacTitlebarSection(
    context_label: Option<String>,
    on_drag: EventHandler<MouseEvent>,
    on_toggle_maximize: EventHandler<MouseEvent>,
) -> Element {
    let i18n = use_i18n();
    rsx! {
        div {
            height: "46px",
            flex_shrink: "0",
            display: "flex",
            align_items: "center",
            justify_content: "space-between",
            padding_left: "84px",
            padding_right: "16px",
            background: "linear-gradient(180deg, {COLOR_BG_SECONDARY} 0%, {COLOR_BG} 100%)",
            border_bottom: "1px solid {COLOR_BORDER}",
            user_select: "none",
            cursor: "default",
            onmousedown: move |evt| on_drag.call(evt),
            ondoubleclick: move |evt| on_toggle_maximize.call(evt),

            div {
                font_size: "13px",
                font_weight: "600",
                letter_spacing: "0.04em",
                color: "{COLOR_TEXT}",
                {i18n.read().t("Redis Desktop")}
            }

            if let Some(label) = context_label {
                div {
                    max_width: "40%",
                    overflow: "hidden",
                    text_overflow: "ellipsis",
                    white_space: "nowrap",
                    font_size: "12px",
                    color: "{COLOR_TEXT_SECONDARY}",
                    "{label}"
                }
            }
        }
    }
}

#[component]
pub(super) fn ConnectedTabShellSection(
    conn_id: Uuid,
    pool: ConnectionPool,
    current_tab: Signal<Tab>,
    connection_version: u32,
    selected_key: Signal<String>,
    current_db: Signal<u8>,
    refresh_trigger: Signal<u32>,
    colors: ThemeColors,
    resolved_theme_key: String,
    auto_refresh_interval: u32,
    on_connection_error: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    rsx! {
        div {
            flex: "1",
            min_width: "0",
            min_height: "0",
            display: "flex",
            flex_direction: "column",
            background: "{COLOR_SURFACE_LOW}",
            overflow: "hidden",

            div {
                display: "flex",
                align_items: "center",
                gap: "8px",
                padding: "10px 16px",
                border_bottom: "1px solid {COLOR_BORDER}",
                background: "{COLOR_BG_SECONDARY}",

                for (tab, label) in [
                    (Tab::Data, i18n.read().t("Data")),
                    (Tab::Terminal, i18n.read().t("Terminal")),
                    (Tab::Monitor, i18n.read().t("Monitor")),
                    (Tab::SlowLog, i18n.read().t("Slow Log")),
                    (Tab::Clients, i18n.read().t("Clients")),
                    (Tab::PubSub, "Pub/Sub".to_string()),
                    (Tab::Script, i18n.read().t("Scripts")),
                ] {
                    button {
                        padding: "8px 14px",
                        background: if current_tab() == tab { COLOR_BG } else { "transparent" },
                        color: if current_tab() == tab { COLOR_TEXT } else { COLOR_TEXT_SECONDARY },
                        border: if current_tab() == tab {
                            format!("1px solid {}", COLOR_BORDER)
                        } else {
                            "1px solid transparent".to_string()
                        },
                        border_bottom: if current_tab() == tab {
                            format!("2px solid {}", COLOR_ACCENT)
                        } else {
                            "2px solid transparent".to_string()
                        },
                        border_radius: "6px",
                        cursor: "pointer",
                        font_size: "13px",
                        font_weight: if current_tab() == tab { "700" } else { "500" },
                        transition: "all 150ms ease-out",
                        onclick: move |_| current_tab.set(tab),
                        "{label}"
                    }
                }
            }

            div {
                flex: "1",
                min_height: "0",
                display: "flex",
                flex_direction: "column",
                overflow: "hidden",

                if current_tab() == Tab::Data {
                    div {
                        flex: "1",
                        min_height: "0px",
                        overflow: "hidden",

                        KeyBrowser {
                            key: "{conn_id}-{connection_version}-{resolved_theme_key}",
                            connection_id: conn_id,
                            connection_pool: pool.clone(),
                            connection_version,
                            selected_key,
                            current_db,
                            refresh_trigger,
                            colors,
                            on_connection_error,
                            on_key_select: move |key: String| {
                                selected_key.set(key);
                                current_tab.set(Tab::Data);
                            },
                        }
                    }
                } else if current_tab() == Tab::Terminal {
                    Terminal {
                        key: "{conn_id}",
                        connection_pool: pool.clone(),
                    }
                } else if current_tab() == Tab::Monitor {
                    MonitorPanel {
                        key: "{conn_id}",
                        connection_pool: pool.clone(),
                        auto_refresh_interval,
                    }
                } else if current_tab() == Tab::SlowLog {
                    SlowLogPanel {
                        key: "{conn_id}",
                        connection_pool: pool.clone(),
                    }
                } else if current_tab() == Tab::Clients {
                    ClientsPanel {
                        key: "{conn_id}",
                        connection_pool: pool.clone(),
                    }
                } else if current_tab() == Tab::PubSub {
                    PubSubPanel {
                        key: "{conn_id}",
                        connection_pool: pool.clone(),
                    }
                } else {
                    ScriptPanel {
                        key: "{conn_id}",
                        connection_pool: pool.clone(),
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn SettingsDialogSection(
    settings: AppSettings,
    colors: ThemeColors,
    resolved_theme_id: ThemeId,
    on_change: EventHandler<AppSettings>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        SettingsDialog {
            settings,
            colors,
            resolved_theme_id,
            on_change,
            on_close,
        }
    }
}

#[component]
pub(super) fn FlushDialogSection(
    pool: ConnectionPool,
    current_db: u8,
    colors: ThemeColors,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        FlushConfirmDialog {
            connection_pool: pool,
            current_db,
            colors,
            on_confirm,
            on_cancel,
        }
    }
}

#[component]
pub(super) fn ImportOverlaySection(pool: ConnectionPool, on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            background: "rgba(0, 0, 0, 0.5)",
            display: "flex",
            align_items: "center",
            justify_content: "center",
            z_index: "1000",

            ImportPanel {
                connection_pool: pool,
                on_close,
            }
        }
    }
}

#[component]
pub(super) fn ExportConnectionsDialogSection(
    config_storage: Arc<ConfigStorage>,
    colors: ThemeColors,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        ConnectionExportDialog {
            config_storage,
            colors,
            on_close,
        }
    }
}

#[component]
pub(super) fn ImportConnectionsDialogSection(
    config_storage: Arc<ConfigStorage>,
    colors: ThemeColors,
    on_import: EventHandler<usize>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        ConnectionImportDialog {
            config_storage,
            colors,
            on_import,
            on_close,
        }
    }
}
