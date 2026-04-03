use crate::config::{AppSettings, ConfigStorage};
use crate::connection::ConnectionPool;
use crate::theme::{
    ThemeColors, ThemeId, COLOR_ACCENT, COLOR_ERROR, COLOR_SURFACE_LOW, COLOR_TEXT,
    COLOR_TEXT_SECONDARY,
};
use crate::ui::{
    ConnectionExportDialog, ConnectionImportDialog, FlushConfirmDialog, ImportPanel, SettingsDialog,
};
use dioxus::prelude::*;
use std::sync::Arc;

pub(super) fn spinner_panel(message: &'static str) -> Element {
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
                "Redis 工作台"
            }

            div {
                font_size: "14px",
                "从左侧选择一个连接，或先创建新的 Redis 连接。"
            }
        }
    }
}

pub(super) fn connection_error_panel(on_retry: EventHandler<MouseEvent>) -> Element {
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
                "连接失败，请检查连接配置后重试"
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
                "重新连接"
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
