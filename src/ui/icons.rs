use dioxus::prelude::*;

#[component]
pub fn IconCopy(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            rect {
                x: "9",
                y: "9",
                width: "13",
                height: "13",
                rx: "2",
                ry: "2",
            }
            path {
                d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1",
            }
        }
    }
}

#[component]
pub fn IconEdit(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7",
            }
            path {
                d: "M18.375 2.625a1.5 1.5 0 1 1 3 3L12 15l-4 1 1-4Z",
            }
        }
    }
}

#[component]
pub fn IconTrash(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path { d: "M3 6h18" }
            path { d: "M8 6V4h8v2" }
            path { d: "M19 6l-1 14H6L5 6" }
            path { d: "M10 11v6" }
            path { d: "M14 11v6" }
        }
    }
}

#[component]
pub fn IconFile(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z",
            }
            polyline {
                points: "14 2 14 8 20 8",
            }
        }
    }
}

#[component]
pub fn IconFolder(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z",
            }
        }
    }
}

#[component]
pub fn IconList(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            line { x1: "8", y1: "6", x2: "21", y2: "6" }
            line { x1: "8", y1: "12", x2: "21", y2: "12" }
            line { x1: "8", y1: "18", x2: "21", y2: "18" }
            line { x1: "3", y1: "6", x2: "3.01", y2: "6" }
            line { x1: "3", y1: "12", x2: "3.01", y2: "12" }
            line { x1: "3", y1: "18", x2: "3.01", y2: "18" }
        }
    }
}

#[component]
pub fn IconCheck(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            polyline {
                points: "20 6 9 17 4 12",
            }
        }
    }
}

#[component]
pub fn IconSettings(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            circle {
                cx: "12",
                cy: "12",
                r: "3",
            }
            path {
                d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z",
            }
        }
    }
}

#[component]
pub fn IconRefresh(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8",
            }
            path {
                d: "M3 3v5h5",
            }
            path {
                d: "M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16",
            }
            path {
                d: "M16 16h5v5",
            }
        }
    }
}

#[component]
pub fn IconX(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            line { x1: "18", y1: "6", x2: "6", y2: "18" }
            line { x1: "6", y1: "6", x2: "18", y2: "18" }
        }
    }
}

#[component]
pub fn IconAlert(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z",
            }
            line { x1: "12", y1: "9", x2: "12", y2: "13" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        }
    }
}

#[component]
pub fn IconDatabase(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            ellipse {
                cx: "12",
                cy: "5",
                rx: "9",
                ry: "3",
            }
            path {
                d: "M21 12c0 1.66-4 3-9 3s-9-1.34-9-3",
            }
            path {
                d: "M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5",
            }
        }
    }
}

#[component]
pub fn IconHash(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            line { x1: "4", y1: "9", x2: "20", y2: "9" }
            line { x1: "4", y1: "15", x2: "20", y2: "15" }
            line { x1: "10", y1: "3", x2: "8", y2: "21" }
            line { x1: "16", y1: "3", x2: "14", y2: "21" }
        }
    }
}

#[component]
pub fn IconSet(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            circle {
                cx: "12",
                cy: "12",
                r: "10",
            }
            circle {
                cx: "12",
                cy: "12",
                r: "4",
            }
            line { x1: "4.93", y1: "4.93", x2: "9.17", y2: "9.17" }
            line { x1: "14.83", y1: "14.83", x2: "19.07", y2: "19.07" }
            line { x1: "14.83", y1: "9.17", x2: "19.07", y2: "4.93" }
            line { x1: "4.93", y1: "19.07", x2: "9.17", y2: "14.83" }
        }
    }
}

#[component]
pub fn IconZSet(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M3 3v18h18",
            }
            path {
                d: "M7 17l4-8 4 8",
            }
            line { x1: "8.5", y1: "14", x2: "13.5", y2: "14" }
        }
    }
}

#[component]
pub fn IconStream(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M22 12h-4l-3 9L9 3l-3 9H2",
            }
        }
    }
}

#[component]
pub fn IconPlus(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            line { x1: "12", y1: "5", x2: "12", y2: "19" }
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}

#[component]
pub fn IconSearch(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            circle {
                cx: "11",
                cy: "11",
                r: "8",
            }
            line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
        }
    }
}

#[component]
pub fn IconKey(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path {
                d: "M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4",
            }
        }
    }
}

#[component]
pub fn IconBell(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path { d: "M15 17h5l-1.4-1.4A2 2 0 0 1 18 14.2V11a6 6 0 1 0-12 0v3.2a2 2 0 0 1-.6 1.4L4 17h5" }
            path { d: "M9 17a3 3 0 0 0 6 0" }
        }
    }
}

#[component]
pub fn IconHelpCircle(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M9.09 9a3 3 0 1 1 5.82 1c0 2-3 2-3 4" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        }
    }
}

#[component]
pub fn IconTerminal(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path { d: "M4 17l6-6-6-6" }
            path { d: "M12 19h8" }
        }
    }
}

#[component]
pub fn IconActivity(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            polyline { points: "22 12 18 12 15 21 9 3 6 12 2 12" }
        }
    }
}

#[component]
pub fn IconUsers(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path { d: "M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" }
            circle { cx: "8.5", cy: "7", r: "4" }
            path { d: "M20 8v6" }
            path { d: "M23 11h-6" }
        }
    }
}

#[component]
pub fn IconClock(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            circle { cx: "12", cy: "12", r: "10" }
            polyline { points: "12 6 12 12 16 14" }
        }
    }
}

#[component]
pub fn IconDownload(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "7 10 12 15 17 10" }
            line { x1: "12", y1: "15", x2: "12", y2: "3" }
        }
    }
}

#[component]
pub fn IconUpload(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "17 8 12 3 7 8" }
            line { x1: "12", y1: "3", x2: "12", y2: "15" }
        }
    }
}

#[component]
pub fn IconMoreHorizontal(size: Option<i32>, color: Option<String>) -> Element {
    let size = size.unwrap_or(16);
    let color = color.unwrap_or_else(|| "currentColor".to_string());

    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            circle { cx: "5", cy: "12", r: "1" }
            circle { cx: "12", cy: "12", r: "1" }
            circle { cx: "19", cy: "12", r: "1" }
        }
    }
}
