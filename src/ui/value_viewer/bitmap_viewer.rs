use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::theme::{
    COLOR_BG, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_INFO, COLOR_INFO_BG, COLOR_PRIMARY,
    COLOR_SUCCESS, COLOR_SUCCESS_BG, COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
    COLOR_TEXT_SUBTLE,
};
use dioxus::prelude::*;

#[component]
pub fn BitmapViewer(
    info: crate::redis::BitmapInfo,
    pool: ConnectionPool,
    redis_key: String,
    on_update: EventHandler<()>,
) -> Element {
    let i18n = use_i18n();
    let mut editing_offset = use_signal(String::new);
    let mut editing_value = use_signal(|| "1".to_string());

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "16px",

            div {
                display: "flex",
                gap: "16px",
                flex_wrap: "wrap",

                div {
                    padding: "8px 12px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",
                        {i18n.read().t("Total bytes: ")}
                    }
                    span {
                        color: COLOR_TEXT,
                        font_size: "12px",
                        font_weight: "600",
                        "{info.total_bytes}"
                    }
                }

                div {
                    padding: "8px 12px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",
                        {i18n.read().t("Total bits: ")}
                    }
                    span {
                        color: COLOR_TEXT,
                        font_size: "12px",
                        font_weight: "600",
                        "{info.total_bits}"
                    }
                }

                div {
                    padding: "8px 12px",
                    background: COLOR_SUCCESS_BG,
                    border_radius: "6px",

                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",
                        {i18n.read().t("Set bits: ")}
                    }
                    span {
                        color: COLOR_SUCCESS,
                        font_size: "12px",
                        font_weight: "600",
                        "{info.set_bits_count}"
                    }
                }
            }

            div {
                span {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",
                    font_weight: "600",
                    margin_bottom: "8px",
                    display: "block",
                    {i18n.read().t("Set bit offsets:")}
                }

                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "6px",
                    max_height: "120px",
                    overflow_y: "auto",
                    padding: "8px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    for offset in info.set_bits.iter().take(200) {
                        span {
                            padding: "2px 8px",
                            background: COLOR_INFO_BG,
                            color: COLOR_INFO,
                            border_radius: "4px",
                            font_size: "11px",
                            font_family: "Consolas, monospace",
                            "{offset}"
                        }
                    }
                    if info.set_bits.len() > 200 {
                        span {
                            padding: "2px 8px",
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",
                            {format!("... {} {}", info.set_bits.len() - 200, i18n.read().t("more"))}
                        }
                    }
                }
            }

            div {
                span {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",
                    font_weight: "600",
                    margin_bottom: "8px",
                    display: "block",
                    {i18n.read().t("Binary view:")}
                }

                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "4px",
                    font_family: "Consolas, monospace",
                    font_size: "11px",
                    max_height: "200px",
                    overflow_y: "auto",
                    padding: "8px",
                    background: COLOR_BG_TERTIARY,
                    border_radius: "6px",

                    for (byte_idx, byte) in info.raw_bytes.iter().enumerate().take(64) {
                        div {
                            display: "flex",
                            flex_direction: "column",
                            align_items: "center",
                            gap: "2px",

                            div {
                                display: "flex",
                                gap: "1px",

                                for bit_idx in 0..8 {
                                    { let bit_val = (*byte >> (7 - bit_idx)) & 1; rsx! {
                                        div {
                                            width: "12px",
                                            height: "12px",
                                            background: if bit_val == 1 { COLOR_SUCCESS } else { COLOR_BG },
                                            border_radius: "2px",
                                        }
                                    }}
                                }
                            }

                            span {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "9px",
                                "{byte_idx}"
                            }
                        }
                    }
                    if info.raw_bytes.len() > 64 {
                        span {
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",
                            {format!("... {} {}", info.raw_bytes.len(), i18n.read().t("bytes total"))}
                        }
                    }
                }
            }

            div {
                span {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",
                    font_weight: "600",
                    margin_bottom: "8px",
                    display: "block",
                    {i18n.read().t("Set or update bit:")}
                }

                div {
                    display: "flex",
                    gap: "8px",
                    align_items: "center",

                    input {
                        width: "100px",
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        placeholder: "Offset",
                        value: "{editing_offset}",
                        oninput: move |e| editing_offset.set(e.value()),
                    }

                    select {
                        padding: "6px 10px",
                        background: COLOR_BG_TERTIARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "4px",
                        color: COLOR_TEXT,
                        font_size: "12px",
                        value: "{editing_value}",
                        onchange: move |e| editing_value.set(e.value()),

                        option { value: "0", {i18n.read().t("Set to 0")} }
                        option { value: "1", {i18n.read().t("Set to 1")} }
                    }

                    button {
                        padding: "6px 12px",
                        background: COLOR_PRIMARY,
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: {
                            let pool = pool.clone();
                            let redis_key = redis_key.clone();
                            move |_| {
                                let offset_str = editing_offset();
                                let value_str = editing_value();
                                if let Ok(offset) = offset_str.parse::<u64>() {
                                    let value = value_str == "1";
                                    let pool = pool.clone();
                                    let redis_key = redis_key.clone();
                                    spawn(async move {
                                        if pool.set_bit(&redis_key, offset, value).await.is_ok() {
                                            on_update.call(());
                                        }
                                    });
                                }
                            }
                        },
                        {i18n.read().t("Apply")}
                    }
                }
            }
        }
    }
}
