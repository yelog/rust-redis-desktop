use crate::config::{AppSettings, ConfigStorage};
use crate::connection::{ConnectionConfig, ConnectionManager, ConnectionPool, ConnectionState};
use crate::theme::{resolve_theme, theme_spec, ThemePreference, ThemeSpec, COLOR_BG, COLOR_BG_SECONDARY, COLOR_SURFACE_LOW, COLOR_BG_LOWEST, COLOR_TEXT, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE, COLOR_TEXT_CONTRAST, COLOR_BORDER, COLOR_ACCENT, COLOR_ERROR, COLOR_PRIMARY};
use crate::ui::{
    ClientsPanel, ConnectionForm, FlushConfirmDialog, ImportPanel, KeyBrowser, LeftRail, MonitorPanel,
    PubSubPanel, ResizableDivider, ScriptPanel, SettingsDialog, SlowLogPanel, Terminal,
};
use dioxus::prelude::*;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Data,
    Terminal,
    Monitor,
    SlowLog,
    Clients,
    PubSub,
    Script,
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

fn build_theme_palette(theme: ThemeSpec) -> Value {
    let colors = theme.colors;
    let derived = theme.derived;
    let syntax = theme.syntax;
    let mut palette = Map::new();

    fn insert_str(palette: &mut Map<String, Value>, key: &str, value: &str) {
        palette.insert(key.to_string(), Value::String(value.to_string()));
    }

    insert_str(&mut palette, "id", theme.id.as_str());
    insert_str(&mut palette, "label", theme.label);
    palette.insert("isDark".to_string(), Value::Bool(theme.is_dark()));
    insert_str(&mut palette, "surfaceBase", colors.background);
    insert_str(
        &mut palette,
        "surfaceSecondary",
        colors.background_secondary,
    );
    insert_str(&mut palette, "surfaceTertiary", colors.background_tertiary);
    insert_str(&mut palette, "surfaceLowest", colors.surface_lowest);
    insert_str(&mut palette, "surfaceLow", colors.surface_low);
    insert_str(&mut palette, "surfaceHigh", colors.surface_high);
    insert_str(&mut palette, "surfaceHighest", colors.surface_highest);
    insert_str(&mut palette, "border", colors.border);
    insert_str(&mut palette, "outlineVariant", colors.outline_variant);
    insert_str(&mut palette, "overlayBackdrop", derived.overlay_backdrop);
    insert_str(&mut palette, "controlBg", derived.control_bg);
    insert_str(&mut palette, "controlBorder", derived.control_border);
    insert_str(&mut palette, "buttonSecondary", derived.button_secondary);
    insert_str(
        &mut palette,
        "buttonSecondaryBorder",
        derived.button_secondary_border,
    );
    insert_str(&mut palette, "textPrimary", colors.text);
    insert_str(&mut palette, "textSecondary", colors.text_secondary);
    insert_str(&mut palette, "textSubtle", colors.text_subtle);
    insert_str(&mut palette, "textSoft", derived.text_soft);
    insert_str(&mut palette, "textContrast", derived.text_contrast);
    insert_str(&mut palette, "primary", colors.primary);
    insert_str(&mut palette, "accent", colors.accent);
    insert_str(&mut palette, "success", colors.success);
    insert_str(&mut palette, "warning", colors.warning);
    insert_str(&mut palette, "error", colors.error);
    insert_str(&mut palette, "info", derived.info);
    insert_str(&mut palette, "outline", derived.outline);
    insert_str(&mut palette, "secondaryAction", derived.secondary_action);
    insert_str(&mut palette, "infoBg", derived.info_bg);
    insert_str(&mut palette, "infoBgAlt", derived.info_bg_alt);
    insert_str(&mut palette, "successBg", derived.success_bg);
    insert_str(&mut palette, "successBgAlt", derived.success_bg_alt);
    insert_str(&mut palette, "warningBg", derived.warning_bg);
    insert_str(&mut palette, "errorBg", derived.error_bg);
    insert_str(&mut palette, "selectionBg", derived.selection_bg);
    insert_str(&mut palette, "selectionBgAlt", derived.selection_bg_alt);
    insert_str(&mut palette, "rowCreateBg", derived.row_create_bg);
    insert_str(&mut palette, "rowEditBg", derived.row_edit_bg);
    insert_str(&mut palette, "toneStringBg", derived.tone_string_bg);
    insert_str(&mut palette, "toneStringBorder", derived.tone_string_border);
    insert_str(&mut palette, "toneHashBg", derived.tone_hash_bg);
    insert_str(&mut palette, "toneHashBorder", derived.tone_hash_border);
    insert_str(&mut palette, "toneListBg", derived.tone_list_bg);
    insert_str(&mut palette, "toneListBorder", derived.tone_list_border);
    insert_str(&mut palette, "toneSetBg", derived.tone_set_bg);
    insert_str(&mut palette, "toneSetBorder", derived.tone_set_border);
    insert_str(&mut palette, "toneZsetBg", derived.tone_zset_bg);
    insert_str(&mut palette, "toneZsetBorder", derived.tone_zset_border);
    insert_str(&mut palette, "toneStreamBg", derived.tone_stream_bg);
    insert_str(&mut palette, "toneStreamBorder", derived.tone_stream_border);
    insert_str(&mut palette, "syntaxKey", syntax.key);
    insert_str(&mut palette, "syntaxString", syntax.string);
    insert_str(&mut palette, "syntaxNumber", syntax.number);
    insert_str(&mut palette, "syntaxBoolean", syntax.boolean);
    insert_str(&mut palette, "syntaxNull", syntax.null);
    insert_str(&mut palette, "syntaxBracket", syntax.bracket);
    insert_str(&mut palette, "syntaxKeyword", syntax.keyword);
    insert_str(&mut palette, "syntaxType", syntax.type_name);
    insert_str(&mut palette, "syntaxFunction", syntax.function);
    insert_str(&mut palette, "syntaxComment", syntax.comment);
    insert_str(&mut palette, "syntaxOperator", syntax.operator);
    insert_str(&mut palette, "syntaxConstant", syntax.constant);

    Value::Object(palette)
}

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
    if (bridge.selectedPreference === "system") {{
      return window.matchMedia("(prefers-color-scheme: dark)").matches
        ? bridge.themes.classic_dark
        : bridge.themes.classic_light;
    }}

    return bridge.themes[bridge.selectedPreference] || bridge.themes.classic_dark;
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

    root.dataset.themeMode = bridge.selectedPreference;
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
    let mut app_settings = use_signal(AppSettings::default);
    let mut show_settings = use_signal(|| false);
    let mut show_flush_dialog = use_signal(|| None::<Uuid>);
    let mut show_import_dialog = use_signal(|| None::<Uuid>);
    let mut current_db = use_signal(|| 0u8);
    let mut theme_preference = use_signal(ThemePreference::default);
    let mut system_theme_dark = use_signal(system_theme_is_dark);
    let mut left_rail_width = use_signal(|| 280.0);

    let active_theme_preference = theme_preference();
    let active_system_theme_dark = system_theme_dark();
    let active_theme = resolve_theme(active_theme_preference, active_system_theme_dark);
    let colors = active_theme.colors;
    let resolved_theme_id = active_theme.id;
    let resolved_theme_key = resolved_theme_id.as_str();

    use_effect(move || {
        if let Some(storage) = config_storage.read().as_ref() {
            if let Ok(saved) = storage.load_connections() {
                connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
            }
            if let Ok(settings) = storage.load_settings() {
                app_settings.set(settings.clone());
                theme_preference.set(settings.theme_preference);
            }
        }
    });

    use_effect(move || {
        let script = build_theme_bridge_script(theme_preference());
        let _ = document::eval(&script);
    });

    use_future(move || {
        let mut show_settings = show_settings.clone();
        let mut form_mode = form_mode.clone();
        let mut show_flush_dialog = show_flush_dialog.clone();
        async move {
            let mut eval = document::eval(
                r#"
document.addEventListener('keydown', (e) => {
    if (e.key === ',' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        dioxus.send('toggle_settings');
    }
    if (e.key === 'Escape') {
        dioxus.send('escape_pressed');
    }
});
await new Promise(() => {});
"#,
            );

            while let Ok(msg) = eval.recv::<String>().await {
                if msg == "toggle_settings" {
                    show_settings.toggle();
                } else if msg == "escape_pressed" {
                    if show_settings() {
                        show_settings.set(false);
                    } else if form_mode().is_some() {
                        form_mode.set(None);
                    } else if show_flush_dialog().is_some() {
                        show_flush_dialog.set(None);
                    }
                }
            }
        }
    });

    #[cfg(target_os = "macos")]
    {
        use dioxus::desktop::use_muda_event_handler;
        let mut show_settings_for_menu = show_settings.clone();
        use_muda_event_handler(move |event| {
            if event.id == "preferences" {
                show_settings_for_menu.toggle();
            }
        });
    }

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
        let mut theme_preference = theme_preference.clone();
        move |settings: AppSettings| {
            app_settings.set(settings.clone());
            theme_preference.set(settings.theme_preference);
            if let Some(storage) = config_storage.read().as_ref() {
                let _ = storage.save_settings(&settings);
            }
        }
    };

    let selected_conn_state = selected_connection()
        .and_then(|id| connection_states.read().get(&id).copied())
        .unwrap_or(ConnectionState::Disconnected);

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
                            let previous_conn = selected_connection();

                            selected_key.set(String::new());
                            current_tab.set(Tab::Data);

                            if previous_conn != Some(id) {
                                connection_states
                                    .write()
                                    .insert(id, ConnectionState::Connecting);
                            }

                            selected_connection.set(Some(id));

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
                                    connection_states.write().insert(id, ConnectionState::Connected);
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
                        on_import_connection: move |id: Uuid| {
                            show_import_dialog.set(Some(id));
                        },
                        on_open_settings: move |_| show_settings.set(true),
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

                                    "正在加载连接..."
                                }
                            }
                        } else if selected_conn_state == ConnectionState::Connected {
                            if let Some(pool) = connection_pools.read().get(&conn_id).cloned() {
                            div {
                                flex: "1",
                                min_width: "0",
                                min_height: "0",
                                display: "flex",
                                flex_direction: "column",
                                background: "{COLOR_SURFACE_LOW}",
                                overflow: "hidden",

                                div {
                                    display: "flex",
                                    align_items: "center",
                                    gap: "8px",
                                    padding: "10px 16px",
                                    border_bottom: "1px solid {COLOR_BORDER}",
                                    background: "{COLOR_BG_SECONDARY}",

                                    for (tab, label) in [
                                        (Tab::Data, "数据"),
                                        (Tab::Terminal, "终端"),
                                        (Tab::Monitor, "监控"),
                                        (Tab::SlowLog, "慢日志"),
                                        (Tab::Clients, "客户端"),
                                        (Tab::PubSub, "Pub/Sub"),
                                        (Tab::Script, "脚本"),
                                    ] {
                                        button {
                                            padding: "8px 14px",
                                            background: if current_tab() == tab { COLOR_BG } else { "transparent" },
                                            color: if current_tab() == tab { COLOR_TEXT } else { COLOR_TEXT_SECONDARY },
                                            border: if current_tab() == tab {
                                                format!("1px solid {}", COLOR_BORDER)
                                            } else {
                                                "1px solid transparent".to_string()
                                            },
                                            border_bottom: if current_tab() == tab {
                                                format!("2px solid {}", COLOR_ACCENT)
                                            } else {
                                                "2px solid transparent".to_string()
                                            },
                                            border_radius: "6px",
                                            cursor: "pointer",
                                            font_size: "13px",
                                            font_weight: if current_tab() == tab { "700" } else { "500" },
                                            transition: "all 150ms ease-out",
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
                                            colors,
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
                                    } else if current_tab() == Tab::Clients {
                                        ClientsPanel {
                                            key: "{conn_id}",
                                            connection_pool: pool.clone(),
                                        }
                                    } else if current_tab() == Tab::PubSub {
                                        PubSubPanel {
                                            key: "{conn_id}",
                                            connection_pool: pool.clone(),
                                        }
                                    } else {
                                        ScriptPanel {
                                            key: "{conn_id}",
                                            connection_pool: pool.clone(),
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

                                    "正在初始化连接..."
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

                                    "正在连接..."
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
                            color: "{COLOR_TEXT_SECONDARY}",
                            background: "{COLOR_SURFACE_LOW}",

                            div {
                                font_size: "28px",
                                font_weight: "700",
                                color: "{COLOR_TEXT}",

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
                    colors,
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
                        colors,
                        resolved_theme_id,
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
                    div {
                        position: "fixed",
                        top: "0",
                        left: "0",
                        right: "0",
                        bottom: "0",
                        background: "rgba(0, 0, 0, 0.5)",
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        z_index: "1000",

                        ImportPanel {
                            connection_pool: pool,
                            on_close: move |_| show_import_dialog.set(None),
                        }
                    }
                }
            }
        }
    }
