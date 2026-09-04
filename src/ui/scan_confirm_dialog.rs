use crate::config::KeyScanMode;
use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::animated_dialog::AnimatedDialog;
use dioxus::prelude::*;

#[component]
pub fn ScanConfirmDialog(
    current_db: u8,
    db_size: Option<u64>,
    threshold: u64,
    progressive_limit: usize,
    scan_mode: KeyScanMode,
    colors: ThemeColors,
    on_progressive: EventHandler<()>,
    on_complete: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let size_label = db_size
        .map(|size| size.to_string())
        .unwrap_or_else(|| i18n.read().t("Unable to determine"));
    let default_action_is_progressive = scan_mode == KeyScanMode::Progressive;

    rsx! {
        AnimatedDialog {
            is_open: true,
            on_close: on_cancel.clone(),
            colors,
            width: "500px".to_string(),

            h3 {
                color: "{colors.warning}",
                margin_bottom: "12px",
                font_size: "18px",
                {i18n.read().t("Confirm large database scan")}
            }

            div {
                color: "{colors.text_secondary}",
                font_size: "13px",
                line_height: "1.5",
                margin_bottom: "14px",
                {format!(
                    "{}: {} · {}: {}",
                    i18n.read().t("Database"),
                    current_db,
                    i18n.read().t("Key count"),
                    size_label
                )}
            }

            div {
                color: "{colors.warning}",
                background: "{colors.background_tertiary}",
                border: "1px solid {colors.border}",
                border_radius: "6px",
                padding: "12px",
                margin_bottom: "16px",
                font_size: "12px",
                line_height: "1.5",
                {format!(
                    "{} {} {}. {}",
                    i18n.read().t("This database reaches the scan confirmation threshold"),
                    threshold,
                    i18n.read().t("keys"),
                    i18n.read().t("Even SCAN can consume Redis CPU, network bandwidth, and local resources during a complete traversal.")
                )}
            }

            div {
                display: "flex",
                gap: "8px",

                button {
                    flex: "1",
                    padding: "9px 10px",
                    background: if default_action_is_progressive { colors.primary } else { colors.background_tertiary },
                    color: if default_action_is_progressive { colors.primary_text } else { colors.text },
                    border: "1px solid {colors.border}",
                    border_radius: "6px",
                    cursor: "pointer",
                    onclick: move |_| on_progressive.call(()),
                    {format!("{} ({} {})", i18n.read().t("Scan progressively"), progressive_limit, i18n.read().t("keys"))}
                }

                button {
                    flex: "1",
                    padding: "9px 10px",
                    background: if !default_action_is_progressive { colors.primary } else { colors.background_tertiary },
                    color: if !default_action_is_progressive { colors.primary_text } else { colors.text },
                    border: "1px solid {colors.border}",
                    border_radius: "6px",
                    cursor: "pointer",
                    onclick: move |_| on_complete.call(()),
                    {i18n.read().t("Complete scan")}
                }

                button {
                    padding: "9px 12px",
                    background: "transparent",
                    color: "{colors.text_secondary}",
                    border: "1px solid {colors.border}",
                    border_radius: "6px",
                    cursor: "pointer",
                    onclick: move |_| on_cancel.call(()),
                    {i18n.read().t("Cancel")}
                }
            }
        }
    }
}
