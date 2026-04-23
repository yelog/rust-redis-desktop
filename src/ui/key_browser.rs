use crate::connection::ConnectionPool;
use crate::i18n::use_i18n;
use crate::redis::{KeyType, TreeBuilder, TreeNode};
use crate::theme::{
    ThemeColors, COLOR_BG, COLOR_BG_LOWEST, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER,
    COLOR_ERROR, COLOR_ERROR_BG, COLOR_OUTLINE_VARIANT, COLOR_PRIMARY, COLOR_TEXT,
    COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use crate::ui::add_key_dialog::AddKeyDialog;
use crate::ui::batch_ttl_dialog::BatchTtlDialog;
use crate::ui::context_menu::{ContextMenu, ContextMenuItem, ContextMenuState};
use crate::ui::delete_confirm_dialog::{DeleteConfirmDialog, DeleteTarget};
use crate::ui::export_dialog::{ExportDialog, ExportTarget};
use crate::ui::icons::*;
use crate::ui::memory_analysis_dialog::MemoryAnalysisDialog;
use crate::ui::pattern_delete_dialog::PatternDeleteDialog;
use crate::ui::{
    copy_text_to_clipboard, LazyTreeNode, ResizableDivider, ToastManager, TreeState, ValueViewer,
    VirtualTreeList,
};
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

const SCAN_BATCH_SIZE: usize = 500;

fn collect_all_node_ids(nodes: &[TreeNode]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for node in nodes {
        ids.insert(node.node_id.clone());
        ids.extend(collect_all_node_ids(&node.children));
    }
    ids
}

fn collect_all_folder_paths(nodes: &[TreeNode]) -> HashSet<String> {
    let mut paths = HashSet::new();
    for node in nodes {
        if !node.is_leaf {
            paths.insert(node.path.clone());
            paths.extend(collect_all_folder_paths(&node.children));
        }
    }
    paths
}

fn collect_all_keys(nodes: &[TreeNode]) -> Vec<String> {
    let mut keys = Vec::new();
    for node in nodes {
        if node.is_leaf {
            keys.push(node.path.clone());
        }
        keys.extend(collect_all_keys(&node.children));
    }
    keys
}

fn key_match_pattern(search_pattern: &str) -> String {
    if search_pattern.trim().is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", search_pattern.trim())
    }
}

fn selection_toolbar_style() -> String {
    format!(
        "margin-top: 8px; padding: 6px 8px; background: {}; border: 1px solid {}; \
         border-radius: 8px; display: flex; align-items: center; justify-content: space-between; gap: 8px;",
        COLOR_BG, COLOR_BORDER
    )
}

fn selection_count_style() -> String {
    format!(
        "padding: 0 8px; height: 24px; display: inline-flex; align-items: center; border-radius: 999px; \
         background: {}; border: 1px solid {}; color: {}; font-size: 11px; font-weight: 600; white-space: nowrap;",
        COLOR_BG_LOWEST, COLOR_BORDER, COLOR_TEXT_SECONDARY
    )
}

fn compact_secondary_button_style(disabled: bool) -> String {
    format!(
        "height: 24px; padding: 0 8px; background: {}; color: {}; border: 1px solid {}; border-radius: 6px; \
         cursor: {}; font-size: 11px; font-weight: 600; opacity: {};",
        COLOR_BG_LOWEST,
        COLOR_TEXT,
        COLOR_BORDER,
        if disabled { "default" } else { "pointer" },
        if disabled { "0.55" } else { "1" }
    )
}

fn compact_primary_button_style(disabled: bool) -> String {
    format!(
        "height: 24px; padding: 0 8px; background: {}; color: {}; border: 1px solid {}; border-radius: 6px; \
         cursor: {}; font-size: 11px; font-weight: 600; opacity: {};",
        COLOR_PRIMARY,
        COLOR_TEXT_CONTRAST,
        COLOR_PRIMARY,
        if disabled { "default" } else { "pointer" },
        if disabled { "0.55" } else { "1" }
    )
}

fn compact_danger_button_style(disabled: bool) -> String {
    format!(
        "height: 24px; padding: 0 8px; background: {}; color: {}; border: 1px solid {}; border-radius: 6px; \
         cursor: {}; font-size: 11px; font-weight: 600; opacity: {};",
        COLOR_ERROR_BG,
        COLOR_ERROR,
        COLOR_ERROR,
        if disabled { "default" } else { "pointer" },
        if disabled { "0.55" } else { "1" }
    )
}

#[derive(Clone, Default)]
pub struct ScanProgress {
    pub scanned: usize,
    pub current_batch: usize,
    pub is_scanning: bool,
}

async fn scan_all_keys(
    pool: ConnectionPool,
    match_pattern: String,
    cancel_flag: Arc<AtomicBool>,
    mut scan_progress: Signal<ScanProgress>,
) -> Result<Vec<String>, String> {
    let mut all_keys = Vec::new();
    let mut cursor: u64 = 0;

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            tracing::info!("Scan cancelled by user");
            break;
        }

        match pool
            .scan_keys_with_cursor(&match_pattern, cursor, SCAN_BATCH_SIZE)
            .await
        {
            Ok((next_cursor, keys)) => {
                let batch_len = keys.len();
                all_keys.extend(keys);

                scan_progress.write().scanned = all_keys.len();
                scan_progress.write().current_batch = batch_len;
                cursor = next_cursor;

                if cursor == 0 {
                    break;
                }
            }
            Err(error) => return Err(error.to_string()),
        }
    }

    Ok(all_keys)
}

#[component]
pub fn KeyBrowser(
    connection_id: Uuid,
    connection_pool: ConnectionPool,
    connection_version: u32,
    selected_key: Signal<String>,
    current_db: Signal<u8>,
    refresh_trigger: Signal<u32>,
    colors: ThemeColors,
    on_connection_error: EventHandler<()>,
    on_key_select: EventHandler<String>,
) -> Element {
    let i18n = use_i18n();
    let tree_nodes = use_signal(Vec::<TreeNode>::new);
    let mut search_input = use_signal(String::new);
    let mut search_pattern = use_signal(String::new);
    let loading = use_signal(|| false);
    let keys_count = use_signal(|| 0usize);
    let mut show_delete_dialog = use_signal(|| None::<Vec<DeleteTarget>>);
    let mut show_add_key_dialog = use_signal(|| false);
    let db_keys_count = use_signal(HashMap::<u8, u64>::new);
    let mut show_batch_ttl_dialog = use_signal(|| None::<(Vec<String>, Option<i64>)>);
    let mut show_pattern_delete_dialog = use_signal(|| false);
    let mut show_memory_analysis_dialog = use_signal(|| false);
    let scan_progress = use_signal(ScanProgress::default);
    let cancel_scan = use_signal(|| Arc::new(AtomicBool::new(false)));
    let key_type_cache = use_signal(HashMap::<String, KeyType>::new);
    let pending_type_keys = use_signal(Vec::<String>::new);
    let mut tree_state = use_signal(TreeState::default);
    let mut context_menu = use_signal(|| None::<ContextMenuState<(String, bool)>>);
    let key_list_width = use_signal(|| 320.0);
    let mut show_export_dialog = use_signal(|| None::<Vec<ExportTarget>>);
    let mut toast_manager = use_context::<Signal<ToastManager>>();
    let expanded_paths = use_signal(HashSet::<String>::new);
    let use_virtual_scroll = use_signal(|| true);
    let mut db_menu = use_signal(|| None::<ContextMenuState<()>>);
    let mut toolbar_menu = use_signal(|| None::<ContextMenuState<()>>);

    {
        let pool = connection_pool.clone();
        let key_type_cache = key_type_cache.clone();
        let pending_type_keys = pending_type_keys.clone();

        use_future(move || {
            let pool = pool.clone();
            let mut key_type_cache = key_type_cache.clone();
            let mut pending_type_keys = pending_type_keys.clone();
            async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    let keys: Vec<String> = std::mem::take(&mut *pending_type_keys.write());
                    if keys.is_empty() {
                        continue;
                    }
                    let cache = key_type_cache.read();
                    let uncached: Vec<String> = keys
                        .into_iter()
                        .filter(|k| !cache.contains_key(k.as_str()))
                        .collect();
                    drop(cache);
                    if uncached.is_empty() {
                        continue;
                    }
                    match pool.get_key_types(&uncached).await {
                        Ok(types) => {
                            let mut cache = key_type_cache.write();
                            for (key, key_type) in types {
                                cache.insert(key, key_type);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to batch fetch key types: {}", e);
                        }
                    }
                }
            }
        });
    }

    {
        let show_delete_dialog = show_delete_dialog.clone();
        let show_add_key_dialog = show_add_key_dialog.clone();
        let show_batch_ttl_dialog = show_batch_ttl_dialog.clone();
        let show_export_dialog = show_export_dialog.clone();
        let show_pattern_delete_dialog = show_pattern_delete_dialog.clone();

        use_future(move || {
            let mut show_delete_dialog = show_delete_dialog.clone();
            let mut show_add_key_dialog = show_add_key_dialog.clone();
            let mut show_batch_ttl_dialog = show_batch_ttl_dialog.clone();
            let mut show_export_dialog = show_export_dialog.clone();
            let mut show_pattern_delete_dialog = show_pattern_delete_dialog.clone();
            async move {
                let mut eval = dioxus::document::eval(
                    r#"
                    document.addEventListener('keydown', function(e) {
                        if (e.key === 'Escape') {
                            dioxus.send('escape_key_browser');
                        }
                    });
                    await new Promise(() => {});
                    "#,
                );
                while let Ok(msg) = eval.recv::<String>().await {
                    if msg == "escape_key_browser" {
                        if show_delete_dialog().is_some() {
                            show_delete_dialog.set(None);
                        } else if show_add_key_dialog() {
                            show_add_key_dialog.set(false);
                        } else if show_batch_ttl_dialog().is_some() {
                            show_batch_ttl_dialog.set(None);
                        } else if show_export_dialog().is_some() {
                            show_export_dialog.set(None);
                        } else if show_pattern_delete_dialog() {
                            show_pattern_delete_dialog.set(false);
                        }
                    }
                }
            }
        });
    }

    let load_keyspace = {
        let pool = connection_pool.clone();
        let db_keys_count = db_keys_count.clone();
        move || {
            let pool = pool.clone();
            let mut db_keys_count = db_keys_count.clone();
            spawn(async move {
                match pool.get_server_info().await {
                    Ok(info) => {
                        let mut map = HashMap::new();
                        for (db, keys) in info.keyspace {
                            if let Some(db_num) = db.strip_prefix("db") {
                                if let Ok(num) = db_num.parse::<u8>() {
                                    map.insert(num, keys);
                                }
                            }
                        }
                        db_keys_count.set(map);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load keyspace: {}", e);
                    }
                }
            });
        }
    };

    let load_keys = {
        let pool = connection_pool.clone();
        let search_pattern = search_pattern.clone();
        let loading = loading.clone();
        let tree_nodes = tree_nodes.clone();
        let keys_count = keys_count.clone();
        let load_keyspace = load_keyspace.clone();
        let scan_progress = scan_progress.clone();
        let mut cancel_scan = cancel_scan.clone();
        let tree_state = tree_state.clone();
        let key_type_cache = key_type_cache.clone();
        let expanded_paths = expanded_paths.clone();
        let toast_manager = toast_manager.clone();
        let on_connection_error = on_connection_error.clone();
        move || {
            let pool = pool.clone();
            let search_snapshot = search_pattern.peek().clone();
            let is_searching = !search_snapshot.trim().is_empty();
            let match_pattern = key_match_pattern(&search_snapshot);
            let mut loading = loading.clone();
            let mut tree_nodes = tree_nodes.clone();
            let mut keys_count = keys_count.clone();
            let load_keyspace = load_keyspace.clone();
            let mut scan_progress = scan_progress.clone();
            let mut tree_state = tree_state.clone();
            {
                let previous = cancel_scan.peek();
                previous.store(true, Ordering::Relaxed);
            }
            let cancel_flag = Arc::new(AtomicBool::new(false));
            cancel_scan.set(cancel_flag.clone());
            let mut key_type_cache = key_type_cache.clone();
            let mut expanded_paths = expanded_paths.clone();
            let mut toast_manager = toast_manager.clone();
            let on_connection_error = on_connection_error.clone();

            spawn(async move {
                loading.set(true);
                let preserved_expanded = tree_state.read().expanded_nodes.clone();
                scan_progress.write().is_scanning = true;
                scan_progress.write().scanned = 0;
                scan_progress.write().current_batch = 0;

                let all_keys = match pool.ensure_connection().await {
                    Ok(_) => match scan_all_keys(
                        pool.clone(),
                        match_pattern.clone(),
                        cancel_flag.clone(),
                        scan_progress,
                    )
                    .await
                    {
                        Ok(keys) => keys,
                        Err(first_error) => {
                            tracing::warn!(
                                "Initial key scan failed for connection {}: {}",
                                connection_id,
                                first_error
                            );
                            scan_progress.write().scanned = 0;
                            scan_progress.write().current_batch = 0;

                            match pool.ensure_connection().await {
                                Ok(_) => match scan_all_keys(
                                    pool.clone(),
                                    match_pattern.clone(),
                                    cancel_flag.clone(),
                                    scan_progress,
                                )
                                .await
                                {
                                    Ok(keys) => keys,
                                    Err(retry_error) => {
                                        let message = format!(
                                            "Failed to refresh keys after reconnect: {}",
                                            retry_error
                                        );
                                        tracing::error!(
                                            "Key refresh failed after reconnect for connection {}: {}",
                                            connection_id,
                                            retry_error
                                        );
                                        toast_manager.write().error(&message);
                                        on_connection_error.call(());
                                        loading.set(false);
                                        scan_progress.write().is_scanning = false;
                                        return;
                                    }
                                },
                                Err(reconnect_error) => {
                                    let message = format!(
                                        "Failed to reconnect before refreshing keys: {}",
                                        reconnect_error
                                    );
                                    tracing::error!(
                                        "Reconnect failed before retrying key refresh for connection {}: {}",
                                        connection_id,
                                        reconnect_error
                                    );
                                    toast_manager.write().error(&message);
                                    on_connection_error.call(());
                                    loading.set(false);
                                    scan_progress.write().is_scanning = false;
                                    return;
                                }
                            }
                        }
                    },
                    Err(error) => {
                        let message =
                            format!("Failed to reconnect before refreshing keys: {}", error);
                        tracing::error!(
                            "Reconnect failed before key refresh for connection {}: {}",
                            connection_id,
                            error
                        );
                        toast_manager.write().error(&message);
                        on_connection_error.call(());
                        loading.set(false);
                        scan_progress.write().is_scanning = false;
                        return;
                    }
                };

                keys_count.set(all_keys.len());
                key_type_cache.set(HashMap::new());

                if cancel_flag.load(Ordering::Relaxed) {
                    loading.set(false);
                    scan_progress.write().is_scanning = false;
                    return;
                }

                let builder = TreeBuilder::new(":");
                let new_nodes = builder.build(all_keys);
                let new_node_ids = collect_all_node_ids(&new_nodes);

                if is_searching {
                    let all_folder_paths = collect_all_folder_paths(&new_nodes);
                    let all_folder_node_ids: HashSet<String> = all_folder_paths
                        .iter()
                        .map(|p| format!("folder:{p}"))
                        .collect();
                    expanded_paths.set(all_folder_paths);
                    tree_nodes.set(new_nodes);
                    let mut state = tree_state.write();
                    state.expanded_nodes = all_folder_node_ids;
                    state.selected_keys.clear();
                    state.selection_mode = false;
                } else {
                    let valid_expanded: HashSet<String> = preserved_expanded
                        .into_iter()
                        .filter(|id| new_node_ids.contains(id))
                        .collect();
                    let valid_expanded_paths: HashSet<String> = valid_expanded
                        .iter()
                        .filter_map(|id| id.strip_prefix("folder:").map(|s| s.to_string()))
                        .collect();
                    expanded_paths.set(valid_expanded_paths);
                    tree_nodes.set(new_nodes);
                    let mut state = tree_state.write();
                    state.expanded_nodes = valid_expanded;
                    state.selected_keys.clear();
                    state.selection_mode = false;
                }

                loading.set(false);
                scan_progress.write().is_scanning = false;
                load_keyspace();
            });
        }
    };

    let toggle_node = {
        let mut tree_state = tree_state.clone();
        move |node_id: String| {
            let mut state = tree_state.write();
            if state.expanded_nodes.contains(&node_id) {
                state.expanded_nodes.remove(&node_id);
            } else {
                state.expanded_nodes.insert(node_id);
            }
        }
    };

    use_effect({
        let mut load_keys = load_keys.clone();
        move || {
            let _ = refresh_trigger();
            let _ = connection_version;
            let _ = connection_id;
            load_keys();
        }
    });

    let selected_count = tree_state.read().selected_keys.len();
    let current_selection_mode = tree_state.read().selection_mode;

    rsx! {
        div {
            width: "100%",
            height: "100%",
            min_height: "0",
            background: COLOR_BG,
            display: "flex",
            box_sizing: "border-box",
            overflow: "hidden",

            div {
                width: "{key_list_width()}px",
                min_width: "200px",
                height: "100%",
                background: COLOR_BG_SECONDARY,
                border_right: "1px solid {COLOR_BORDER}",
                display: "flex",
                flex_direction: "column",
                box_sizing: "border-box",
                overflow: "hidden",

                div {
                    padding: "12px",
                    border_bottom: "1px solid {COLOR_BORDER}",
                    background: COLOR_BG_SECONDARY,

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "6px",

                        button {
                            width: "60px",
                            height: "32px",
                            padding: "0 8px",
                            background: COLOR_BG_TERTIARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            cursor: "pointer",
                            display: "flex",
                            align_items: "center",
                            justify_content: "space_between",
                            color: COLOR_TEXT,
                            box_sizing: "border-box",
                            flex_shrink: "0",
                            title: i18n.read().t("Select database"),
                            onclick: move |e| {
                                let coords = e.client_coordinates();
                                toolbar_menu.set(None);
                                db_menu.set(Some(ContextMenuState::new(
                                    (),
                                    coords.x as i32,
                                    coords.y as i32,
                                )));
                            },

                            span {
                                color: COLOR_TEXT,
                                font_size: "12px",
                                font_weight: "700",
                                line_height: "1",
                                white_space: "nowrap",

                                "DB {current_db()}"
                            }

                            span {
                                color: COLOR_TEXT_SUBTLE,
                                font_size: "10px",
                                flex_shrink: "0",

                                "▾"
                            }
                        }

                        div {
                            flex: "1",
                            min_width: "0",
                            display: "flex",
                            align_items: "center",
                            gap: "6px",
                            padding: "0 8px 0 10px",
                            height: "32px",
                            background: COLOR_BG_LOWEST,
                            border: "1px solid {COLOR_OUTLINE_VARIANT}",
                            border_radius: "6px",

                            IconSearch { size: Some(14) }

                            input {
                                flex: "1",
                                min_width: "0",
                                background: "transparent",
                                border: "none",
                                color: COLOR_TEXT,
                                font_size: "12px",
                                placeholder: i18n.read().t("Enter to search"),
                                value: "{search_input}",
                                autocapitalize: "off",
                                autocorrect: "off",
                                oninput: move |e| {
                                    search_input.set(e.value());
                                },
                                onkeydown: move |e| {
                                    if e.data().key() == Key::Enter {
                                        search_pattern.set(search_input());
                                        refresh_trigger.set(refresh_trigger() + 1);
                                    }
                                },
                            }

                            div {
                                width: "1px",
                                height: "14px",
                                background: COLOR_BORDER,
                                flex_shrink: "0",
                            }

                            button {
                                width: "24px",
                                height: "24px",
                                background: "transparent",
                                color: COLOR_TEXT_SECONDARY,
                                border: "none",
                                border_radius: "4px",
                                cursor: "pointer",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                flex_shrink: "0",
                                title: i18n.read().t("Refresh keys"),
                                aria_label: i18n.read().t("Refresh keys"),
                                onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),

                                IconRefresh { size: Some(14) }
                            }

                            button {
                                width: "24px",
                                height: "24px",
                                background: "transparent",
                                color: COLOR_TEXT_SECONDARY,
                                border: "none",
                                border_radius: "4px",
                                cursor: "pointer",
                                display: "flex",
                                align_items: "center",
                                justify_content: "center",
                                flex_shrink: "0",
                                title: i18n.read().t("More actions"),
                                aria_label: i18n.read().t("More actions"),
                                onclick: move |e| {
                                    let coords = e.client_coordinates();
                                    db_menu.set(None);
                                    toolbar_menu.set(Some(ContextMenuState::new(
                                        (),
                                        coords.x as i32,
                                        coords.y as i32,
                                    )));
                                },

                                IconMoreHorizontal { size: Some(14) }
                            }
                        }

                        button {
                            width: "32px",
                            height: "32px",
                            background: COLOR_PRIMARY,
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            display: "flex",
                            align_items: "center",
                            justify_content: "center",
                            flex_shrink: "0",
                            title: i18n.read().t("Add key"),
                            aria_label: i18n.read().t("Add key"),
                            onclick: move |_| show_add_key_dialog.set(true),

                            IconPlus { size: Some(14) }
                        }
                    }

                    if scan_progress.read().is_scanning {
                        div {
                            margin_top: "8px",
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",

                            {format!("{} {}", scan_progress.read().scanned, i18n.read().t("keys scanned"))}
                        }
                    }

                    if current_selection_mode {
                        div {
                            style: "{selection_toolbar_style()}",

                            span {
                                style: "{selection_count_style()}",

                                {format!("{} {}", i18n.read().t("Selected"), selected_count)}
                            }

                            div {
                                display: "flex",
                                align_items: "center",
                                gap: "4px",

                                button {
                                    style: "{compact_secondary_button_style(false)}",
                                    onclick: move |_| {
                                        let all_keys = collect_all_keys(&tree_nodes());
                                        tree_state.write().selected_keys.extend(all_keys);
                                    },

                                    {i18n.read().t("Select all")}
                                }

                                button {
                                    style: "{compact_secondary_button_style(false)}",
                                    onclick: move |_| tree_state.write().selected_keys.clear(),

                                    {i18n.read().t("Clear")}
                                }

                                button {
                                    style: "{compact_primary_button_style(selected_count == 0)}",
                                    disabled: selected_count == 0,
                                    onclick: {
                                        let keys: Vec<String> = tree_state.read().selected_keys.iter().cloned().collect();
                                        move |_| show_batch_ttl_dialog.set(Some((keys.clone(), None)))
                                    },

                                    "TTL"
                                }

                                button {
                                    style: "{compact_danger_button_style(selected_count == 0)}",
                                    disabled: selected_count == 0,
                                    onclick: {
                                        let keys: Vec<String> = tree_state.read().selected_keys.iter().cloned().collect();
                                        move |_| {
                                            let targets = keys
                                                .iter()
                                                .map(|key| DeleteTarget {
                                                    key: key.clone(),
                                                    is_folder: false,
                                                })
                                                .collect();
                                            show_delete_dialog.set(Some(targets));
                                        }
                                    },

                                    {i18n.read().t("Delete")}
                                }
                            }
                        }
                    }
                }

                div {
                    flex: "1",
                    min_height: "0",
                    overflow_y: "auto",
                    padding: "4px 0",

                    if tree_nodes.read().is_empty() {
                        div {
                            padding: "40px 20px",
                            text_align: "center",
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "13px",

                            if loading() {
                                {i18n.read().t("Loading...")}
                            } else {
                                {i18n.read().t("No keys found")}
                            }
                        }
                    } else if use_virtual_scroll() && !tree_state.read().selection_mode {
                        VirtualTreeList {
                            nodes: tree_nodes.read().clone(),
                            selected_key: selected_key(),
                            expanded_paths: expanded_paths,
                            search_keyword: search_input(),
                            key_type_cache: key_type_cache,
                            on_select: {
                                let on_key_select = on_key_select.clone();
                                move |key: String| {
                                    on_key_select.call(key);
                                }
                            },
                            on_toggle: {
                                let mut expanded_paths = expanded_paths.clone();
                                move |path: String| {
                                    let mut expanded = expanded_paths.write();
                                    if expanded.contains(&path) {
                                        expanded.remove(&path);
                                    } else {
                                        expanded.insert(path);
                                    }
                                }
                            },
                            on_visible_keys_change: {
                                let mut pending_type_keys = pending_type_keys.clone();
                                move |keys: Vec<String>| {
                                    if !keys.is_empty() {
                                        pending_type_keys.set(keys);
                                    }
                                }
                            },
                            context_menu: context_menu,
                        }
                    } else {
                        for node in tree_nodes.read().iter() {
                            LazyTreeNode {
                                key: "{node.node_id}",
                                node: node.clone(),
                                depth: 0,
                                selected_key: selected_key(),
                                tree_state: tree_state,
                                search_keyword: search_input(),
                                on_select: {
                                    let on_key_select = on_key_select.clone();
                                    move |key: String| {
                                        on_key_select.call(key);
                                    }
                                },
                                on_expand: {
                                    let mut toggle_node = toggle_node.clone();
                                    move |node_id: String| {
                                        toggle_node(node_id);
                                    }
                                },
                                context_menu: context_menu,
                            }
                        }
                    }
                }

                div {
                    padding: "8px 16px",
                    border_top: "1px solid {COLOR_BORDER}",
                    color: COLOR_TEXT_SUBTLE,
                    font_size: "11px",

                    {format!("{} {}", keys_count(), i18n.read().t("Keys"))}
                }
            }

            ResizableDivider {
                size: key_list_width,
                min_size: 200.0,
                max_size: 500.0,
            }

            div {
                flex: "1",
                min_width: "0",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                overflow: "hidden",

                ValueViewer {
                    key: "{connection_id}",
                    connection_pool: connection_pool.clone(),
                    connection_version: connection_version,
                    selected_key: selected_key,
                    on_connection_error: move |_| on_connection_error.call(()),
                    on_refresh: move |_| {
                        refresh_trigger.set(refresh_trigger() + 1);
                    },
                }
            }
        }

        if let Some(menu) = context_menu() {
            {
                let menu_id = menu.id;
                let x = menu.x;
                let y = menu.y;
                let (node_path, is_leaf) = menu.data.clone();
                let mut context_menu_for_close = context_menu.clone();
                rsx! {
                    ContextMenu {
                        key: "{menu_id}",
                        menu_id: menu_id,
                        x: x,
                        y: y,
                        on_close: move |closing_menu_id| {
                            if context_menu_for_close()
                                .as_ref()
                                .map(|menu| menu.id)
                                == Some(closing_menu_id)
                            {
                                context_menu_for_close.set(None);
                            }
                        },

                        ContextMenuItem {
                            icon: Some(rsx! { IconCopy { size: Some(14) } }),
                            label: i18n.read().t("Copy path"),
                            danger: false,
                            disabled: false,
                            onclick: {
                                let node_path = node_path.clone();
                                move |_| {
                                    context_menu.set(None);
                                    if copy_text_to_clipboard(&node_path).is_ok() {
                                        toast_manager.write().success(&i18n.read().t("Path copied"));
                                    }
                                }
                            },
                        }

                        ContextMenuItem {
                            icon: Some(rsx! { IconDownload { size: Some(14) } }),
                            label: i18n.read().t("Export"),
                            danger: false,
                            disabled: false,
                            onclick: {
                                let node_path = node_path.clone();
                                let is_leaf = is_leaf;
                                move |_| {
                                    context_menu.set(None);
                                    show_export_dialog.set(Some(vec![ExportTarget {
                                        key: node_path.clone(),
                                        is_folder: !is_leaf,
                                    }]));
                                }
                            },
                        }

                        ContextMenuItem {
                            icon: Some(rsx! { IconClock { size: Some(14) } }),
                            label: i18n.read().t("Set TTL"),
                            danger: false,
                            disabled: !is_leaf,
                            onclick: {
                                let node_path = node_path.clone();
                                let pool = connection_pool.clone();
                                move |_| {
                                    context_menu.set(None);
                                    let pool = pool.clone();
                                    let node_path = node_path.clone();
                                    spawn(async move {
                                        let ttl = pool.get_key_info(&node_path).await.ok().and_then(|info| info.ttl);
                                        show_batch_ttl_dialog.set(Some((vec![node_path], ttl)));
                                    });
                                }
                            },
                        }

                        ContextMenuItem {
                            icon: Some(rsx! { IconRefresh { size: Some(14) } }),
                            label: i18n.read().t("Rename"),
                            danger: false,
                            disabled: !is_leaf,
                            onclick: {
                                let _node_path = node_path.clone();
                                move |_| {
                                    context_menu.set(None);
                                    toast_manager.write().success(&i18n.read().t("Rename is not implemented yet"));
                                }
                            },
                        }

                        ContextMenuItem {
                            icon: Some(rsx! { IconTrash { size: Some(14) } }),
                            label: i18n.read().t("Delete"),
                            danger: true,
                            disabled: false,
                            onclick: {
                                let node_path = node_path.clone();
                                let is_leaf = is_leaf;
                                move |_| {
                                    context_menu.set(None);
                                    show_delete_dialog.set(Some(vec![DeleteTarget {
                                        key: node_path.clone(),
                                        is_folder: !is_leaf,
                                    }]));
                                }
                            },
                        }
                    }
                }
            }
        }

        if let Some(menu) = db_menu() {
            {
                let menu_id = menu.id;
                let x = menu.x;
                let y = menu.y;
                let mut db_menu_for_close = db_menu.clone();

                rsx! {
                    ContextMenu {
                        key: "{menu_id}",
                        menu_id: menu_id,
                        x: x,
                        y: y,
                        on_close: move |closing_menu_id| {
                            if db_menu_for_close()
                                .as_ref()
                                .map(|menu| menu.id)
                                == Some(closing_menu_id)
                            {
                                db_menu_for_close.set(None);
                            }
                        },

                            for i in 0..16u8 {
                                {
                                    let keys = db_keys_count.read().get(&i).copied().unwrap_or(0);
                                    let label = format!("DB {} · {} {}", i, keys, i18n.read().t("Keys"));
                                    let pool = connection_pool.clone();
                                    let mut current_db_signal = current_db.clone();
                                    let mut refresh_trigger_signal = refresh_trigger.clone();
                                    let mut selected_key_signal = selected_key.clone();
                                    rsx! {
                                        ContextMenuItem {
                                            icon: Some(rsx! { IconDatabase { size: Some(14) } }),
                                            label: label,
                                            danger: false,
                                            disabled: current_db() == i,
                                            onclick: move |_| {
                                                db_menu.set(None);
                                                let pool = pool.clone();
                                                spawn(async move {
                                                    match pool.select_database(i).await {
                                                        Ok(_) => {
                                                            selected_key_signal.set(String::new());
                                                            current_db_signal.set(i);
                                                            refresh_trigger_signal.set(refresh_trigger_signal() + 1);
                                                        }
                                                        Err(error) => {
                                                            tracing::error!("Failed to select database: {}", error);
                                                        }
                                                    }
                                                });
                                            },
                                        }
                                    }
                                }
                        }
                    }
                }
            }
        }

        if let Some(menu) = toolbar_menu() {
            {
                let menu_id = menu.id;
                let x = menu.x;
                let y = menu.y;
                let mut toolbar_menu_for_close = toolbar_menu.clone();

                rsx! {
                    ContextMenu {
                        key: "{menu_id}",
                        menu_id: menu_id,
                        x: x,
                        y: y,
                        on_close: move |closing_menu_id| {
                            if toolbar_menu_for_close()
                                .as_ref()
                                .map(|menu| menu.id)
                                == Some(closing_menu_id)
                            {
                                toolbar_menu_for_close.set(None);
                            }
                        },

                        ContextMenuItem {
                            icon: Some(rsx! { IconSearch { size: Some(14) } }),
                            label: i18n.read().t("Memory analysis"),
                            danger: false,
                            disabled: false,
                            onclick: move |_| {
                                toolbar_menu.set(None);
                                show_memory_analysis_dialog.set(true);
                            },
                        }

                        ContextMenuItem {
                            icon: Some(rsx! { IconTrash { size: Some(14) } }),
                            label: i18n.read().t("Delete by pattern"),
                            danger: true,
                            disabled: false,
                            onclick: move |_| {
                                toolbar_menu.set(None);
                                show_pattern_delete_dialog.set(true);
                            },
                        }

                        ContextMenuItem {
                            icon: Some(rsx! {
                                if current_selection_mode {
                                    IconX { size: Some(14) }
                                } else {
                                    IconCheck { size: Some(14) }
                                }
                            }),
                            label: if current_selection_mode {
                                i18n.read().t("Exit multi-select")
                            } else {
                                i18n.read().t("Enter multi-select")
                            },
                            danger: false,
                            disabled: false,
                            onclick: move |_| {
                                toolbar_menu.set(None);
                                tree_state.write().selection_mode = !current_selection_mode;
                                if current_selection_mode {
                                    tree_state.write().selected_keys.clear();
                                }
                            },
                        }

                        if scan_progress.read().is_scanning {
                            ContextMenuItem {
                                icon: Some(rsx! { IconX { size: Some(14) } }),
                                label: i18n.read().t("Cancel scan"),
                                danger: true,
                                disabled: false,
                                onclick: {
                                    let cancel_scan = cancel_scan.clone();
                                    move |_| {
                                        toolbar_menu.set(None);
                                        cancel_scan.read().store(true, Ordering::Relaxed);
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }

        if let Some(targets) = show_delete_dialog() {
            DeleteConfirmDialog {
                connection_pool: connection_pool.clone(),
                targets: targets.clone(),
                colors,
                on_confirm: move |_| {
                    show_delete_dialog.set(None);
                    tree_state.write().selected_keys.clear();
                    selected_key.set(String::new());
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                on_cancel: move |_| show_delete_dialog.set(None),
            }
        }

        if show_add_key_dialog() {
            AddKeyDialog {
                connection_pool: connection_pool.clone(),
                colors,
                on_save: {
                    let mut refresh_trigger = refresh_trigger.clone();
                    let mut selected_key = selected_key.clone();
                    move |key: String| {
                        show_add_key_dialog.set(false);
                        selected_key.set(key);
                        refresh_trigger.set(refresh_trigger() + 1);
                    }
                },
                on_cancel: move |_| show_add_key_dialog.set(false),
            }
        }

        if let Some((keys, current_ttl)) = show_batch_ttl_dialog() {
            BatchTtlDialog {
                connection_pool: connection_pool.clone(),
                keys: keys.clone(),
                current_ttl,
                colors,
                on_confirm: move |_| {
                    show_batch_ttl_dialog.set(None);
                    tree_state.write().selected_keys.clear();
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                on_cancel: move |_| show_batch_ttl_dialog.set(None),
            }
        }

        if let Some(targets) = show_export_dialog() {
            ExportDialog {
                connection_pool: connection_pool.clone(),
                targets: targets.clone(),
                colors,
                on_close: move |_| show_export_dialog.set(None),
            }
        }

        if show_pattern_delete_dialog() {
            PatternDeleteDialog {
                connection_pool: connection_pool.clone(),
                initial_pattern: search_input(),
                colors,
                on_confirm: {
                    let mut refresh_trigger = refresh_trigger.clone();
                    let mut toast_manager = toast_manager.clone();
                    move |deleted_count: usize| {
                        show_pattern_delete_dialog.set(false);
                        refresh_trigger.set(refresh_trigger() + 1);
                        toast_manager
                            .write()
                            .success(&format!("{} {}", i18n.read().t("Deleted"), deleted_count));
                    }
                },
                on_cancel: move |_| show_pattern_delete_dialog.set(false),
            }
        }

        if show_memory_analysis_dialog() {
            MemoryAnalysisDialog {
                connection_pool: connection_pool.clone(),
                colors,
                on_select_key: {
                    let on_key_select = on_key_select.clone();
                    move |key: String| {
                        show_memory_analysis_dialog.set(false);
                        on_key_select.call(key);
                    }
                },
                on_close: move |_| show_memory_analysis_dialog.set(false),
            }
        }
    }
}
