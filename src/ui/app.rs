use crate::config::{AppSettings, ConfigStorage};
use crate::connection::{ConnectionConfig, ConnectionManager, ConnectionPool, ConnectionState};
use crate::theme::{ThemeColors, ThemeMode};
use crate::ui::{
    ClientsPanel, ConnectionForm, FlushConfirmDialog, KeyBrowser, LeftRail, MonitorPanel,
    SettingsDialog, SlowLogPanel, Terminal,
};
use dioxus::prelude::*;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Data,
    Terminal,
    Monitor,
    SlowLog,
    Clients,
}

#[derive(Clone, PartialEq)]
pub enum FormMode {
    New,
    Edit(ConnectionConfig),
}

fn system_theme_is_dark() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains("Dark"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

fn resolve_theme_colors(mode: ThemeMode, system_is_dark: bool) -> ThemeColors {
    match mode {
        ThemeMode::Dark => ThemeColors::dark(),
        ThemeMode::Light => ThemeColors::light(),
        ThemeMode::System => {
            if system_is_dark {
                ThemeColors::dark()
            } else {
                ThemeColors::light()
            }
        }
    }
}

fn build_theme_palette(colors: &ThemeColors, is_dark: bool) -> serde_json::Value {
    json!({
        "isDark": is_dark,
        "surfaceBase": colors.background,
        "surfaceSecondary": colors.background_secondary,
        "surfaceTertiary": colors.background_tertiary,
        "surfaceLowest": colors.surface_lowest,
        "surfaceLow": colors.surface_low,
        "surfaceHigh": colors.surface_high,
        "surfaceHighest": colors.surface_highest,
        "border": colors.border,
        "outlineVariant": colors.outline_variant,
        "controlBg": if is_dark { "#353535" } else { "#fffaf8" },
        "controlBorder": colors.outline_variant,
        "buttonSecondary": colors.surface_highest,
        "buttonSecondaryBorder": colors.outline_variant,
        "textPrimary": colors.text,
        "textSecondary": colors.text_secondary,
        "textSubtle": colors.text_subtle,
        "textSoft": if is_dark { "#e5e2e1" } else { "#6f5953" },
        "textContrast": "#ffffff",
        "primary": colors.primary,
        "accent": colors.accent,
        "success": colors.success,
        "warning": colors.warning,
        "error": colors.error,
        "info": if is_dark { "#00daf3" } else { colors.accent },
        "outline": colors.text_subtle,
        "infoBg": if is_dark { "#1c1b1b" } else { "#e4f4f6" },
        "infoBgAlt": if is_dark { "#2a2a2a" } else { "#d8ecef" },
        "successBg": if is_dark { "#1a3a1a" } else { "#edf7f0" },
        "successBgAlt": if is_dark { "#1e4620" } else { "#e3f1e7" },
        "errorBg": if is_dark { "#2d1f1f" } else { "#fff1ef" },
        "selectionBg": if is_dark { "#2a2a2a" } else { "rgba(0, 127, 142, 0.12)" },
        "selectionBgAlt": if is_dark { "#1a4a1a" } else { "rgba(177, 44, 25, 0.08)" },
        "syntaxKey": if is_dark { "#e2bfb8" } else { "#9f2d1f" },
        "syntaxString": if is_dark { "#00daf3" } else { "#006d79" },
        "syntaxNumber": if is_dark { "#ffb4a6" } else { "#b15d00" },
        "syntaxBoolean": if is_dark { "#a98a84" } else { "#b42318" },
        "syntaxNull": if is_dark { "#5a413c" } else { "#8c6f68" },
        "syntaxBracket": if is_dark { "#e5e2e1" } else { "#241917" },
        "syntaxKeyword": if is_dark { "#569cd6" } else { "#005f73" },
        "syntaxType": if is_dark { "#dcdcaa" } else { "#7a5c1f" },
    })
}

fn build_theme_bridge_script(mode: ThemeMode) -> String {
    let selected_mode = match mode {
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "light",
        ThemeMode::System => "system",
    };
    let dark_theme = build_theme_palette(&ThemeColors::dark(), true);
    let light_theme = build_theme_palette(&ThemeColors::light(), false);

    format!(
        r##"
(() => {{
  const bridge = (window.__rrdThemeBridge = window.__rrdThemeBridge || {{}});
  bridge.selectedMode = {selected_mode};
  bridge.dark = {dark_theme};
  bridge.light = {light_theme};

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
    [["#1a1a2e"], "infoBg"],
    [["#1f2937", "#202a33"], "infoBgAlt"],
    [["#1a3a1a"], "successBg"],
    [["#1a4a1a", "#1e4620"], "successBgAlt"],
    [["#2d1f1f"], "errorBg"],
    [["#094771"], "selectionBg"],
  ];

  const themedAccentAliases = [
    [["#0e639c", "#3182ce"], "primary"],
    [["#007acc", "#4ec9b0"], "accent"],
    [["#38a169", "#68d391", "#22c55e", "#4ade80"], "success"],
    [["#f59e0b"], "warning"],
    [["#c53030", "#f87171", "#ef4444", "#f44336"], "error"],
    [["#63b3ed"], "info"],
    [["#805ad5", "#a78bfa"], "purple"],
    [["#9cdcfe"], "syntaxKey"],
    [["#ce9178"], "syntaxString"],
    [["#b5cea8"], "syntaxNumber"],
    [["#569cd6"], "syntaxKeyword"],
    [["#dcdcaa"], "syntaxType"],
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
    if (bridge.selectedMode === "system") {{
      return window.matchMedia("(prefers-color-scheme: dark)").matches ? bridge.dark : bridge.light;
    }}

    return bridge.selectedMode === "dark" ? bridge.dark : bridge.light;
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
      bridge.dark.primary,
      bridge.light.primary,
      bridge.dark.success,
      bridge.light.success,
      bridge.dark.error,
      bridge.light.error,
      bridge.dark.warning,
      bridge.light.warning,
      bridge.dark.buttonSecondary,
      bridge.light.buttonSecondary,
      bridge.dark.purple,
      bridge.light.purple,
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

    root.dataset.themeMode = bridge.selectedMode;
    root.dataset.themeResolved = theme.isDark ? "dark" : "light";
    root.style.colorScheme = theme.isDark ? "dark" : "light";
    root.style.setProperty("--theme-bg", theme.surfaceBase);
    root.style.setProperty("--theme-bg-secondary", theme.surfaceSecondary);
    root.style.setProperty("--theme-bg-tertiary", theme.surfaceTertiary);
    root.style.setProperty("--theme-bg-lowest", theme.surfaceLowest);
    root.style.setProperty("--theme-surface-low", theme.surfaceLow);
    root.style.setProperty("--theme-surface-high", theme.surfaceHigh);
    root.style.setProperty("--theme-surface-highest", theme.surfaceHighest);
    root.style.setProperty("--theme-border", theme.border);
    root.style.setProperty("--theme-text", theme.textPrimary);
    root.style.setProperty("--theme-text-secondary", theme.textSecondary);
    root.style.setProperty("--theme-text-subtle", theme.textSubtle);
    root.style.setProperty("--theme-text-soft", theme.textSoft);
    root.style.setProperty("--theme-primary", theme.primary);
    root.style.setProperty("--theme-accent", theme.accent);
    root.style.setProperty("--theme-success", theme.success);
    root.style.setProperty("--theme-warning", theme.warning);
    root.style.setProperty("--theme-error", theme.error);
    root.style.setProperty("--theme-info", theme.info);
    root.style.setProperty("--theme-outline", theme.outline);
    root.style.setProperty("--theme-outline-variant", theme.outlineVariant);
    root.style.setProperty("--theme-syntax-key", theme.syntaxKey);
    root.style.setProperty("--theme-syntax-string", theme.syntaxString);
    root.style.setProperty("--theme-syntax-number", theme.syntaxNumber);
    root.style.setProperty("--theme-syntax-boolean", theme.syntaxBoolean);
    root.style.setProperty("--theme-syntax-null", theme.syntaxNull);
    root.style.setProperty("--theme-syntax-bracket", theme.syntaxBracket);
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
        selected_mode = serde_json::to_string(selected_mode).unwrap(),
        dark_theme = dark_theme,
        light_theme = light_theme,
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
    let mut app_settings = use_signal(AppSettings::default);
    let mut show_settings = use_signal(|| false);
    let mut show_flush_dialog = use_signal(|| None::<Uuid>);
    let mut current_db = use_signal(|| 0u8);
    let mut theme_mode = use_signal(ThemeMode::default);
    let mut system_theme_dark = use_signal(system_theme_is_dark);
    let left_rail_width = 280.0;

    let active_theme_mode = *theme_mode.read();
    let active_system_theme_dark = system_theme_dark();
    let colors = resolve_theme_colors(active_theme_mode, active_system_theme_dark);
    let resolved_theme_key = match active_theme_mode {
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "light",
        ThemeMode::System => {
            if active_system_theme_dark {
                "dark"
            } else {
                "light"
            }
        }
    };

    use_effect(move || {
        if let Some(storage) = config_storage.read().as_ref() {
            if let Ok(saved) = storage.load_connections() {
                connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
            }
            if let Ok(settings) = storage.load_settings() {
                app_settings.set(settings.clone());
                theme_mode.set(settings.theme_mode);
            }
        }
    });

    use_effect(move || {
        let current_theme_mode = theme_mode();
        let script = build_theme_bridge_script(current_theme_mode);
        let _ = document::eval(&script);
    });

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

    let save_settings = {
        let config_storage = config_storage.clone();
        let mut theme_mode = theme_mode.clone();
        move |settings: AppSettings| {
            app_settings.set(settings.clone());
            theme_mode.set(settings.theme_mode);
            if let Some(storage) = config_storage.read().as_ref() {
                let _ = storage.save_settings(&settings);
            }
        }
    };

    let selected_conn_state = selected_connection()
        .and_then(|id| connection_states.read().get(&id).copied())
        .unwrap_or(ConnectionState::Disconnected);

    rsx! {
            div {
                display: "flex",
                flex_direction: "column",
                height: "100vh",
                background: "{colors.background}",
                color: "{colors.text}",
                overflow: "hidden",
                onkeydown: move |e| {
                    let key = e.data().key();
                    let modifiers = e.data().modifiers();
                    if key == Key::Character(",".to_string()) && modifiers.contains(Modifiers::SUPER) {
                        show_settings.set(true);
                    }
                },

                div {
                    flex: "1",
                    min_height: "0",
                    display: "flex",
                    overflow: "hidden",

                    LeftRail {
                        width: left_rail_width,
                        connections: connections(),
                        connection_states: connection_states(),
                        selected_connection: selected_connection(),
                        colors: colors.clone(),
                        on_add_connection: move |_| form_mode.set(Some(FormMode::New)),
                        on_select_connection: move |id: Uuid| {
                            selected_connection.set(Some(id));
                            selected_key.set(String::new());
                            current_tab.set(Tab::Data);

                            if let Some(pool) = connection_pools.read().get(&id).cloned() {
                                current_db.set(pool.current_db());
                            } else if let Some(storage) = config_storage.read().as_ref() {
                                if let Ok(saved) = storage.load_connections() {
                                    if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                        current_db.set(config.db);
                                    }
                                }
                            }

                            spawn(async move {
                                if let Some(pool) = connection_pools.read().get(&id).cloned() {
                                    let db = pool.current_db();
                                    if let Err(error) = pool.select_database(db).await {
                                        tracing::error!("Failed to sync database for connection {id}: {error}");
                                    }

                                    let version =
                                        connection_versions.read().get(&id).copied().unwrap_or(0);
                                    connection_versions.write().insert(id, version + 1);
                                    return;
                                }

                                connection_states.write().insert(id, ConnectionState::Connecting);

                                if let Some(pool) = connection_manager.read().get_connection(id).await {
                                    let db = pool.current_db();
                                    if let Err(error) = pool.select_database(db).await {
                                        tracing::error!("Failed to sync database for connection {id}: {error}");
                                    }
                                    current_db.set(db);
                                    connection_pools.write().insert(id, pool);
                                    connection_states.write().insert(id, ConnectionState::Connected);
                                    return;
                                }

                                if let Some(storage) = config_storage.read().as_ref() {
                                    if let Ok(saved) = storage.load_connections() {
                                        if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                            match ConnectionPool::new(config.clone()).await {
                                                Ok(pool) => {
                                                    current_db.set(pool.current_db());
                                                    let _ = connection_manager.read().add_connection(config).await;
                                                    connection_pools.write().insert(id, pool);
                                                    connection_states.write().insert(id, ConnectionState::Connected);
                                                }
                                                Err(_) => {
                                                    connection_states.write().insert(id, ConnectionState::Error);
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        },
                        on_reconnect_connection: move |id: Uuid| {
                            spawn(async move {
                                reconnecting_ids.write().insert(id);
                                connection_states.write().insert(id, ConnectionState::Connecting);

                                if let Some(storage) = config_storage.read().as_ref() {
                                    if let Ok(saved) = storage.load_connections() {
                                        if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                            match ConnectionPool::new(config.clone()).await {
                                                Ok(pool) => {
                                                    let db = pool.current_db();
                                                    connection_pools.write().insert(id, pool);
                                                    let _ = connection_manager.read().add_connection(config).await;

                                                    let version = connection_versions.read().get(&id).copied().unwrap_or(0);
                                                    connection_versions.write().insert(id, version + 1);
                                                    connection_states.write().insert(id, ConnectionState::Connected);
                                                    if selected_connection() == Some(id) {
                                                        current_db.set(db);
                                                    }
                                                }
                                                Err(_) => {
                                                    connection_states.write().insert(id, ConnectionState::Error);
                                                }
                                            }
                                        }
                                    }
                                }

                                reconnecting_ids.write().remove(&id);
                            });
                        },
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
                        on_edit_connection: move |id: Uuid| {
                            if let Some(storage) = config_storage.read().as_ref() {
                                if let Ok(saved) = storage.load_connections() {
                                    if let Some(config) = saved.into_iter().find(|c| c.id == id) {
                                        form_mode.set(Some(FormMode::Edit(config)));
                                    }
                                }
                            }
                        },
                        on_delete_connection: move |id: Uuid| {
                            spawn(async move {
                                if let Some(storage) = config_storage.read().as_ref() {
                                    let _ = storage.delete_connection(id);
                                }

                                connection_pools.write().remove(&id);
                                connection_manager.read().remove_connection(id).await;
                                connection_states.write().remove(&id);

                                if let Some(storage) = config_storage.read().as_ref() {
                                    if let Ok(saved) = storage.load_connections() {
                                        connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
                                    }
                                }

                                if selected_connection() == Some(id) {
                                    selected_connection.set(None);
                                    selected_key.set(String::new());
                                    current_db.set(0);
                                }
                            });
                        },
                        on_flush_connection: move |id: Uuid| {
                            show_flush_dialog.set(Some(id));
                        },
                        on_open_settings: move |_| show_settings.set(true),
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
                                background: "{colors.surface_low}",

                                style { {r#"
                                @keyframes spin {
                                    from { transform: rotate(0deg); }
                                    to { transform: rotate(360deg); }
                                }
                            "#} }

                                div {
                                    width: "40px",
                                    height: "40px",
                                    border: "3px solid {colors.accent}",
                                    border_top_color: "transparent",
                                    border_radius: "50%",
                                    animation: "spin 0.8s linear infinite",
                                }

                                div {
                                    color: "{colors.text_secondary}",
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
                                background: "{colors.surface_low}",

                                div {
                                    color: "{colors.error}",
                                    font_size: "14px",

                                    "连接失败，请检查连接配置后重试"
                                }

                                button {
                                    padding: "10px 20px",
                                    background: "{colors.primary}",
                                    color: "{colors.primary_text}",
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
                            div {
                                flex: "1",
                                display: "flex",
                                flex_direction: "column",
                                align_items: "center",
                                justify_content: "center",
                                gap: "16px",
                                background: "{colors.surface_low}",

                                style { {r#"
                                @keyframes spin {
                                    from { transform: rotate(0deg); }
                                    to { transform: rotate(360deg); }
                                }
                            "#} }

                                div {
                                    width: "40px",
                                    height: "40px",
                                    border: "3px solid {colors.accent}",
                                    border_top_color: "transparent",
                                    border_radius: "50%",
                                    animation: "spin 0.8s linear infinite",
                                }

                                div {
                                    color: "{colors.text_secondary}",
                                    font_size: "14px",

                                    "正在加载连接..."
                                }
                            }
                        } else if let Some(pool) = connection_pools.read().get(&conn_id).cloned() {
                            div {
                                flex: "1",
                                min_width: "0",
                                min_height: "0",
                                display: "flex",
                                flex_direction: "column",
                                background: "{colors.surface_low}",
                                overflow: "hidden",

                                div {
                                    display: "flex",
                                    align_items: "center",
                                    gap: "8px",
                                    padding: "10px 16px",
                                    border_bottom: "1px solid {colors.border}",
                                    background: "{colors.background_secondary}",

                                    for (tab, label) in [
                                        (Tab::Data, "数据"),
                                        (Tab::Terminal, "终端"),
                                        (Tab::Monitor, "监控"),
                                        (Tab::SlowLog, "慢日志"),
                                        (Tab::Clients, "客户端"),
                                    ] {
                                        button {
                                            padding: "8px 14px",
                                            background: if current_tab() == tab { colors.background } else { "transparent" },
                                            color: if current_tab() == tab { colors.text } else { colors.text_secondary },
                                            border: if current_tab() == tab {
                                                format!("1px solid {}", colors.border)
                                            } else {
                                                "1px solid transparent".to_string()
                                            },
                                            border_bottom: if current_tab() == tab {
                                                format!("2px solid {}", colors.accent)
                                            } else {
                                                "2px solid transparent".to_string()
                                            },
                                            border_radius: "6px",
                                            cursor: "pointer",
                                            font_size: "13px",
                                            font_weight: if current_tab() == tab { "700" } else { "500" },
                                            onclick: move |_| current_tab.set(tab),

                                            "{label}"
                                        }
                                    }
                                }

                                div {
                                    flex: "1",
                                    min_height: "0",
                                    display: "flex",
                                    flex_direction: "column",
                                    overflow: "hidden",

                                if current_tab() == Tab::Data {
                                div {
                                    flex: "1",
                                    min_height: "0px",
                                    overflow: "hidden",

                                        KeyBrowser {
                                            key: "{conn_id}-{connection_versions.read().get(&conn_id).copied().unwrap_or(0)}-{resolved_theme_key}",
                                            connection_id: conn_id,
                                            connection_pool: pool.clone(),
                                            connection_version: connection_versions.read().get(&conn_id).copied().unwrap_or(0),
                                            selected_key: selected_key,
                                            current_db: current_db,
                                            refresh_trigger: refresh_trigger,
                                            on_key_select: move |key: String| {
                                                selected_key.set(key);
                                                current_tab.set(Tab::Data);
                                            },
                                        }
                                    }
                                } else if current_tab() == Tab::Terminal {
                                        Terminal {
                                            key: "{conn_id}",
                                            connection_pool: pool.clone(),
                                        }
                                    } else if current_tab() == Tab::Monitor {
                                        MonitorPanel {
                                            key: "{conn_id}",
                                            connection_pool: pool.clone(),
                                            auto_refresh_interval: app_settings.read().auto_refresh_interval,
                                        }
                                    } else if current_tab() == Tab::SlowLog {
                                        SlowLogPanel {
                                            key: "{conn_id}",
                                            connection_pool: pool.clone(),
                                        }
                                    } else {
                                        ClientsPanel {
                                            key: "{conn_id}",
                                            connection_pool: pool.clone(),
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            flex: "1",
                            display: "flex",
                            flex_direction: "column",
                            align_items: "center",
                            justify_content: "center",
                            gap: "10px",
                            color: "{colors.text_secondary}",
                            background: "{colors.surface_low}",

                            div {
                                font_size: "28px",
                                font_weight: "700",
                                color: "{colors.text}",

                                "Redis 工作台"
                            }

                            div {
                                font_size: "14px",

                                "从左侧选择一个连接，或先创建新的 Redis 连接。"
                            }
                        }
                    }
                }
            }

            if let Some(mode) = form_mode() {
                ConnectionForm {
                    editing_config: match mode {
                        FormMode::Edit(config) => Some(config),
                        FormMode::New => None,
                    },
                    on_save: move |config: ConnectionConfig| {
                        let id = config.id;
                        let name = config.name.clone();

                        spawn(async move {
                            tracing::info!("=== Save Connection Start ===");
                            tracing::info!("Connection: {} ({})", name, id);

                            let storage = config_storage.read();
                            if storage.is_none() {
                                tracing::error!("ConfigStorage is None!");
                                form_mode.set(None);
                                return;
                            }

                            let storage = storage.as_ref().unwrap();

                            tracing::info!("Saving to storage...");
                            match storage.save_connection(config.clone()) {
                                Ok(_) => tracing::info!("✓ Config saved successfully"),
                                Err(e) => {
                                    tracing::error!("✗ Save failed: {}", e);
                                    form_mode.set(None);
                                    return;
                                }
                            }

                            tracing::info!("Reloading connections...");
                            match storage.load_connections() {
                                Ok(saved) => {
                                    let list: Vec<(Uuid, String)> = saved.into_iter().map(|c| (c.id, c.name)).collect();
                                    tracing::info!("✓ Loaded {} connections: {:?}", list.len(), list);
                                    connections.set(list);
                                }
                                Err(e) => {
                                    tracing::error!("✗ Load failed: {}", e);
                                }
                            }

                            let _ = connection_manager.read().add_connection(config).await;

                            tracing::info!("=== Save Connection End ===");
                            form_mode.set(None);
                        });
                    },
                    on_cancel: move |_| form_mode.set(None),
                }
            }

            if show_settings() {
                SettingsDialog {
                    settings: app_settings.read().clone(),
                    colors: colors.clone(),
                    on_save: {
                        let mut save_settings = save_settings.clone();
                        move |settings: AppSettings| {
                            save_settings(settings);
                        }
                    },
                    on_close: move |_| show_settings.set(false),
                }
            }

            if let Some(flush_id) = show_flush_dialog() {
                if let Some(pool) = connection_pools.read().get(&flush_id).cloned() {
                    FlushConfirmDialog {
                        connection_pool: pool,
                        current_db: current_db(),
                        on_confirm: move |_| {
                            show_flush_dialog.set(None);
                            refresh_trigger.set(refresh_trigger() + 1);
                        },
                        on_cancel: move |_| show_flush_dialog.set(None),
                    }
                }
            }
        }
}
