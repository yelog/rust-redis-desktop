mod actions;
mod effects;
mod render;
mod state;
mod theme;

use self::actions::{
    confirm_delete_connection_action, delete_connection_prompt_action, edit_connection_action,
    import_connections_action, open_bool_signal, open_optional_uuid_signal,
    reconnect_connection_action, reorder_connections_action, save_connection_action,
    save_settings_action, select_connection_action,
};
use self::effects::{
    use_keyboard_shortcuts, use_load_saved_connections, use_manual_update_check,
    use_system_theme_listener, use_theme_bridge,
};
use self::render::{
    empty_connection_panel, spinner_panel, ConnectedTabShellSection,
    ExportConnectionsDialogSection, FlushDialogSection, ImportConnectionsDialogSection,
    ImportOverlaySection, MacTitlebarSection, SettingsDialogSection,
};
use self::theme::{build_theme_palette, load_initial_settings, system_theme_is_dark};
use crate::config::{AppSettings, ConfigStorage};
use crate::connection::{ConnectionConfig, ConnectionManager, ConnectionPool, ConnectionState};
use crate::theme::{
    preferred_window_theme, resolve_theme, theme_spec, ThemePreference, ThemeSpec, COLOR_ACCENT,
    COLOR_BG, COLOR_BG_SECONDARY, COLOR_BORDER, COLOR_ERROR, COLOR_PRIMARY, COLOR_SURFACE_LOW,
    COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY,
};
use crate::ui::value_viewer::ImagePreview;
use crate::ui::{
    ClientsPanel, ConnectionExportDialog, ConnectionForm, ConnectionImportDialog,
    DeleteConnectionConfirmDialog, FlushConfirmDialog, ImportPanel, KeyBrowser, LeftRail,
    MonitorPanel, PubSubPanel, ResizableDivider, ScriptPanel, SettingsDialog, SlowLogPanel,
    Terminal, ToastContainer, ToastManager, UpdateDialog,
};
use crate::updater::{
    set_checking, set_pending_update, should_trigger_manual_check, InstallResult, UpdateInfo,
    UpdateManager, UPDATE_STATUS,
};
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub use self::state::{FormMode, Tab};

const MACOS_TITLEBAR_HEIGHT: &str = "46px";
const MACOS_TRAFFIC_LIGHT_PADDING_LEFT: &str = "84px";

fn build_theme_bridge_script(preference: ThemePreference) -> String {
    let selected_preference = serde_json::to_string(&preference).unwrap();
    let classic_dark = build_theme_palette(theme_spec(crate::theme::ThemeId::ClassicDark));
    let classic_light = build_theme_palette(theme_spec(crate::theme::ThemeId::ClassicLight));
    let tokyo_night = build_theme_palette(theme_spec(crate::theme::ThemeId::TokyoNight));
    let tokyo_night_light = build_theme_palette(theme_spec(crate::theme::ThemeId::TokyoNightLight));
    let atom_one_light = build_theme_palette(theme_spec(crate::theme::ThemeId::AtomOneLight));
    let github_light = build_theme_palette(theme_spec(crate::theme::ThemeId::GitHubLight));
    let one_dark_pro = build_theme_palette(theme_spec(crate::theme::ThemeId::OneDarkPro));
    let dracula = build_theme_palette(theme_spec(crate::theme::ThemeId::Dracula));

    format!(
        r##"
(() => {{
  const bridge = (window.__rrdThemeBridge = window.__rrdThemeBridge || {{}});
  bridge.selectedPreference = {selected_preference};
  bridge.themes = {{
    classic_dark: {classic_dark},
    classic_light: {classic_light},
    tokyo_night: {tokyo_night},
    tokyo_night_light: {tokyo_night_light},
    atom_one_light: {atom_one_light},
    github_light: {github_light},
    one_dark_pro: {one_dark_pro},
    dracula: {dracula},
  }};

  const normalize = (value) => (value || "")
    .toString()
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "");

  const hexToRgb = (value) => {{
    if (!value || !value.startsWith("#")) {{
      return null;
    }}

    let hex = value.slice(1);
    if (hex.length === 3) {{
      hex = hex
        .split("")
        .map((part) => part + part)
        .join("");
    }}

    if (hex.length !== 6) {{
      return null;
    }}

    const num = Number.parseInt(hex, 16);
    if (Number.isNaN(num)) {{
      return null;
    }}

    const red = (num >> 16) & 255;
    const green = (num >> 8) & 255;
    const blue = num & 255;
    return `rgb(${{red}},${{green}},${{blue}})`;
  }};

  const addAlias = (map, aliases, target) => {{
    aliases.forEach((alias) => {{
      if (!alias) {{
        return;
      }}

      const normalized = normalize(alias);
      if (!normalized) {{
        return;
      }}

      map.set(normalized, target);

      const rgb = hexToRgb(alias);
      if (rgb) {{
        map.set(normalize(rgb), target);
      }}
    }});
  }};

  const themedSurfaceAliases = [
    [["#1e1e1e", "#ffffff"], "surfaceBase"],
    [["#252526", "#f3f3f3"], "surfaceSecondary"],
    [["#2d2d2d", "#e8e8e8"], "surfaceTertiary"],
    [["#3c3c3c", "#d4d4d4"], "border"],
    [["#555", "#555555", "#c7c7c7"], "controlBorder"],
    [["#333", "#5a5a5a", "#d9d9d9"], "buttonSecondary"],
    [["#444"], "buttonSecondaryBorder"],
    [["rgba(0, 0, 0, 0.7)"], "overlayBackdrop"],
    [["#1a1a2e"], "infoBg"],
    [["#1f2937", "#202a33", "rgba(0, 122, 204, 0.12)", "rgba(49, 130, 206, 0.18)"], "infoBgAlt"],
    [["#1a3a1a", "rgba(16, 124, 16, 0.12)", "rgba(48, 209, 88, 0.08)", "rgba(48, 209, 88, 0.10)", "rgba(78, 201, 176, 0.1)"], "successBg"],
    [["#1a4a1a", "#1e4620", "rgba(47, 133, 90, 0.16)", "rgba(48, 209, 88, 0.15)", "rgba(48, 209, 88, 0.20)"], "successBgAlt"],
    [["rgba(255, 159, 10, 0.15)", "rgba(245, 158, 11, 0.1)"], "warningBg"],
    [["#2d1f1f", "rgba(209, 52, 56, 0.12)", "rgba(248, 113, 113, 0.1)", "rgba(197, 48, 48, 0.18)"], "errorBg"],
    [["#094771", "rgba(0, 218, 243, 0.06)", "rgba(0, 127, 142, 0.12)"], "selectionBg"],
    [["rgba(177, 44, 25, 0.08)"], "selectionBgAlt"],
    [["rgba(15, 108, 189, 0.08)"], "rowCreateBg"],
    [["rgba(15, 108, 189, 0.12)"], "rowEditBg"],
    [["rgba(255, 180, 166, 0.10)", "rgba(255, 180, 166, 0.12)"], "toneStringBg"],
    [["rgba(255, 180, 166, 0.20)", "rgba(255, 180, 166, 0.24)"], "toneStringBorder"],
    [["rgba(0, 218, 243, 0.10)"], "toneHashBg"],
    [["rgba(0, 218, 243, 0.22)"], "toneHashBorder"],
    [["rgba(229, 226, 225, 0.08)", "rgba(169, 138, 132, 0.10)"], "toneListBg"],
    [["rgba(229, 226, 225, 0.18)", "rgba(169, 138, 132, 0.18)"], "toneListBorder"],
    [["rgba(48, 209, 88, 0.10)"], "toneStreamBg"],
    [["rgba(48, 209, 88, 0.20)"], "toneStreamBorder"],
  ];

  const themedAccentAliases = [
    [["#0e639c", "#3182ce"], "primary"],
    [["#007acc", "#4ec9b0"], "accent"],
    [["#38a169", "#68d391", "#22c55e", "#4ade80"], "success"],
    [["#f59e0b"], "warning"],
    [["#c53030", "#f87171", "#ef4444", "#f44336"], "error"],
    [["#63b3ed"], "info"],
    [["#805ad5", "#a78bfa"], "secondaryAction"],
    [["#9cdcfe"], "syntaxKey"],
    [["#ce9178"], "syntaxString"],
    [["#b5cea8"], "syntaxNumber"],
    [["#569cd6"], "syntaxBoolean"],
    [["#dcdcaa"], "syntaxType"],
    [["#6b7280"], "syntaxComment"],
    [["#808080"], "syntaxNull"],
  ];

  const legacyTextPrimary = new Set([
    "white",
    "#fff",
    "#ffffff",
    "black",
    "#000",
    "#000000",
    "#1e1e1e",
    "#d4d4d4",
    "#ccc",
    "#cccccc",
    "#bbb",
    "#333",
  ].map(normalize));

  const legacyTextSecondary = new Set(["#888", "#666", "#808080"].map(normalize));

  const resolveTheme = () => {{
    const pref = bridge.selectedPreference;
    if (pref.system) {{
      const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      const themeId = isDark ? pref.system.dark : pref.system.light;
      return bridge.themes[themeId] || bridge.themes.classic_dark;
    }}
    if (pref.dark) {{
      return bridge.themes[pref.dark] || bridge.themes.classic_dark;
    }}
    if (pref.light) {{
      return bridge.themes[pref.light] || bridge.themes.classic_light;
    }}
    return bridge.themes.classic_dark;
  }};

  const buildMaps = (theme) => {{
    const exact = new Map();
    themedSurfaceAliases.forEach(([aliases, key]) => addAlias(exact, aliases, theme[key]));
    themedAccentAliases.forEach(([aliases, key]) => addAlias(exact, aliases, theme[key]));
    addAlias(exact, [theme.surfaceBase, theme.surfaceSecondary, theme.surfaceTertiary], theme.surfaceBase);
    addAlias(exact, [theme.controlBg], theme.controlBg);
    addAlias(exact, [theme.controlBorder], theme.controlBorder);
    addAlias(exact, [theme.textPrimary], theme.textPrimary);
    addAlias(exact, [theme.textSecondary], theme.textSecondary);
    addAlias(exact, [theme.textSubtle], theme.textSubtle);
    addAlias(exact, [theme.textSoft], theme.textSoft);
    return exact;
  }};

  const isContrastBackground = (value) => {{
    const normalized = normalize(value);
    return [
      bridge.themes.classic_dark.primary,
      bridge.themes.classic_light.primary,
      bridge.themes.classic_dark.success,
      bridge.themes.classic_light.success,
      bridge.themes.classic_dark.error,
      bridge.themes.classic_light.error,
      bridge.themes.classic_dark.warning,
      bridge.themes.classic_light.warning,
      bridge.themes.classic_dark.buttonSecondary,
      bridge.themes.classic_light.buttonSecondary,
      bridge.themes.classic_dark.secondaryAction,
      bridge.themes.classic_light.secondaryAction,
      bridge.themes.tokyo_night.primary,
      bridge.themes.tokyo_night.success,
      bridge.themes.tokyo_night.error,
      bridge.themes.tokyo_night.warning,
      bridge.themes.tokyo_night.buttonSecondary,
      bridge.themes.tokyo_night.secondaryAction,
      bridge.themes.tokyo_night_light.primary,
      bridge.themes.tokyo_night_light.success,
      bridge.themes.tokyo_night_light.error,
      bridge.themes.tokyo_night_light.warning,
      bridge.themes.tokyo_night_light.buttonSecondary,
      bridge.themes.tokyo_night_light.secondaryAction,
      bridge.themes.atom_one_light.primary,
      bridge.themes.atom_one_light.success,
      bridge.themes.atom_one_light.error,
      bridge.themes.atom_one_light.warning,
      bridge.themes.atom_one_light.buttonSecondary,
      bridge.themes.atom_one_light.secondaryAction,
      bridge.themes.github_light.primary,
      bridge.themes.github_light.success,
      bridge.themes.github_light.error,
      bridge.themes.github_light.warning,
      bridge.themes.github_light.buttonSecondary,
      bridge.themes.github_light.secondaryAction,
      bridge.themes.one_dark_pro.primary,
      bridge.themes.one_dark_pro.success,
      bridge.themes.one_dark_pro.error,
      bridge.themes.one_dark_pro.warning,
      bridge.themes.one_dark_pro.buttonSecondary,
      bridge.themes.one_dark_pro.secondaryAction,
      bridge.themes.dracula.primary,
      bridge.themes.dracula.success,
      bridge.themes.dracula.error,
      bridge.themes.dracula.warning,
      bridge.themes.dracula.buttonSecondary,
      bridge.themes.dracula.secondaryAction,
    ]
      .map(normalize)
      .includes(normalized);
  }};

  const replaceFragments = (value, map) => {{
    if (!value) {{
      return value;
    }}

    let nextValue = value;
    map.forEach((target, source) => {{
      if (nextValue.toLowerCase().includes(source)) {{
        nextValue = nextValue.replaceAll(source, target);
      }}
    }});
    return nextValue;
  }};

  const updateInlineStyles = (element, theme, map) => {{
    const style = element.style;
    if (!style) {{
      return;
    }}

    const backgroundValue = style.backgroundColor || style.background;
    const normalizedBackground = normalize(backgroundValue);

    if (backgroundValue) {{
      const mappedBackground = map.get(normalizedBackground);
      if (mappedBackground && backgroundValue !== mappedBackground) {{
        style.background = mappedBackground;
      }} else {{
        const fragmentBackground = replaceFragments(backgroundValue, map);
        if (fragmentBackground !== backgroundValue) {{
          style.background = fragmentBackground;
        }}
      }}
    }}

    if (style.borderColor) {{
      const mappedBorder = map.get(normalize(style.borderColor));
      if (mappedBorder && mappedBorder !== style.borderColor) {{
        style.borderColor = mappedBorder;
      }}
    }}

    if (style.borderBottomColor) {{
      const mappedBorderBottomColor = map.get(normalize(style.borderBottomColor));
      if (mappedBorderBottomColor && mappedBorderBottomColor !== style.borderBottomColor) {{
        style.borderBottomColor = mappedBorderBottomColor;
      }}
    }}

    if (style.borderTopColor) {{
      const mappedBorderTopColor = map.get(normalize(style.borderTopColor));
      if (mappedBorderTopColor && mappedBorderTopColor !== style.borderTopColor) {{
        style.borderTopColor = mappedBorderTopColor;
      }}
    }}

    if (style.border) {{
      const mappedBorder = replaceFragments(style.border, map);
      if (mappedBorder !== style.border) {{
        style.border = mappedBorder;
      }}
    }}

    if (style.borderBottom) {{
      const mappedBorderBottom = replaceFragments(style.borderBottom, map);
      if (mappedBorderBottom !== style.borderBottom) {{
        style.borderBottom = mappedBorderBottom;
      }}
    }}

    if (style.borderTop) {{
      const mappedBorderTop = replaceFragments(style.borderTop, map);
      if (mappedBorderTop !== style.borderTop) {{
        style.borderTop = mappedBorderTop;
      }}
    }}

    if (style.color) {{
      const normalizedColor = normalize(style.color);
      let nextColor = map.get(normalizedColor) || null;

      if (!nextColor && legacyTextPrimary.has(normalizedColor)) {{
        nextColor = isContrastBackground(backgroundValue) ? theme.textContrast : theme.textPrimary;
      }}

      if (!nextColor && legacyTextSecondary.has(normalizedColor)) {{
        nextColor = normalizedColor === normalize("#666") ? theme.textSubtle : theme.textSecondary;
      }}

      if (nextColor && nextColor !== style.color) {{
        style.color = nextColor;
      }}
    }}
  }};

  bridge.apply = () => {{
    const theme = resolveTheme();
    const map = buildMaps(theme);
    const root = document.documentElement;
    const body = document.body;

    if (bridge.observer) {{
      bridge.observer.disconnect();
    }}

    const getThemeMode = () => {{
    const pref = bridge.selectedPreference;
    if (pref.system) return "system";
    if (pref.dark) return "dark";
    if (pref.light) return "light";
    return "system";
  }};

    root.dataset.themeMode = getThemeMode();
    root.dataset.themeResolved = theme.isDark ? "dark" : "light";
    root.style.colorScheme = theme.isDark ? "dark" : "light";
    root.style.setProperty("--theme-bg", theme.surfaceBase);
    root.style.setProperty("--theme-bg-secondary", theme.surfaceSecondary);
    root.style.setProperty("--theme-bg-tertiary", theme.surfaceTertiary);
    root.style.setProperty("--theme-bg-lowest", theme.surfaceLowest);
    root.style.setProperty("--theme-overlay-backdrop", theme.overlayBackdrop);
    root.style.setProperty("--theme-control-bg", theme.controlBg);
    root.style.setProperty("--theme-control-border", theme.controlBorder);
    root.style.setProperty("--theme-button-secondary", theme.buttonSecondary);
    root.style.setProperty("--theme-button-secondary-border", theme.buttonSecondaryBorder);
    root.style.setProperty("--theme-surface-low", theme.surfaceLow);
    root.style.setProperty("--theme-surface-high", theme.surfaceHigh);
    root.style.setProperty("--theme-surface-highest", theme.surfaceHighest);
    root.style.setProperty("--theme-border", theme.border);
    root.style.setProperty("--theme-text", theme.textPrimary);
    root.style.setProperty("--theme-text-secondary", theme.textSecondary);
    root.style.setProperty("--theme-text-subtle", theme.textSubtle);
    root.style.setProperty("--theme-text-soft", theme.textSoft);
    root.style.setProperty("--theme-text-contrast", theme.textContrast);
    root.style.setProperty("--theme-primary", theme.primary);
    root.style.setProperty("--theme-accent", theme.accent);
    root.style.setProperty("--theme-success", theme.success);
    root.style.setProperty("--theme-warning", theme.warning);
    root.style.setProperty("--theme-error", theme.error);
    root.style.setProperty("--theme-info", theme.info);
    root.style.setProperty("--theme-info-bg", theme.infoBg);
    root.style.setProperty("--theme-info-bg-alt", theme.infoBgAlt);
    root.style.setProperty("--theme-success-bg", theme.successBg);
    root.style.setProperty("--theme-success-bg-alt", theme.successBgAlt);
    root.style.setProperty("--theme-warning-bg", theme.warningBg);
    root.style.setProperty("--theme-error-bg", theme.errorBg);
    root.style.setProperty("--theme-selection-bg", theme.selectionBg);
    root.style.setProperty("--theme-selection-bg-alt", theme.selectionBgAlt);
    root.style.setProperty("--theme-row-create-bg", theme.rowCreateBg);
    root.style.setProperty("--theme-row-edit-bg", theme.rowEditBg);
    root.style.setProperty("--theme-outline", theme.outline);
    root.style.setProperty("--theme-outline-variant", theme.outlineVariant);
    root.style.setProperty("--theme-tone-string-bg", theme.toneStringBg);
    root.style.setProperty("--theme-tone-string-border", theme.toneStringBorder);
    root.style.setProperty("--theme-tone-hash-bg", theme.toneHashBg);
    root.style.setProperty("--theme-tone-hash-border", theme.toneHashBorder);
    root.style.setProperty("--theme-tone-list-bg", theme.toneListBg);
    root.style.setProperty("--theme-tone-list-border", theme.toneListBorder);
    root.style.setProperty("--theme-tone-set-bg", theme.toneSetBg);
    root.style.setProperty("--theme-tone-set-border", theme.toneSetBorder);
    root.style.setProperty("--theme-tone-zset-bg", theme.toneZsetBg);
    root.style.setProperty("--theme-tone-zset-border", theme.toneZsetBorder);
    root.style.setProperty("--theme-tone-stream-bg", theme.toneStreamBg);
    root.style.setProperty("--theme-tone-stream-border", theme.toneStreamBorder);
    root.style.setProperty("--theme-syntax-key", theme.syntaxKey);
    root.style.setProperty("--theme-syntax-string", theme.syntaxString);
    root.style.setProperty("--theme-syntax-number", theme.syntaxNumber);
    root.style.setProperty("--theme-syntax-boolean", theme.syntaxBoolean);
    root.style.setProperty("--theme-syntax-null", theme.syntaxNull);
    root.style.setProperty("--theme-syntax-bracket", theme.syntaxBracket);
    root.style.setProperty("--theme-syntax-keyword", theme.syntaxKeyword);
    root.style.setProperty("--theme-syntax-type", theme.syntaxType);
    root.style.setProperty("--theme-syntax-function", theme.syntaxFunction);
    root.style.setProperty("--theme-syntax-comment", theme.syntaxComment);
    root.style.setProperty("--theme-syntax-operator", theme.syntaxOperator);
    root.style.setProperty("--theme-syntax-constant", theme.syntaxConstant);
    body.style.margin = "0";
    body.style.padding = "0";
    body.style.backgroundColor = theme.surfaceBase;
    body.style.color = theme.textPrimary;

    [body, ...body.querySelectorAll("*")].forEach((element) => {{
      if (!(element instanceof HTMLElement)) {{
        return;
      }}
      updateInlineStyles(element, theme, map);
    }});

    if (bridge.observer) {{
      bridge.observer.observe(document.body, bridge.observerConfig);
    }}
  }};

  bridge.schedule = () => {{
    if (bridge.raf) {{
      cancelAnimationFrame(bridge.raf);
    }}
    bridge.raf = requestAnimationFrame(() => bridge.apply());
  }};

  if (!bridge.mediaQuery) {{
    bridge.mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    bridge.mediaQuery.addEventListener("change", () => bridge.schedule());
  }}

  if (!bridge.observer) {{
    bridge.observerConfig = {{
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["style"],
    }};
    bridge.observer = new MutationObserver(() => bridge.schedule());
    bridge.observer.observe(document.body, bridge.observerConfig);
  }}

  bridge.schedule();
}})();
"##,
        selected_preference = selected_preference,
        classic_dark = classic_dark,
        classic_light = classic_light,
        tokyo_night = tokyo_night,
        tokyo_night_light = tokyo_night_light,
        atom_one_light = atom_one_light,
        github_light = github_light,
        one_dark_pro = one_dark_pro,
        dracula = dracula,
    )
}

#[component]
pub fn App() -> Element {
    let mut connections = use_signal(Vec::new);
    let mut form_mode = use_signal(|| None::<FormMode>);
    let mut selected_connection = use_signal(|| None::<Uuid>);
    let connection_manager = use_signal(ConnectionManager::new);
    let config_storage = use_signal(|| ConfigStorage::new().ok());
    let mut selected_key = use_signal(String::new);
    let mut connection_pools = use_signal(HashMap::<Uuid, ConnectionPool>::new);
    let mut refresh_trigger = use_signal(|| 0u32);
    let mut current_tab = use_signal(|| Tab::Data);
    let mut reconnecting_ids = use_signal(HashSet::<Uuid>::new);
    let mut connection_versions = use_signal(HashMap::<Uuid, u32>::new);
    let mut connection_states = use_signal(HashMap::<Uuid, ConnectionState>::new);
    let mut readonly_connections = use_signal(HashMap::<Uuid, bool>::new);
    let mut app_settings = use_signal(load_initial_settings);
    let mut show_settings = use_signal(|| false);
    let mut show_flush_dialog = use_signal(|| None::<Uuid>);
    let mut show_import_dialog = use_signal(|| None::<Uuid>);
    let mut show_delete_connection_dialog = use_signal(|| None::<(Uuid, String)>);
    let mut show_export_connections_dialog = use_signal(|| false);
    let mut show_import_connections_dialog = use_signal(|| false);
    let mut current_db = use_signal(|| 0u8);
    let theme_preference = use_signal(|| load_initial_settings().theme_preference);
    let mut system_theme_dark = use_signal(system_theme_is_dark);
    let left_rail_width = use_signal(|| 280.0);
    let toast_manager = use_context_provider(|| Signal::new(ToastManager::new()));
    let desktop = use_window();
    let desktop_for_theme = desktop.clone();
    let desktop_for_drag = desktop.clone();
    let desktop_for_maximize = desktop.clone();

    let mut update_download_progress = use_signal(|| (0u64, 0u64));
    let mut update_downloaded_path = use_signal(|| None::<PathBuf>);
    let toast_for_update = toast_manager.clone();

    let active_theme_preference = theme_preference();
    let active_system_theme_dark = system_theme_dark();
    let active_theme = resolve_theme(active_theme_preference, active_system_theme_dark);
    let colors = active_theme.colors;
    let resolved_theme_id = active_theme.id;
    let resolved_theme_key = resolved_theme_id.as_str();

    use_load_saved_connections(config_storage, connections, readonly_connections);

    use_theme_bridge(theme_preference, build_theme_bridge_script);

    use_effect(move || {
        desktop_for_theme.set_theme(preferred_window_theme(theme_preference()));
    });

    use_keyboard_shortcuts(
        show_settings,
        form_mode,
        show_flush_dialog,
        show_delete_connection_dialog,
    );

    #[cfg(target_os = "macos")]
    {
        use dioxus::desktop::use_muda_event_handler;
        let mut show_settings_for_menu = show_settings.clone();
        use_muda_event_handler(move |event| {
            if event.id == "preferences" {
                show_settings_for_menu.toggle();
            } else if event.id == "check_updates" {
                crate::updater::trigger_manual_check();
            }
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        use dioxus::desktop::use_muda_event_handler;
        use_muda_event_handler(move |event| {
            if event.id == "check_updates" {
                crate::updater::trigger_manual_check();
            }
        });
    }

    use_manual_update_check(toast_for_update);

    use_system_theme_listener(system_theme_dark);

    let save_settings = save_settings_action(config_storage, app_settings, theme_preference);

    let selected_conn_state = selected_connection()
        .and_then(|id| connection_states.read().get(&id).copied())
        .unwrap_or(ConnectionState::Disconnected);
    let titlebar_context_label = selected_connection().and_then(|id| {
        connections
            .read()
            .iter()
            .find(|(conn_id, _)| *conn_id == id)
            .map(|(_, name)| name.clone())
    });

    rsx! {
                style { {r#"
                * {
                    transition: background-color 300ms ease-in-out,
                                border-color 300ms ease-in-out,
                                color 300ms ease-in-out,
                                box-shadow 300ms ease-in-out;
                }
            "#} }

                div {
                    display: "flex",
                    flex_direction: "column",
                    height: "100vh",
                    background: COLOR_BG,
                    color: COLOR_TEXT,
                    overflow: "hidden",

                    if cfg!(target_os = "macos") {
                        MacTitlebarSection {
                            context_label: titlebar_context_label.clone(),
                            on_drag: move |_| desktop_for_drag.drag(),
                            on_toggle_maximize: move |_| desktop_for_maximize.toggle_maximized(),
                        }
                    }

                    div {
                        flex: "1",
                        min_height: "0",
                        display: "flex",
                        overflow: "hidden",

                        LeftRail {
                                width: left_rail_width,
                                connections: connections(),
                                connection_states: connection_states(),
                                readonly_connections: readonly_connections(),
                                selected_connection: selected_connection(),
                                colors: colors.clone(),
                                on_add_connection: move |_| form_mode.set(Some(FormMode::New)),
                                on_select_connection: select_connection_action(
                                    selected_connection,
                                    selected_key,
                                    current_tab,
                                    current_db,
                                    connection_states,
                                    connection_versions,
                                    connection_pools,
                                    connection_manager,
                                    config_storage,
                                ),
                                on_reconnect_connection: reconnect_connection_action(
                                    reconnecting_ids,
                                    connection_states,
                                    config_storage,
                                    connection_pools,
                                    connection_manager,
                                    connection_versions,
                                    selected_connection,
                                    current_db,
                                ),
                            on_close_connection: move |id: Uuid| {
                                spawn(async move {
                                    connection_pools.write().remove(&id);
                                    connection_manager.read().remove_connection(id).await;
                                    connection_states.write().insert(id, ConnectionState::Disconnected);

                                    if selected_connection() == Some(id) {
                                        selected_connection.set(None);
                                        selected_key.set(String::new());
                                        current_db.set(0);
                                    }
                                });
                            },
                            on_edit_connection: edit_connection_action(config_storage, form_mode),
                            on_delete_connection: delete_connection_prompt_action(
                                connections,
                                show_delete_connection_dialog,
                            ),
                            on_flush_connection: open_optional_uuid_signal(show_flush_dialog),
                            on_import_connection: open_optional_uuid_signal(show_import_dialog),
                            on_export_connections: open_bool_signal(show_export_connections_dialog),
                            on_import_connections: open_bool_signal(show_import_connections_dialog),
                            on_open_settings: open_bool_signal(show_settings),
                            on_reorder_connection: reorder_connections_action(
                                connections,
                                config_storage,
                            ),
                        }

                        ResizableDivider {
                            size: left_rail_width,
                            min_size: 180.0,
                            max_size: 400.0,
                        }

        if let Some(conn_id) = selected_connection() {
                            if reconnecting_ids.read().contains(&conn_id) {
                                div {
                                    flex: "1",
                                    display: "flex",
                                    flex_direction: "column",
                                    align_items: "center",
                                    justify_content: "center",
                                    gap: "16px",
                                    background: "{COLOR_SURFACE_LOW}",

                                    style { {r#"
                                @keyframes spin {
                                    from { transform: rotate(0deg); }
                                    to { transform: rotate(360deg); }
                                }
                            "#} }

                                    div {
                                        width: "40px",
                                        height: "40px",
                                        border: "3px solid {COLOR_ACCENT}",
                                        border_top_color: "transparent",
                                        border_radius: "50%",
                                        animation: "spin 0.8s linear infinite",
                                    }

                                    div {
                                        color: "{COLOR_TEXT_SECONDARY}",
                                        font_size: "14px",

                                        "正在重新连接..."
                                    }
                                }
                            } else if selected_conn_state == ConnectionState::Error {
                                div {
                                    flex: "1",
                                    display: "flex",
                                    flex_direction: "column",
                                    align_items: "center",
                                    justify_content: "center",
                                    gap: "16px",
                                    background: "{COLOR_SURFACE_LOW}",

                                    div {
                                        color: "{COLOR_ERROR}",
                                        font_size: "14px",

                                        "连接失败，请检查连接配置后重试"
                                    }

                                    button {
                                        padding: "10px 20px",
                                        background: "{COLOR_PRIMARY}",
                                        color: "{COLOR_TEXT_CONTRAST}",
                                        border: "none",
                                        border_radius: "6px",
                                        cursor: "pointer",
                                        font_size: "13px",

                                        onclick: move |_| {
                                            spawn(async move {
                                                reconnecting_ids.write().insert(conn_id);
                                                connection_states.write().insert(conn_id, ConnectionState::Connecting);

                                                if let Some(storage) = config_storage.read().as_ref() {
                                                    if let Ok(saved) = storage.load_connections() {
                                                        if let Some(config) = saved.into_iter().find(|c| c.id == conn_id) {
                                                            match ConnectionPool::new(config.clone()).await {
                                                                Ok(pool) => {
                                                                    connection_pools.write().insert(conn_id, pool);
                                                                    let _ = connection_manager.read().add_connection(config).await;
                                                                    let version = connection_versions.read().get(&conn_id).copied().unwrap_or(0);
                                                                    connection_versions.write().insert(conn_id, version + 1);
                                                                    connection_states.write().insert(conn_id, ConnectionState::Connected);
                                                                }
                                                                Err(_) => {
                                                                    connection_states.write().insert(conn_id, ConnectionState::Error);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                reconnecting_ids.write().remove(&conn_id);
                                            });
                                        },

                                        "重新连接"
                                    }
                                }
                            } else if selected_conn_state == ConnectionState::Connecting {
                                { spinner_panel("正在加载连接...") }
                            } else if selected_conn_state == ConnectionState::Connected {
                                if let Some(pool) = connection_pools.read().get(&conn_id).cloned() {
                                    ConnectedTabShellSection {
                                        conn_id,
                                        pool,
                                        current_tab,
                                        connection_version: connection_versions.read().get(&conn_id).copied().unwrap_or(0),
                                        selected_key,
                                        current_db,
                                        refresh_trigger,
                                        colors,
                                        resolved_theme_key: resolved_theme_key.to_string(),
                                        auto_refresh_interval: app_settings.read().auto_refresh_interval,
                                    }
                            } else {
                                { spinner_panel("正在初始化连接...") }
                            }
                            } else {
                                { spinner_panel("正在连接...") }
                            }
                        } else {
                            { empty_connection_panel() }
                        }
                    }
                }

                if let Some(mode) = form_mode() {
                    ConnectionForm {
                        editing_config: match mode {
                            FormMode::Edit(config) => Some(config),
                            FormMode::New => None,
                        },
                        colors,
                        on_save: save_connection_action(
                            config_storage,
                            connections,
                            readonly_connections,
                            connection_manager,
                            form_mode,
                        ),
                        on_cancel: move |_| form_mode.set(None),
                    }
                }

                if show_settings() {
                    SettingsDialogSection {
                        settings: app_settings.read().clone(),
                        colors,
                        resolved_theme_id,
                        on_change: {
                            let save_settings = save_settings.clone();
                            move |settings: AppSettings| {
                                save_settings.call(settings);
                            }
                        },
                        on_close: move |_| show_settings.set(false),
                    }
                }

    if let Some((delete_id, delete_name)) = show_delete_connection_dialog() {
                    DeleteConnectionConfirmDialog {
                        connection_id: delete_id,
                        connection_name: delete_name,
                        colors,
                        on_confirm: confirm_delete_connection_action(
                            config_storage,
                            show_delete_connection_dialog,
                            connection_pools,
                            connection_manager,
                            connection_states,
                            connections,
                            selected_connection,
                            selected_key,
                            current_db,
                        ),
                        on_cancel: move |_| show_delete_connection_dialog.set(None),
                    }
                }

    if let Some(flush_id) = show_flush_dialog() {
                    if let Some(pool) = connection_pools.read().get(&flush_id).cloned() {
                        FlushDialogSection {
                            pool,
                            current_db: current_db(),
                            colors,
                            on_confirm: move |_| {
                                show_flush_dialog.set(None);
                                refresh_trigger.set(refresh_trigger() + 1);
                            },
                            on_cancel: move |_| show_flush_dialog.set(None),
                        }
                    }
                }

    if let Some(import_id) = show_import_dialog() {
                    if let Some(pool) = connection_pools.read().get(&import_id).cloned() {
                        ImportOverlaySection {
                            pool,
                            on_close: move |_| show_import_dialog.set(None),
                        }
                    }
                }

    if show_export_connections_dialog() {
                    if let Some(storage) = config_storage.read().as_ref() {
                        ExportConnectionsDialogSection {
                            config_storage: Arc::new(storage.clone()),
                            colors,
                            on_close: move |_| show_export_connections_dialog.set(false),
                        }
                    }
                }

    if show_import_connections_dialog() {
        if let Some(storage) = config_storage.read().as_ref() {
            {
                let config_storage_arc = Arc::new(storage.clone());
                rsx! {
                    ImportConnectionsDialogSection {
                        config_storage: config_storage_arc.clone(),
                        colors,
                        on_import: import_connections_action(
                            show_import_connections_dialog,
                            connections,
                            readonly_connections,
                            toast_manager,
                            config_storage_arc.clone(),
                        ),
                        on_close: move |_| show_import_connections_dialog.set(false),
                    }
                }
            }
        }
    }

    {
        let update_status = UPDATE_STATUS();
        if let Some(ref info) = update_status.pending_update {
            rsx! {
                UpdateDialog {
                    update_info: info.clone(),
                    colors,
                    on_update: {
                        let update_download_progress = update_download_progress.clone();
                        let update_downloaded_path = update_downloaded_path.clone();
                        let toast_for_update = toast_for_update.clone();
                        let info_for_closure = info.clone();
                        move |_| {
                            let info_clone = info_for_closure.clone();
                            let mut update_download_progress = update_download_progress.clone();
                            let mut update_downloaded_path = update_downloaded_path.clone();
                            let mut toast_for_update = toast_for_update.clone();
                            spawn(async move {
                                if let Ok(mut manager) = UpdateManager::new() {
                                    let (tx, mut rx) = tokio::sync::mpsc::channel::<(u64, u64)>(100);
                                    let mut progress = update_download_progress.clone();
                                    spawn(async move {
                                        while let Some((downloaded, total)) = rx.recv().await {
                                            progress.set((downloaded, total));
                                        }
                                    });
                                    match manager.download_update(&info_clone, Some(tx)).await {
                                        Ok(path) => {
                                            update_downloaded_path.set(Some(path.clone()));
                                            match manager.install_update(&path) {
                                                Ok(result) => {
                                                    set_pending_update(None);
                                                    match result {
                                                        InstallResult::RestartRequired => {
                                                            toast_for_update.write().success("更新完成，请重启应用");
                                                        }
                                                        InstallResult::RestartInProgress => {
                                                            toast_for_update.write().success("正在安装更新...");
                                                        }
                                                        InstallResult::OpenExternal(url) => {
                                                            let _ = open::that(&url);
                                                            toast_for_update.write().success("请在浏览器中下载更新");
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    let msg = format!("安装失败: {}", e);
                                                    toast_for_update.write().error(&msg);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let msg = format!("下载失败: {}", e);
                                            toast_for_update.write().error(&msg);
                                        }
                                    }
                                }
                            });
                        }
                    },
                    on_skip: {
                        move |version: String| {
                            spawn(async move {
                                if let Ok(mut manager) = UpdateManager::new() {
                                    let _ = manager.skip_version(&version);
                                }
                                set_pending_update(None);
                            });
                        }
                    },
                    on_close: move |_| {
                        set_pending_update(None);
                    },
                }
            }
        } else {
            rsx! {}
        }
    }

    ToastContainer { manager: toast_manager }
    ImagePreview {}
            }
}
