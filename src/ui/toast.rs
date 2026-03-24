use crate::theme::{COLOR_ERROR, COLOR_SUCCESS};
use dioxus::prelude::*;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use uuid::Uuid;

const TOAST_DURATION_SECS: u64 = 2;
const MAX_TOASTS: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
}

#[derive(Clone)]
pub struct Toast {
    pub id: Uuid,
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
}

impl PartialEq for Toast {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Default)]
pub struct ToastManager {
    toasts: VecDeque<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: VecDeque::new(),
        }
    }

    pub fn show(&mut self, message: &str, toast_type: ToastType) {
        let toast = Toast {
            id: Uuid::new_v4(),
            message: message.to_string(),
            toast_type,
            created_at: Instant::now(),
        };

        if self.toasts.len() >= MAX_TOASTS {
            self.toasts.pop_front();
        }
        self.toasts.push_back(toast);
    }

    pub fn success(&mut self, message: &str) {
        self.show(message, ToastType::Success);
    }

    pub fn error(&mut self, message: &str) {
        self.show(message, ToastType::Error);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.toasts.retain(|t| t.id != id);
    }

    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|t| {
            now.duration_since(t.created_at) < Duration::from_secs(TOAST_DURATION_SECS)
        });
    }

    pub fn toasts(&self) -> &VecDeque<Toast> {
        &self.toasts
    }
}

#[component]
pub fn ToastContainer(manager: Signal<ToastManager>) -> Element {
    use_future(move || {
        let mut manager = manager.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                manager.write().cleanup_expired();
            }
        }
    });

    rsx! {
        div {
            position: "fixed",
            bottom: "20px",
            left: "50%",
            transform: "translateX(-50%)",
            display: "flex",
            flex_direction: "column",
            gap: "8px",
            z_index: "9999",
            pointer_events: "none",

            for toast in manager.read().toasts().iter() {
                ToastItem {
                    key: "{toast.id}",
                    toast: toast.clone(),
                    manager: manager.clone(),
                }
            }
        }
    }
}

#[component]
fn ToastItem(toast: Toast, manager: Signal<ToastManager>) -> Element {
    let bg_color = match toast.toast_type {
        ToastType::Success => COLOR_SUCCESS,
        ToastType::Error => COLOR_ERROR,
    };

    let icon = match toast.toast_type {
        ToastType::Success => "✓",
        ToastType::Error => "✕",
    };

    rsx! {
        div {
            display: "flex",
            align_items: "center",
            gap: "8px",
            padding: "10px 16px",
            background: bg_color,
            border_radius: "6px",
            color: "#1a1a1a",
            font_size: "13px",
            box_shadow: "0 4px 12px rgba(0, 0, 0, 0.3)",
            pointer_events: "auto",
            cursor: "pointer",
            white_space: "nowrap",

            onclick: {
                let id = toast.id;
                move |_| {
                    manager.write().remove(id);
                }
            },

            span {
                font_weight: "bold",
                font_size: "14px",
                "{icon}"
            }

            span {
                "{toast.message}"
            }
        }
    }
}
