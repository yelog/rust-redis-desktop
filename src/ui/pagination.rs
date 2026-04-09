use crate::i18n::use_i18n;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct PageInfo {
    pub current: usize,
    pub total: usize,
    pub page_size: usize,
}

impl Default for PageInfo {
    fn default() -> Self {
        Self {
            current: 0,
            total: 0,
            page_size: 100,
        }
    }
}

impl PageInfo {
    pub fn total_pages(&self) -> usize {
        if self.total == 0 {
            0
        } else {
            (self.total + self.page_size - 1) / self.page_size
        }
    }

    pub fn start_index(&self) -> usize {
        self.current * self.page_size
    }

    pub fn end_index(&self) -> usize {
        ((self.current + 1) * self.page_size).min(self.total)
    }

    pub fn has_prev(&self) -> bool {
        self.current > 0
    }

    pub fn has_next(&self) -> bool {
        self.current + 1 < self.total_pages()
    }
}

#[component]
pub fn Pagination(page_info: PageInfo, on_page_change: EventHandler<usize>) -> Element {
    let _i18n = use_i18n();
    if page_info.total_pages() <= 1 {
        return rsx! {};
    }

    let total_pages = page_info.total_pages();
    let current = page_info.current;

    let mut pages_to_show = Vec::new();
    if total_pages <= 7 {
        for i in 0..total_pages {
            pages_to_show.push(i);
        }
    } else {
        if current < 3 {
            for i in 0..4 {
                pages_to_show.push(i);
            }
            pages_to_show.push(usize::MAX);
            pages_to_show.push(total_pages - 1);
        } else if current > total_pages - 4 {
            pages_to_show.push(0);
            pages_to_show.push(usize::MAX);
            for i in (total_pages - 4)..total_pages {
                pages_to_show.push(i);
            }
        } else {
            pages_to_show.push(0);
            pages_to_show.push(usize::MAX);
            for i in (current - 1)..=(current + 1) {
                pages_to_show.push(i);
            }
            pages_to_show.push(usize::MAX);
            pages_to_show.push(total_pages - 1);
        }
    }

    rsx! {
        div {
            display: "flex",
            align_items: "center",
            justify_content: "center",
            gap: "4px",
            padding: "8px",
            background: "#1e1e1e",
            border_top: "1px solid #3c3c3c",

            button {
                padding: "4px 8px",
                background: if page_info.has_prev() { "#3c3c3c" } else { "#2d2d2d" },
                color: if page_info.has_prev() { "white" } else { "#666" },
                border: "1px solid #555",
                border_radius: "4px",
                cursor: if page_info.has_prev() { "pointer" } else { "not-allowed" },
                font_size: "12px",
                disabled: !page_info.has_prev(),
                onclick: move |_| {
                    if page_info.has_prev() {
                        on_page_change.call(page_info.current - 1);
                    }
                },

                "◀"
            }

            for (idx, page) in pages_to_show.iter().enumerate() {
                if *page == usize::MAX {
                    span {
                        key: "ellipsis-{idx}",
                        color: "#888",
                        padding: "0 4px",

                        "..."
                    }
                } else {
                    button {
                        key: "page-{page}",
                        padding: "4px 10px",
                        background: if *page == current { "#0e639c" } else { "#3c3c3c" },
                        color: "white",
                        border: "1px solid #555",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: {
                            let page = *page;
                            move |_| on_page_change.call(page)
                        },

                        "{page + 1}"
                    }
                }
            }

            button {
                padding: "4px 8px",
                background: if page_info.has_next() { "#3c3c3c" } else { "#2d2d2d" },
                color: if page_info.has_next() { "white" } else { "#666" },
                border: "1px solid #555",
                border_radius: "4px",
                cursor: if page_info.has_next() { "pointer" } else { "not-allowed" },
                font_size: "12px",
                disabled: !page_info.has_next(),
                onclick: move |_| {
                    if page_info.has_next() {
                        on_page_change.call(page_info.current + 1);
                    }
                },

                "▶"
            }

            span {
                color: "#888",
                font_size: "12px",
                margin_left: "8px",

                "{page_info.start_index() + 1}-{page_info.end_index()} / {page_info.total}"
            }
        }
    }
}

#[component]
pub fn LargeKeyWarning(key_type: String, size: usize, threshold: usize) -> Element {
    let i18n = use_i18n();
    rsx! {
        div {
            display: "flex",
            align_items: "center",
            gap: "8px",
            padding: "8px 12px",
            background: "rgba(245, 158, 11, 0.1)",
            border: "1px solid rgba(245, 158, 11, 0.3)",
            border_radius: "6px",
            margin_bottom: "12px",

            span {
                color: "#f59e0b",
                font_size: "13px",

                {format!(
                    "{}: {} {} {}, {} ({})",
                    i18n.read().t("Large key warning"),
                    key_type,
                    i18n.read().t("contains"),
                    size,
                    i18n.read().t("threshold exceeded"),
                    threshold
                )}
            }
        }
    }
}
