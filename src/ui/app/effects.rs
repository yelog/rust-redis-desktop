use super::state::FormMode;
use crate::config::ConfigStorage;
use crate::i18n::I18n;
use crate::theme::ThemePreference;
use crate::ui::ToastManager;
use crate::updater::{
    set_checking, set_pending_update, should_trigger_manual_check, UpdateManager,
};
use dioxus::document;
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

pub(super) fn use_load_saved_connections(
    config_storage: Signal<Option<ConfigStorage>>,
    mut connections: Signal<Vec<(Uuid, String)>>,
    mut readonly_connections: Signal<HashMap<Uuid, bool>>,
) {
    use_effect(move || {
        if let Some(storage) = config_storage.read().as_ref() {
            if let Ok(saved) = storage.load_connections() {
                let conns: Vec<(Uuid, String)> =
                    saved.iter().map(|c| (c.id, c.name.clone())).collect();
                let readonly: HashMap<Uuid, bool> =
                    saved.iter().map(|c| (c.id, c.readonly)).collect();
                connections.set(conns);
                readonly_connections.set(readonly);
            }
        }
    });
}

pub(super) fn use_theme_bridge(
    theme_preference: Signal<ThemePreference>,
    bridge_script: fn(ThemePreference) -> String,
) {
    use_effect(move || {
        let script = bridge_script(theme_preference());
        let _ = document::eval(&script);
    });
}

pub(super) fn use_keyboard_shortcuts(
    mut show_settings: Signal<bool>,
    mut form_mode: Signal<Option<FormMode>>,
    mut show_flush_dialog: Signal<Option<Uuid>>,
    mut show_delete_connection_dialog: Signal<Option<(Uuid, String)>>,
) {
    use_future(move || async move {
        let mut eval = document::eval(
            r#"
document.addEventListener('keydown', (e) => {
    if (e.key === ',' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        dioxus.send('toggle_settings');
    }
    if (e.key === 'n' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        dioxus.send('new_connection');
    }
    if (e.key === 'f' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        dioxus.send('focus_search');
    }
    if (e.key === 'r' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        dioxus.send('refresh_keys');
    }
    if (e.key === 'Escape') {
        const dialog = document.querySelector('[data-dialog="true"]');
        if (dialog) {
            return;
        }
        dioxus.send('escape_pressed');
    }
});
await new Promise(() => {});
"#,
        );

        while let Ok(msg) = eval.recv::<String>().await {
            if msg == "toggle_settings" {
                show_settings.toggle();
            } else if msg == "new_connection" {
                form_mode.set(Some(FormMode::New));
            } else if msg == "escape_pressed" {
                if show_settings() {
                    show_settings.set(false);
                } else if form_mode().is_some() {
                    form_mode.set(None);
                } else if show_flush_dialog().is_some() {
                    show_flush_dialog.set(None);
                } else if show_delete_connection_dialog().is_some() {
                    show_delete_connection_dialog.set(None);
                }
            }
        }
    });
}

pub(super) fn use_manual_update_check(
    i18n: Signal<I18n>,
    mut toast_for_update: Signal<ToastManager>,
) {
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if should_trigger_manual_check() {
                set_checking(true);
                if let Ok(mut manager) = UpdateManager::new() {
                    match manager.check_for_updates().await {
                        Ok(Some(info)) => {
                            tracing::info!("Manual check found new version: {}", info.version);
                            set_pending_update(Some(info));
                        }
                        Ok(None) => {
                            set_pending_update(None);
                            toast_for_update
                                .write()
                                .success(&i18n.read().t("Already up to date"));
                        }
                        Err(e) => {
                            set_pending_update(None);
                            let msg =
                                format!("{}{}", i18n.read().t("Failed to check for updates: "), e);
                            toast_for_update.write().error(&msg);
                        }
                    }
                } else {
                    set_pending_update(None);
                    toast_for_update
                        .write()
                        .error(&i18n.read().t("Unable to initialize update checker"));
                }
                set_checking(false);
            }
        }
    });
}

pub(super) fn use_system_theme_listener(mut system_theme_dark: Signal<bool>) {
    use_future(move || async move {
        let mut eval = document::eval(
            r#"
const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
const notify = () => dioxus.send(mediaQuery.matches);

notify();

if (typeof mediaQuery.addEventListener === "function") {
  mediaQuery.addEventListener("change", notify);
} else if (typeof mediaQuery.addListener === "function") {
  mediaQuery.addListener(notify);
}

await new Promise(() => {});
"#,
        );

        while let Ok(is_dark) = eval.recv::<bool>().await {
            system_theme_dark.set(is_dark);
        }
    });
}
