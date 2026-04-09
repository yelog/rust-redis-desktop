use crate::i18n::use_i18n;
use crate::theme::ThemeColors;
use crate::ui::icons::{IconBell, IconHelpCircle, IconSearch, IconSettings};
use dioxus::prelude::*;

#[component]
pub fn TopNav(colors: ThemeColors, on_open_settings: EventHandler<()>) -> Element {
    let i18n = use_i18n();
    rsx! {
        div {
            height: "56px",
            background: "{colors.background}",
            border_bottom: "1px solid {colors.border}",
            display: "flex",
            align_items: "center",
            justify_content: "space_between",
            padding: "0 20px",
            box_sizing: "border-box",
            gap: "16px",

            div {
                display: "flex",
                align_items: "center",
                gap: "28px",

                div {
                    color: "{colors.primary}",
                    font_size: "18px",
                    font_weight: "800",
                    letter_spacing: "-0.04em",

                    "REDIS ENGINE"
                }

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

                    for label in ["Cluster", "Backups", "Metrics"] {
                        button {
                            padding: "8px 10px",
                            background: "transparent",
                            border: "none",
                            color: "{colors.text_secondary}",
                            cursor: "pointer",
                            font_size: "13px",
                            font_weight: "600",

                            "{label}"
                        }
                    }
                }
            }

            div {
                display: "flex",
                align_items: "center",
                gap: "10px",

                div {
                    width: "320px",
                    max_width: "32vw",
                    height: "38px",
                    background: "{colors.surface_lowest}",
                    border: "1px solid {colors.outline_variant}",
                    border_radius: "6px",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    padding: "0 12px",
                    box_sizing: "border-box",

                    IconSearch { size: Some(15) }

                    input {
                        flex: "1",
                        background: "transparent",
                        border: "none",
                        color: "{colors.text}",
                        font_size: "13px",
                        value: "",
                        placeholder: i18n.read().t("Global search (coming soon)"),
                    }

                    span {
                        color: "{colors.text_subtle}",
                        font_size: "11px",

                        "⌘K"
                    }
                }

                button {
                    width: "36px",
                    height: "36px",
                    background: "transparent",
                    border: "1px solid transparent",
                    border_radius: "6px",
                    color: "{colors.text_secondary}",
                    cursor: "pointer",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    onclick: move |_| on_open_settings.call(()),

                    IconSettings { size: Some(16) }
                }

                button {
                    width: "36px",
                    height: "36px",
                    background: "transparent",
                    border: "1px solid transparent",
                    border_radius: "6px",
                    color: "{colors.text_secondary}",
                    cursor: "pointer",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",

                    IconHelpCircle { size: Some(16) }
                }

                button {
                    width: "36px",
                    height: "36px",
                    background: "transparent",
                    border: "1px solid transparent",
                    border_radius: "6px",
                    color: "{colors.text_secondary}",
                    cursor: "pointer",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",

                    IconBell { size: Some(16) }
                }
            }
        }
    }
}
