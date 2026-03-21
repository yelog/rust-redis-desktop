use dioxus::prelude::*;

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

    rsx! {
        div {
            margin_bottom: "12px",

            label {
                display: "block",
                color: "#888",
                font_size: "12px",
                margin_bottom: "4px",

                "{label}"
            }

            if is_editing() {
                div {
                    display: "flex",
                    gap: "8px",

                    if multiline {
                        textarea {
                            flex: "1",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            font_family: "Consolas, monospace",
                            rows: "5",
                            value: "{temp_value}",
                            oninput: move |e| temp_value.set(e.value()),
                        }
                    } else {
                        input {
                            flex: "1",
                            padding: "8px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{temp_value}",
                            oninput: move |e| temp_value.set(e.value()),
                        }
                    }

                    button {
                        padding: "6px 12px",
                        background: "#38a169",
                        color: "white",
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
                        background: "#5a5a5a",
                        color: "white",
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
                    align_items: "center",
                    gap: "8px",

                    div {
                        flex: "1",
                        padding: "8px",
                        background: "#2d2d2d",
                        border_radius: "4px",
                        color: "white",
                        font_family: if multiline { "Consolas, monospace" } else { "inherit" },
                        overflow: if multiline { "auto" } else { "hidden" },
                        text_overflow: if multiline { "unset" } else { "ellipsis" },
                        white_space: if multiline { "pre-wrap" } else { "nowrap" },
                        word_break: if multiline { "break-all" } else { "normal" },

                        "{value}"
                    }

                    if editable {
                        button {
                            padding: "6px 12px",
                            background: "#3182ce",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            onclick: move |_| {
                                temp_value.set(value.clone());
                                is_editing.set(true);
                            },

                            "✏️"
                        }
                    }
                }
            }
        }
    }
}
