use crate::config::ConfigStorage;
use crate::connection::ConnectionConfig;
use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use crate::ui::clipboard::copy_text_to_clipboard;
use crate::ui::icons::{IconDownload, IconUpload};
use crate::ui::ToastManager;
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportedConnections {
    pub version: String,
    pub exported_at: String,
    pub connections: Vec<ConnectionConfig>,
}

#[derive(Clone, PartialEq)]
enum ExportState {
    Idle,
    Exporting,
    Success(String),
    Error(String),
}

#[component]
pub fn ConnectionExportDialog(
    config_storage: Arc<ConfigStorage>,
    colors: ThemeColors,
    on_close: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let mut state = use_signal(|| ExportState::Exporting);
    let mut started = use_signal(|| false);

    use_effect({
        let config_storage = config_storage.clone();
        move || {
            if !started() {
                started.set(true);
                let config_storage = config_storage.clone();
                spawn(async move {
                    let result = async {
                        let connections = config_storage
                            .load_connections()
                            .map_err(|e| format!("{}{}", i18n.read().t("Failed to load: "), e))?;

                        let exported = ExportedConnections {
                            version: "1.0".to_string(),
                            exported_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            connections,
                        };

                        serde_json::to_string_pretty(&exported).map_err(|e| {
                            format!("{}{}", i18n.read().t("Serialization failed: "), e)
                        })
                    }
                    .await;

                    state.set(match result {
                        Ok(json) => ExportState::Success(json),
                        Err(e) => ExportState::Error(e),
                    });
                });
            }
        }
    });

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "500px".to_string(),
            max_height: "80vh".to_string(),

            h3 {
                color: "{colors.text}",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "8px",
                font_size: "18px",

                IconDownload { size: Some(20) }
                {i18n.read().t("Export connections")}
            }

            match state() {
                ExportState::Exporting => rsx! {
                    div {
                        padding: "40px",
                        text_align: "center",
                        color: "{colors.text_secondary}",

                        {i18n.read().t("Exporting...")}
                    }
                },
                ExportState::Success(ref json) => rsx! {
                    ExportSuccessView {
                        json: json.clone(),
                        colors,
                        on_close: on_close.clone(),
                    }
                },
                ExportState::Error(ref e) => rsx! {
                    div {
                        padding: "20px",
                        background: "rgba(239, 68, 68, 0.1)",
                        border: "1px solid rgba(239, 68, 68, 0.3)",
                        border_radius: "4px",
                        color: "{colors.error}",
                        font_size: "13px",

                        "{e}"
                    }
                },
                ExportState::Idle => rsx! {},
            }
        }
    }
}

#[component]
fn ExportSuccessView(json: String, colors: ThemeColors, on_close: EventHandler<()>) -> Element {
    let i18n = use_i18n();
    let mut toast_manager = use_context::<Signal<ToastManager>>();

    let copy_to_clipboard = {
        let json = json.clone();
        move |_| {
            if copy_text_to_clipboard(&json).is_ok() {
                toast_manager
                    .write()
                    .success(&i18n.read().t("Copied to clipboard"));
            }
        }
    };

    let export_to_file = {
        let json = json.clone();
        move |_| {
            let json = json.clone();
            spawn(async move {
                let file_path = rfd::AsyncFileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("connections.json")
                    .save_file()
                    .await;

                if let Some(path) = file_path {
                    match std::fs::write(path.path(), &json) {
                        Ok(_) => {
                            toast_manager
                                .write()
                                .success(&i18n.read().t("Exported to file"));
                        }
                        Err(e) => {
                            toast_manager.write().error(&format!(
                                "{}{}",
                                i18n.read().t("Failed to save: "),
                                e
                            ));
                        }
                    }
                }
            });
        }
    };

    rsx! {
        div {
            color: "{colors.text_secondary}",
            margin_bottom: "12px",
            font_size: "13px",

            {i18n.read().t("Export successful! Copy the content below:")}
        }

        div {
            background: "{colors.background_tertiary}",
            border: "1px solid {colors.border}",
            border_radius: "4px",
            padding: "12px",
            margin_bottom: "16px",
            max_height: "300px",
            overflow_y: "auto",

            pre {
                color: "{colors.text}",
                font_size: "11px",
                font_family: "monospace",
                white_space: "pre_wrap",
                word_break: "break_all",
                margin: "0",

                "{json}"
            }
        }

        div {
            display: "flex",
            gap: "8px",

            button {
                flex: "1",
                padding: "10px",
                background: "{colors.primary}",
                color: "{colors.primary_text}",
                border: "none",
                border_radius: "4px",
                cursor: "pointer",
                font_size: "13px",

                onclick: copy_to_clipboard,

                {i18n.read().t("Copy to clipboard")}
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
                onclick: export_to_file,

                {i18n.read().t("Export as file")}
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum ImportState {
    Idle,
    Importing,
    Success(usize),
}

#[component]
pub fn ConnectionImportDialog(
    config_storage: Arc<ConfigStorage>,
    colors: ThemeColors,
    on_import: EventHandler<usize>,
    on_close: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let mut import_text = use_signal(String::new);
    let mut state = use_signal(|| ImportState::Idle);
    let mut preview = use_signal(|| None::<Vec<ConnectionConfig>>);
    let mut error_msg = use_signal(|| None::<String>);

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_close.clone(),
            colors,
            width: "500px".to_string(),
            max_height: "80vh".to_string(),

            h3 {
                color: "{colors.text}",
                margin_bottom: "16px",
                display: "flex",
                align_items: "center",
                gap: "8px",
                font_size: "18px",

                IconUpload { size: Some(20) }
                {i18n.read().t("Import connections")}
            }

            match state() {
                ImportState::Success(count) => rsx! {
                    div {
                        padding: "20px",
                        text_align: "center",

                        div {
                            color: "{colors.accent}",
                            font_size: "16px",
                            font_weight: "600",
                            margin_bottom: "12px",

                            {format!("{} {}", i18n.read().t("Successfully imported connections:"), count)}
                        }

                        button {
                            padding: "10px 24px",
                            background: "{colors.primary}",
                            color: "{colors.primary_text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: move |_| on_close.call(()),

                            {i18n.read().t("Close")}
                        }
                    }
                },
                ImportState::Importing => rsx! {
                    div {
                        padding: "40px",
                        text_align: "center",
                        color: "{colors.text_secondary}",

                        {i18n.read().t("Importing...")}
                    }
                },
                ImportState::Idle => rsx! {
                    div {
                        color: "{colors.text_secondary}",
                        margin_bottom: "12px",
                        font_size: "13px",

                        {i18n.read().t("Paste the exported JSON content:")}
                    }

                    textarea {
                        width: "100%",
                        height: "150px",
                        padding: "10px 12px",
                        background: "{colors.background_tertiary}",
                        border: "1px solid {colors.border}",
                        border_radius: "4px",
                        color: "{colors.text}",
                        font_size: "12px",
                        font_family: "monospace",
                        box_sizing: "border_box",
                        resize: "vertical",
                        value: "{import_text}",
                        oninput: move |e| {
                            import_text.set(e.value());
                            preview.set(None);
                            error_msg.set(None);
                        },
                    }

                    if let Some(connections) = preview() {
                        div {
                            margin_top: "12px",
                            padding: "12px",
                            background: "rgba(34, 197, 94, 0.1)",
                            border: "1px solid rgba(34, 197, 94, 0.3)",
                            border_radius: "4px",

                            div {
                                color: "#22c55e",
                                font_size: "13px",
                                margin_bottom: "8px",

                                {format!("{} {}", i18n.read().t("Found connections:"), connections.len())}
                            }

                            div {
                                max_height: "120px",
                                overflow_y: "auto",

                                for config in connections.iter() {
                                    div {
                                        color: "{colors.text}",
                                        font_size: "12px",
                                        padding: "4px 0",

                                        "{config.name} ({config.host}:{config.port})"
                                    }
                                }
                            }
                        }
                    }

                    if let Some(e) = error_msg() {
                        div {
                            margin_top: "12px",
                            padding: "12px",
                            background: "rgba(239, 68, 68, 0.1)",
                            border: "1px solid rgba(239, 68, 68, 0.3)",
                            border_radius: "4px",
                            color: "{colors.error}",
                            font_size: "13px",

                            "{e}"
                        }
                    }

                    div {
                        display: "flex",
                        gap: "8px",
                        margin_top: "16px",

                        button {
                            flex: "1",
                            padding: "10px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "1px solid {colors.border}",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: {
                                let import_text = import_text.clone();
                                move |_| {
                                    let text = import_text();
                                    match serde_json::from_str::<ExportedConnections>(&text) {
                                        Ok(exported) => {
                                            preview.set(Some(exported.connections));
                                            error_msg.set(None);
                                        }
                                        Err(e) => {
                                            preview.set(None);
                                            error_msg.set(Some(format!("{}{}", i18n.read().t("Parse error: "), e)));
                                        }
                                    }
                                }
                            },

                            {i18n.read().t("Preview")}
                        }

                        button {
                            flex: "1",
                            padding: "10px",
                            background: "{colors.primary}",
                            color: "{colors.primary_text}",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            disabled: preview().is_none(),
                            onclick: {
                                let config_storage = config_storage.clone();
                                let preview = preview.clone();
                                move |_| {
                                    let config_storage = config_storage.clone();
                                    let preview = preview.clone();
                                    state.set(ImportState::Importing);

                                    spawn(async move {
                                        if let Some(connections) = preview.read().as_ref() {
                                            let mut imported = 0;
                                            for config in connections {
                                                if config_storage.save_connection(config.clone()).is_ok() {
                                                    imported += 1;
                                                }
                                            }
                                            on_import.call(imported);
                                            state.set(ImportState::Success(imported));
                                        }
                                    });
                                }
                            },

                            {i18n.read().t("Import")}
                        }

                        button {
                            padding: "10px 16px",
                            background: "{colors.background_tertiary}",
                            color: "{colors.text}",
                            border: "1px solid {colors.border}",
                            border_radius: "4px",
                            cursor: "pointer",
                            font_size: "13px",
                            onclick: move |_| on_close.call(()),

                            {i18n.read().t("Cancel")}
                        }
                    }
                },
            }
        }
    }
}
