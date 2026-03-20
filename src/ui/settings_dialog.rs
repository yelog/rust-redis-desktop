use crate::config::AppSettings;
use dioxus::prelude::*;

#[component]
pub fn SettingsDialog(
    settings: AppSettings,
    on_save: EventHandler<AppSettings>,
    on_close: EventHandler<()>,
) -> Element {
    let mut auto_refresh_interval = use_signal(|| settings.auto_refresh_interval);

    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            background: "rgba(0, 0, 0, 0.7)",
            display: "flex",
            align_items: "center",
            justify_content: "center",
            z_index: "1000",
            onclick: move |_| on_close.call(()),

            div {
                width: "400px",
                padding: "24px",
                background: "#1e1e1e",
                border_radius: "8px",
                box_shadow: "0 4px 24px rgba(0, 0, 0, 0.5)",

                onclick: move |evt| {
                    evt.stop_propagation();
                },

                h2 {
                    color: "white",
                    margin_bottom: "24px",
                    font_size: "20px",

                    "设置"
                }

                div {
                    margin_bottom: "24px",

                    div {
                        color: "#888",
                        font_size: "13px",
                        margin_bottom: "8px",

                        "服务器信息自动刷新"
                    }

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "12px",

                        select {
                            flex: "1",
                            padding: "8px 12px",
                            background: "#2d2d2d",
                            border: "1px solid #3c3c3c",
                            border_radius: "4px",
                            color: "white",
                            font_size: "13px",
                            value: "{auto_refresh_interval}",
                            onchange: move |e| {
                                if let Ok(v) = e.value().parse() {
                                    auto_refresh_interval.set(v);
                                }
                            },

                            option { value: "0", "关闭" }
                            option { value: "5", "5 秒" }
                            option { value: "10", "10 秒" }
                            option { value: "30", "30 秒" }
                            option { value: "60", "60 秒" }
                        }

                        if auto_refresh_interval() > 0 {
                            span {
                                color: "#4ec9b0",
                                font_size: "12px",

                                "每 {auto_refresh_interval} 秒刷新"
                            }
                        }
                    }
                }

                div {
                    display: "flex",
                    justify_content: "flex_end",
                    gap: "12px",

                    button {
                        padding: "8px 16px",
                        background: "#3c3c3c",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| on_close.call(()),

                        "取消"
                    }

                    button {
                        padding: "8px 16px",
                        background: "#007acc",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "13px",
                        onclick: move |_| {
                            on_save.call(AppSettings {
                                auto_refresh_interval: auto_refresh_interval(),
                            });
                            on_close.call(());
                        },

                        "保存"
                    }
                }
            }
        }
    }
}
