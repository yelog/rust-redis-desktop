use crate::config::{AppSettings, ConfigStorage};
use crate::theme::ThemePreference;
use crate::ui::ToastManager;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub(super) fn save_settings_action(
    config_storage: Signal<Option<ConfigStorage>>,
    mut app_settings: Signal<AppSettings>,
    mut theme_preference: Signal<ThemePreference>,
) -> Callback<AppSettings> {
    Callback::new(move |settings: AppSettings| {
        app_settings.set(settings.clone());
        theme_preference.set(settings.theme_preference);
        if let Some(storage) = config_storage.read().as_ref() {
            let _ = storage.save_settings(&settings);
        }
    })
}

pub(super) fn import_connections_action(
    mut show_import_connections_dialog: Signal<bool>,
    mut connections: Signal<Vec<(Uuid, String)>>,
    mut readonly_connections: Signal<HashMap<Uuid, bool>>,
    mut toast_manager: Signal<ToastManager>,
    config_storage: Arc<ConfigStorage>,
) -> Callback<usize> {
    Callback::new(move |_count: usize| {
        show_import_connections_dialog.set(false);
        if let Ok(saved) = config_storage.load_connections() {
            let conns: Vec<(Uuid, String)> = saved.iter().map(|c| (c.id, c.name.clone())).collect();
            let readonly: HashMap<Uuid, bool> = saved.iter().map(|c| (c.id, c.readonly)).collect();
            connections.set(conns);
            readonly_connections.set(readonly);
        }
        toast_manager.write().success("Connections imported");
    })
}
