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

pub(super) fn edit_connection_action(
    config_storage: Signal<Option<ConfigStorage>>,
    mut form_mode: Signal<Option<super::state::FormMode>>,
) -> Callback<Uuid> {
    Callback::new(move |id: Uuid| {
        if let Some(storage) = config_storage.read().as_ref() {
            if let Ok(saved) = storage.load_connections() {
                if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                    form_mode.set(Some(super::state::FormMode::Edit(config)));
                }
            }
        }
    })
}

pub(super) fn delete_connection_prompt_action(
    mut connections: Signal<Vec<(Uuid, String)>>,
    mut show_delete_connection_dialog: Signal<Option<(Uuid, String)>>,
) -> Callback<Uuid> {
    Callback::new(move |id: Uuid| {
        if let Some((_, name)) = connections.read().iter().find(|(conn_id, _)| *conn_id == id) {
            show_delete_connection_dialog.set(Some((id, name.clone())));
        }
    })
}

pub(super) fn open_optional_uuid_signal(
    mut signal: Signal<Option<Uuid>>,
) -> Callback<Uuid> {
    Callback::new(move |id: Uuid| signal.set(Some(id)))
}

pub(super) fn open_bool_signal(mut signal: Signal<bool>) -> Callback<()> {
    Callback::new(move |_| signal.set(true))
}

pub(super) fn reorder_connections_action(
    mut connections: Signal<Vec<(Uuid, String)>>,
    config_storage: Signal<Option<ConfigStorage>>,
) -> Callback<(usize, usize)> {
    Callback::new(move |(from, to): (usize, usize)| {
        let mut conns = connections.write();
        if from < conns.len() && to < conns.len() {
            let conn = conns.remove(from);
            conns.insert(to, conn);
            drop(conns);

            spawn(async move {
                if let Some(storage) = config_storage.read().as_ref() {
                    let _ = storage.reorder_connections(from, to);
                }
            });
        }
    })
}
