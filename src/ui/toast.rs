use crate::theme::{COLOR_ERROR, COLOR_SUCCESS};
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use uuid::Uuid;

const TOAST_DURATION_SECS: u64 = 2;
const EXIT_ANIMATION_DURATION_MS: u64 = 200;
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
    exiting_ids: HashSet<Uuid>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: VecDeque::new(),
            exiting_ids: HashSet::new(),
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
            if let Some(old) = self.toasts.pop_front() {
                self.exiting_ids.remove(&old.id);
            }
        }
        self.toasts.push_back(toast);
    }

    pub fn success(&mut self, message: &str) {
        self.show(message, ToastType::Success);
    }

    pub fn error(&mut self, message: &str) {
        self.show(message, ToastType::Error);
    }

    pub fn start_exit(&mut self, id: Uuid) {
        self.exiting_ids.insert(id);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.toasts.retain(|t| t.id != id);
        self.exiting_ids.remove(&id);
    }

    pub fn is_exiting(&self, id: Uuid) -> bool {
        self.exiting_ids.contains(&id)
    }

    pub fn cleanup_expired(&mut self) -> Vec<Uuid> {
        let now = Instant::now();
        let mut to_exit = Vec::new();
        
        for toast in self.toasts.iter() {
            if now.duration_since(toast.created_at) >= Duration::from_secs(TOAST_DURATION_SECS)
                && !self.exiting_ids.contains(&toast.id)
            {
                to_exit.push(toast.id);
            }
        }
        
        for id in &to_exit {
            self.exiting_ids.insert(*id);
        }
        
        to_exit
    }

    pub fn toasts(&self) -> &VecDeque<Toast> {
        &self.toasts
    }
}

#[component]
pub fn ToastContainer(manager: Signal<ToastManager>) -> Element {
    let pending_removals: Signal<HashMap<Uuid, bool>> = use_signal(HashMap::new);

    use_future(move || {
        let mut manager = manager.clone();
        let mut pending_removals = pending_removals.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let to_exit = manager.write().cleanup_expired();
                
                for id in to_exit {
                    pending_removals.write().insert(id, true);
                    let mut manager_clone = manager.clone();
                    let mut pending_clone = pending_removals.clone();
                    spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                        manager_clone.write().remove(id);
                        pending_clone.write().remove(&id);
                    });
                }
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

            style {
                r#"
                @keyframes toastSlideIn {{
                    from {{ opacity: 0; transform: translateY(20px); }}
                    to {{ opacity: 1; transform: translateY(0); }}
                }}
                @keyframes toastSlideOut {{
                    from {{ opacity: 1; transform: translateY(0); }}
                    to {{ opacity: 0; transform: translateY(-20px); }}
                }}
                "#
            }

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

    let is_exiting = manager.read().is_exiting(toast.id);
    let animation = if is_exiting {
        "toastSlideOut 0.2s ease-out forwards"
    } else {
        "toastSlideIn 0.3s ease-out"
    };

    rsx! {
        div {
            display: "flex",
            align_items: "center",
            gap: "8px",
            padding: "10px 16px",
            background: bg_color,
            border_radius: "6px",
            color: "#ffffff",
            font_size: "13px",
            box_shadow: "0 4px 12px rgba(0, 0, 0, 0.3)",
            pointer_events: "auto",
            cursor: "pointer",
            white_space: "nowrap",
            animation: "{animation}",

            onclick: {
                let id = toast.id;
                move |_| {
                    manager.write().start_exit(id);
                    let mut manager_clone = manager.clone();
                    spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(EXIT_ANIMATION_DURATION_MS)).await;
                        manager_clone.write().remove(id);
                    });
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