use super::{base64_decode, image_preview_button_style, image_preview_info_chip_style};
use crate::theme::COLOR_OVERLAY_BACKDROP;
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

#[derive(Clone, Default)]
pub struct PreviewImageData {
    pub data_uri: String,
    pub format: String,
    pub size: String,
}

pub static PREVIEW_IMAGE: GlobalSignal<Option<PreviewImageData>> = Signal::global(|| None);

#[component]
pub fn ImagePreview() -> Element {
    let preview = PREVIEW_IMAGE();

    let Some(ref data) = preview else {
        return rsx! {};
    };

    let data_uri = data.data_uri.clone();
    let format = data.format.clone();
    let size = data.size.clone();

    let data_uri_for_save = data_uri.clone();
    let data_uri_for_img = data_uri.clone();
    let format_for_save = format.clone();
    let mut zoom_level = use_signal(|| 1.0f32);

    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            right: "0",
            bottom: "0",
            background: COLOR_OVERLAY_BACKDROP,
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            justify_content: "center",
            z_index: "9999",
            animation: "fadeIn 0.2s ease-out",

            onclick: move |_| {
                *PREVIEW_IMAGE.write() = None;
                zoom_level.set(1.0);
            },

            onkeydown: move |e: Event<KeyboardData>| {
                if e.data().key() == Key::Escape {
                    e.prevent_default();
                    e.stop_propagation();
                    *PREVIEW_IMAGE.write() = None;
                    zoom_level.set(1.0);
                }
            },

            style { {r#"
                @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                }
                @keyframes scaleIn {
                    from { transform: scale(0.9); opacity: 0; }
                    to { transform: scale(1); opacity: 1; }
                }
            "#} }

            div {
                position: "absolute",
                top: "16px",
                right: "16px",
                display: "flex",
                gap: "8px",
                z_index: "10",

                button {
                    style: "{image_preview_button_style()}",

                    onclick: move |e| {
                        e.stop_propagation();
                        let image_data = base64_decode(&data_uri_for_save);
                        let extension = format_for_save.to_lowercase();
                        let file_name = format!("image.{}", extension);

                        spawn(async move {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name(&file_name)
                                .add_filter("Image", &[&extension])
                                .save_file()
                            {
                                let _ = std::fs::write(&path, &image_data);
                            }
                        });
                    },

                    "保存图片"
                }

                button {
                    style: "{image_preview_button_style()}",

                    onclick: move |e| {
                        e.stop_propagation();
                        zoom_level.set(1.0);
                    },

                    "重置"
                }

                button {
                    style: "{image_preview_button_style()}",

                    onclick: move |e| {
                        e.stop_propagation();
                        *PREVIEW_IMAGE.write() = None;
                        zoom_level.set(1.0);
                    },

                    "关闭 (Esc)"
                }
            }

            div {
                width: "100vw",
                height: "100vh",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                animation: "scaleIn 0.2s ease-out",
                overflow: "hidden",

                onclick: |e| e.stop_propagation(),

                onwheel: move |e: Event<WheelData>| {
                    e.stop_propagation();
                    let delta = match e.delta() {
                        WheelDelta::Pixels(p) => {
                            if p.y > 0.0 { -0.1 } else { 0.1 }
                        }
                        WheelDelta::Lines(l) => {
                            if l.y > 0.0 { -0.1 } else { 0.1 }
                        }
                        WheelDelta::Pages(p) => {
                            if p.y > 0.0 { -0.1 } else { 0.1 }
                        }
                    };
                    let current = zoom_level();
                    let new_zoom = (current + delta).clamp(0.1, 5.0);
                    zoom_level.set(new_zoom);
                },

                img {
                    src: "{data_uri_for_img}",
                    max_width: "90vw",
                    max_height: "85vh",
                    object_fit: "contain",
                    transform: "scale({zoom_level})",
                    transform_origin: "center",
                    transition: "transform 0.1s ease-out",
                    draggable: false,
                }
            }

            div {
                position: "absolute",
                bottom: "24px",
                left: "50%",
                transform: "translateX(-50%)",
                display: "flex",
                gap: "16px",
                align_items: "center",

                div {
                    style: "{image_preview_info_chip_style()}",
                    "{format} - {size}"
                }

                div {
                    style: "{image_preview_info_chip_style()}",
                    "缩放: {(zoom_level() * 100.0) as i32}%"
                }
            }
        }
    }
}
