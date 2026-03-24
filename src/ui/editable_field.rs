use crate::theme::{
    COLOR_BG, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_SUCCESS, COLOR_TEXT, COLOR_TEXT_CONTRAST,
    COLOR_TEXT_SECONDARY,
};
use crate::ui::{copy_text_to_clipboard, icons::IconCopy, ToastManager};
use dioxus::prelude::*;

fn copy_to_clipboard(value: &str) -> Result<(), String> {
    copy_text_to_clipboard(value)
}

#[derive(Clone, PartialEq)]
pub enum EditorMode {
    View,
    EditString,
    EditTTL,
}

#[component]
pub fn EditableField(
    label: String,
    value: String,
    on_change: EventHandler<String>,
    editable: bool,
    multiline: bool,
) -> Element {
    let mut is_editing = use_signal(|| false);
    let mut temp_value = use_signal(String::new);
    let mut toast_manager = use_context::<Signal<ToastManager>>();

    rsx! {
        div {
            height: "100%",
            display: "flex",
            flex_direction: "column",

            if is_editing() {
                div {
                    display: "flex",
                    gap: "8px",

                    if multiline {
                        textarea {
                            flex: "1",
                            padding: "8px",
                            background: COLOR_BG_TERTIARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "4px",
                            color: COLOR_TEXT,
                            font_family: "Consolas, monospace",
                            rows: "5",
                            value: "{temp_value}",
                            oninput: move |e| temp_value.set(e.value()),
                        }
                    } else {
                        input {
                            flex: "1",
                            padding: "8px",
                            background: COLOR_BG_TERTIARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "4px",
                            color: COLOR_TEXT,
                            value: "{temp_value}",
                            oninput: move |e| temp_value.set(e.value()),
                        }
                    }

                    button {
                        padding: "6px 12px",
                        background: "#38a169",
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        onclick: move |_| {
                            on_change.call(temp_value());
                            is_editing.set(false);
                        },

                        "✓"
                    }

                    button {
                        padding: "6px 12px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        onclick: move |_| is_editing.set(false),

                        "✗"
                    }
                }
            } else {
                div {
                    display: "flex",
                    justify_content: "space_between",
                    align_items: "center",
                    margin_bottom: "8px",
                    flex_shrink: "0",

                    if !label.is_empty() {
                        label {
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "12px",

                            "{label}"
                        }
                    }

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",

                        button {
                            padding: "6px 10px",
                            background: "rgba(47, 133, 90, 0.16)",
                            color: COLOR_SUCCESS,
                            border: "1px solid rgba(104, 211, 145, 0.28)",
                            border_radius: "6px",
                            cursor: "pointer",
                            display: "flex",
                            align_items: "center",
                            gap: "4px",
                            title: "复制",
                            onclick: {
                                let val = value.clone();
                                move |_| match copy_to_clipboard(&val) {
                                    Ok(_) => {
                                        toast_manager.write().success("已复制到剪贴板");
                                    }
                                    Err(e) => {
                                        toast_manager.write().error(&format!("复制失败：{}", e));
                                    }
                                }
                            },

                            IconCopy { size: Some(14) }
                            "复制"
                        }

                        if editable {
                            {
                                let val = value.clone();
                                rsx! {
                                    button {
                                        padding: "6px 12px",
                                        background: "#3182ce",
                                        color: COLOR_TEXT_CONTRAST,
                                        border: "none",
                                        border_radius: "4px",
                                        cursor: "pointer",
                                        onclick: move |_| {
                                            temp_value.set(val.clone());
                                            is_editing.set(true);
                                        },

                                        "✏️"
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    flex: "1",
                    min_height: "0",
                    padding: "8px",
                    background: COLOR_BG,
                    border_radius: "4px",
                    color: COLOR_TEXT,
                    font_family: if multiline { "Consolas, monospace" } else { "inherit" },
                    overflow: "auto",
                    white_space: if multiline { "pre-wrap" } else { "nowrap" },
                    word_break: if multiline { "break-all" } else { "normal" },

                    "{value}"
                }
            }
        }
    }
}
