use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType, TreeBuilder, TreeNode};
use crate::theme::{
    COLOR_ACCENT, COLOR_BG, COLOR_BG_LOWEST, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER,
    COLOR_OUTLINE_VARIANT, COLOR_PRIMARY, COLOR_SURFACE_HIGH, COLOR_SURFACE_HIGHEST, COLOR_TEXT,
    COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use crate::ui::add_key_dialog::AddKeyDialog;
use crate::ui::batch_ttl_dialog::BatchTtlDialog;
use crate::ui::delete_confirm_dialog::{DeleteConfirmDialog, DeleteTarget};
use crate::ui::icons::*;
use crate::ui::{KeyTable, KeyTableRow};
use arboard::Clipboard;
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

const PAGE_SIZE: usize = 100;
const SCAN_BATCH_SIZE: usize = 500;
const TYPE_FILTER_OPTIONS: [(&str, &str); 7] = [
    ("all", "全部"),
    ("string", "String"),
    ("hash", "Hash"),
    ("list", "List"),
    ("set", "Set"),
    ("zset", "ZSet"),
    ("stream", "Stream"),
];

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

fn collect_leaf_nodes(nodes: &[TreeNode], leaves: &mut Vec<TreeNode>) {
    for node in nodes {
        if node.is_leaf {
            leaves.push(node.clone());
        } else {
            collect_leaf_nodes(&node.children, leaves);
        }
    }
}

fn update_node_key_info(nodes: &mut [TreeNode], key_path: &str, key_info: KeyInfo) -> bool {
    for node in nodes.iter_mut() {
        if node.is_leaf && node.path == key_path {
            node.key_info = Some(key_info);
            return true;
        }
        if update_node_key_info(&mut node.children, key_path, key_info.clone()) {
            return true;
        }
    }
    false
}

fn update_multiple_key_info(nodes: &mut [TreeNode], updates: &[(String, KeyInfo)]) -> bool {
    let mut changed = false;
    for (key_path, key_info) in updates {
        if update_node_key_info(nodes, key_path, key_info.clone()) {
            changed = true;
        }
    }
    changed
}

fn key_match_pattern(search_pattern: &str) -> String {
    if search_pattern.trim().is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", search_pattern.trim())
    }
}

fn type_filter_matches(key_type: Option<&KeyType>, filter: &str) -> bool {
    match filter {
        "string" => matches!(key_type, Some(KeyType::String)),
        "hash" => matches!(key_type, Some(KeyType::Hash)),
        "list" => matches!(key_type, Some(KeyType::List)),
        "set" => matches!(key_type, Some(KeyType::Set)),
        "zset" => matches!(key_type, Some(KeyType::ZSet)),
        "stream" => matches!(key_type, Some(KeyType::Stream)),
        _ => true,
    }
}

fn pattern_label(node: &TreeNode) -> String {
    if node.is_leaf {
        node.path.clone()
    } else {
        format!("{}*", node.path)
    }
}

fn type_filter_display(filter: &str) -> &str {
    match filter {
        "string" => "String",
        "hash" => "Hash",
        "list" => "List",
        "set" => "Set",
        "zset" => "ZSet",
        "stream" => "Stream",
        _ => "全部",
    }
}

#[derive(Clone, Default)]
pub struct ScanProgress {
    pub scanned: usize,
    pub current_batch: usize,
    pub is_scanning: bool,
}

#[component]
pub fn KeyBrowser(
    height: f64,
    connection_id: Uuid,
    connection_pool: ConnectionPool,
    connection_version: u32,
    selected_key: Signal<String>,
    current_db: Signal<u8>,
    refresh_trigger: Signal<u32>,
    on_key_select: EventHandler<String>,
) -> Element {
    let tree_nodes = use_signal(Vec::<TreeNode>::new);
    let mut search_pattern = use_signal(String::new);
    let loading = use_signal(|| false);
    let keys_count = use_signal(|| 0usize);
    let mut show_delete_dialog = use_signal(|| None::<Vec<DeleteTarget>>);
    let mut show_add_key_dialog = use_signal(|| false);
    let db_keys_count = use_signal(HashMap::<u8, u64>::new);
    let mut show_batch_ttl_dialog = use_signal(|| None::<Vec<String>>);
    let scan_progress = use_signal(ScanProgress::default);
    let cancel_scan = use_signal(|| Arc::new(AtomicBool::new(false)));
    let key_type_cache = use_signal(HashMap::<String, KeyType>::new);
    let mut selected_keys = use_signal(HashSet::<String>::new);
    let mut selection_mode = use_signal(|| false);
    let mut current_page = use_signal(|| 0usize);
    let mut type_filter = use_signal(|| "all".to_string());

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
        let selected_keys = selected_keys.clone();
        let selection_mode = selection_mode.clone();
        let mut current_page = current_page.clone();
        let mut key_type_cache = key_type_cache.clone();
        move || {
            let pool = pool.clone();
            let match_pattern = key_match_pattern(&search_pattern.read());
            let mut loading = loading.clone();
            let mut tree_nodes = tree_nodes.clone();
            let mut keys_count = keys_count.clone();
            let load_keyspace = load_keyspace.clone();
            let mut scan_progress = scan_progress.clone();
            let mut selected_keys = selected_keys.clone();
            let mut selection_mode = selection_mode.clone();
            let cancel_flag = Arc::new(AtomicBool::new(false));
            cancel_scan.set(cancel_flag.clone());

            spawn(async move {
                loading.set(true);
                tree_nodes.set(Vec::new());
                selected_keys.write().clear();
                selection_mode.set(false);
                current_page.set(0);
                key_type_cache.set(HashMap::new());
                scan_progress.write().is_scanning = true;
                scan_progress.write().scanned = 0;
                scan_progress.write().current_batch = 0;

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
                        Err(e) => {
                            tracing::error!("Failed to load keys: {}", e);
                            break;
                        }
                    }
                }

                keys_count.set(all_keys.len());

                let builder = TreeBuilder::new(":");
                tree_nodes.set(builder.build(all_keys));

                loading.set(false);
                scan_progress.write().is_scanning = false;
                load_keyspace();
            });
        }
    };

    let select_db = {
        let pool = connection_pool.clone();
        let mut refresh_trigger = refresh_trigger.clone();
        move |db: u8| {
            let pool = pool.clone();
            spawn(async move {
                match pool.select_database(db).await {
                    Ok(_) => {
                        current_db.set(db);
                        refresh_trigger.set(refresh_trigger() + 1);
                    }
                    Err(e) => {
                        tracing::error!("Failed to select database: {}", e);
                    }
                }
            });
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

    use_effect({
        let pool = connection_pool.clone();
        let tree_nodes = tree_nodes.clone();
        let key_type_cache = key_type_cache.clone();
        move || {
            let page = current_page();
            let filter = type_filter();
            let mut leaf_nodes = Vec::new();
            collect_leaf_nodes(&tree_nodes.read(), &mut leaf_nodes);

            let filtered_leaves: Vec<TreeNode> = leaf_nodes
                .into_iter()
                .filter(|node| {
                    type_filter_matches(node.key_info.as_ref().map(|info| &info.key_type), &filter)
                })
                .collect();

            let start = page.saturating_mul(PAGE_SIZE);
            let end = (start + PAGE_SIZE).min(filtered_leaves.len());
            if start >= end {
                return;
            }

            let cached_types = key_type_cache.read().clone();
            let keys_to_fetch: Vec<String> = filtered_leaves[start..end]
                .iter()
                .filter(|node| node.key_info.is_none() && !cached_types.contains_key(&node.path))
                .map(|node| node.path.clone())
                .collect();

            if keys_to_fetch.is_empty() {
                return;
            }

            let pool = pool.clone();
            let mut key_type_cache = key_type_cache.clone();
            let mut tree_nodes = tree_nodes.clone();
            spawn(async move {
                let mut updates = Vec::new();
                for key in keys_to_fetch {
                    match pool.get_key_info(&key).await {
                        Ok(info) => {
                            key_type_cache
                                .write()
                                .insert(key.clone(), info.key_type.clone());
                            updates.push((key, info));
                        }
                        Err(e) => tracing::error!("Failed to get key info: {}", e),
                    }
                }

                if !updates.is_empty() {
                    let mut nodes = tree_nodes.write();
                    update_multiple_key_info(&mut nodes, &updates);
                    drop(nodes);
                    let snapshot = tree_nodes.read().clone();
                    tree_nodes.set(snapshot);
                }
            });
        }
    });

    let mut leaf_nodes = Vec::new();
    collect_leaf_nodes(&tree_nodes.read(), &mut leaf_nodes);

    let current_filter = type_filter();
    let filtered_leaves: Vec<TreeNode> = leaf_nodes
        .into_iter()
        .filter(|node| {
            type_filter_matches(
                node.key_info.as_ref().map(|info| &info.key_type),
                &current_filter,
            )
        })
        .collect();

    let total_filtered = filtered_leaves.len();
    let total_pages = total_filtered.max(1).div_ceil(PAGE_SIZE);
    let page = current_page().min(total_pages.saturating_sub(1));
    let start_index = page.saturating_mul(PAGE_SIZE);
    let end_index = (start_index + PAGE_SIZE).min(total_filtered);
    let page_nodes = if start_index < end_index {
        filtered_leaves[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    let selected_count = selected_keys.read().len();
    let pattern_nodes = tree_nodes.read().clone();
    let active_match_pattern = key_match_pattern(&search_pattern());

    let rows: Vec<KeyTableRow> = page_nodes
        .iter()
        .map(|node| {
            let key_info = node.key_info.as_ref();
            KeyTableRow {
                key: node.path.clone(),
                key_type: key_info.map(|info| info.key_type.clone()),
                ttl: key_info.and_then(|info| info.ttl),
                has_details: key_info.is_some(),
                is_selected: selected_keys.read().contains(&node.path),
            }
        })
        .collect();

    rsx! {
        div {
            width: "100%",
            height: "{height}px",
            min_height: "0",
            background: COLOR_BG,
            display: "flex",
            flex_direction: "column",
            box_sizing: "border-box",
            overflow: "hidden",

            div {
                padding: "18px 20px 14px",
                border_bottom: "1px solid {COLOR_BORDER}",
                background: COLOR_BG_SECONDARY,
                display: "flex",
                justify_content: "space_between",
                align_items: "flex_end",
                gap: "16px",

                div {
                    display: "flex",
                    flex_direction: "column",
                    gap: "4px",

                    h1 {
                        margin: "0",
                        color: COLOR_TEXT,
                        font_size: "24px",
                        font_weight: "800",

                        "键浏览器"
                    }

                    div {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",

                        "DB {current_db()} • 共 {keys_count()} 个 Key"
                    }
                }

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    flex_wrap: "wrap",

                    button {
                        padding: "8px 12px",
                        background: COLOR_SURFACE_HIGHEST,
                        color: COLOR_TEXT,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "8px",
                        cursor: "pointer",
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),

                        IconRefresh { size: Some(14) }
                        "刷新"
                    }

                    div {
                        padding: "8px 12px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "8px",
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        font_size: "12px",

                        IconList { size: Some(14) }
                        "筛选 {type_filter_display(&type_filter())} • {total_filtered} 项"
                    }
                }
            }

            div {
                padding: "14px 20px",
                border_bottom: "1px solid {COLOR_BORDER}",
                background: COLOR_BG_SECONDARY,
                display: "flex",
                align_items: "center",
                gap: "10px",
                flex_wrap: "wrap",

                select {
                    width: "140px",
                    padding: "9px 12px",
                    background: COLOR_BG_TERTIARY,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "8px",
                    color: COLOR_TEXT,
                    font_size: "13px",
                    value: "db{current_db}",
                    onchange: move |e| {
                        if let Some(db_str) = e.value().strip_prefix("db") {
                            if let Ok(db) = db_str.parse::<u8>() {
                                select_db(db);
                            }
                        }
                    },

                    for i in 0..16u8 {
                        {
                            let keys = db_keys_count.read().get(&i).copied().unwrap_or(0);
                            let label = if keys > 0 {
                                format!("DB {} ({})", i, keys)
                            } else {
                                format!("DB {}", i)
                            };
                            rsx! {
                                option {
                                    value: "db{i}",
                                    selected: current_db() == i,

                                    "{label}"
                                }
                            }
                        }
                    }
                }

                div {
                    flex: "1",
                    min_width: "220px",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    padding: "0 12px",
                    height: "40px",
                    background: COLOR_BG_LOWEST,
                    border: "1px solid {COLOR_OUTLINE_VARIANT}",
                    border_radius: "8px",

                    IconSearch { size: Some(14) }

                    input {
                        flex: "1",
                        background: "transparent",
                        border: "none",
                        color: COLOR_TEXT,
                        font_size: "13px",
                        placeholder: "搜索 key、命名空间或模式",
                        value: "{search_pattern}",
                        oninput: move |e| search_pattern.set(e.value()),
                        onkeydown: move |e| {
                            if e.data().key() == Key::Enter {
                                refresh_trigger.set(refresh_trigger() + 1);
                            }
                        },
                    }
                }

                button {
                    padding: "9px 14px",
                    background: COLOR_PRIMARY,
                    color: COLOR_TEXT_CONTRAST,
                    border: "none",
                    border_radius: "8px",
                    cursor: "pointer",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    onclick: move |_| show_add_key_dialog.set(true),

                    IconPlus { size: Some(14) }
                    "新增"
                }

                if scan_progress.read().is_scanning {
                    button {
                        padding: "9px 14px",
                        background: "#c53030",
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "8px",
                        cursor: "pointer",
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        onclick: {
                            let cancel_scan = cancel_scan.clone();
                            move |_| cancel_scan.read().store(true, Ordering::Relaxed)
                        },

                        IconX { size: Some(14) }
                        "取消扫描"
                    }
                } else {
                    button {
                        padding: "9px 14px",
                        background: COLOR_SURFACE_HIGHEST,
                        color: COLOR_TEXT,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "8px",
                        cursor: "pointer",
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),

                        IconRefresh { size: Some(14) }
                        "运行扫描"
                    }
                }
            }

            div {
                padding: "0 20px 14px",
                background: COLOR_BG_SECONDARY,
                border_bottom: "1px solid {COLOR_BORDER}",

                div {
                    padding: "12px 14px",
                    background: COLOR_BG_LOWEST,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "8px",
                    display: "flex",
                    align_items: "center",
                    justify_content: "space_between",
                    gap: "16px",

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "10px",

                        IconTerminal { size: Some(14) }

                        span {
                            color: COLOR_ACCENT,
                            font_size: "13px",
                            font_family: "Consolas, 'Courier New', monospace",

                            "SCAN 0 MATCH {active_match_pattern} COUNT {SCAN_BATCH_SIZE}"
                        }
                    }

                    if scan_progress.read().is_scanning {
                        span {
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "12px",

                            "已扫描 {scan_progress.read().scanned} 个 key，当前批次 {scan_progress.read().current_batch}"
                        }
                    }
                }

                if !pattern_nodes.is_empty() {
                    div {
                        margin_top: "12px",
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        flex_wrap: "wrap",

                        span {
                            color: COLOR_TEXT_SUBTLE,
                            font_size: "11px",
                            text_transform: "uppercase",
                            letter_spacing: "0.12em",

                            "Key 模式"
                        }

                        for node in pattern_nodes.iter().take(6) {
                            {
                                let label = pattern_label(node);
                                let count = node.total_keys.max(1);
                                let query_value = if node.is_leaf {
                                    node.path.clone()
                                } else {
                                    node.path.clone()
                                };
                                rsx! {
                                    button {
                                        padding: "6px 10px",
                                        background: COLOR_SURFACE_HIGH,
                                        color: COLOR_TEXT_SECONDARY,
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "999px",
                                        cursor: "pointer",
                                        font_size: "12px",
                                        display: "flex",
                                        align_items: "center",
                                        gap: "8px",
                                        onclick: move |_| {
                                            search_pattern.set(query_value.clone());
                                            refresh_trigger.set(refresh_trigger() + 1);
                                        },

                                        span {
                                            font_family: "Consolas, 'Courier New', monospace",

                                            "{label}"
                                        }

                                        span {
                                            color: COLOR_TEXT_SUBTLE,

                                            "{count}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    margin_top: "12px",
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    flex_wrap: "wrap",

                    span {
                        color: COLOR_TEXT_SUBTLE,
                        font_size: "11px",
                        text_transform: "uppercase",
                        letter_spacing: "0.12em",

                        "类型筛选"
                    }

                    for (filter_key, label) in TYPE_FILTER_OPTIONS {
                        button {
                            key: "{filter_key}",
                            padding: "6px 10px",
                            background: if current_filter == filter_key {
                                "rgba(0, 218, 243, 0.12)"
                            } else {
                                COLOR_SURFACE_HIGH
                            },
                            color: if current_filter == filter_key {
                                COLOR_ACCENT
                            } else {
                                COLOR_TEXT_SECONDARY
                            },
                            border: if current_filter == filter_key {
                                "1px solid rgba(0, 218, 243, 0.28)".to_string()
                            } else {
                                format!("1px solid {}", COLOR_BORDER)
                            },
                            border_radius: "999px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| {
                                type_filter.set(filter_key.to_string());
                                current_page.set(0);
                            },

                            "{label}"
                        }
                    }
                }
            }

            if selection_mode() {
                div {
                    padding: "10px 20px",
                    background: "rgba(48, 209, 88, 0.10)",
                    border_bottom: "1px solid {COLOR_BORDER}",
                    display: "flex",
                    align_items: "center",
                    justify_content: "space_between",
                    gap: "12px",
                    flex_wrap: "wrap",

                    div {
                        color: COLOR_ACCENT,
                        font_size: "12px",

                        "已选 {selected_count} 项"
                    }

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        flex_wrap: "wrap",

                        button {
                            padding: "6px 10px",
                            background: "#38a169",
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| {
                                let all_keys = collect_all_keys(&tree_nodes());
                                selected_keys.write().extend(all_keys);
                            },

                            "全选"
                        }

                        button {
                            padding: "6px 10px",
                            background: COLOR_SURFACE_HIGHEST,
                            color: COLOR_TEXT,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| selected_keys.write().clear(),

                            "清空"
                        }

                        button {
                            padding: "6px 10px",
                            background: COLOR_PRIMARY,
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            disabled: selected_count == 0,
                            onclick: {
                                let keys: Vec<String> = selected_keys.read().iter().cloned().collect();
                                move |_| show_batch_ttl_dialog.set(Some(keys.clone()))
                            },

                            "批量 TTL"
                        }

                        button {
                            padding: "6px 10px",
                            background: "rgba(255, 180, 171, 0.10)",
                            color: "#ffb4ab",
                            border: "1px solid rgba(255, 180, 171, 0.24)",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            disabled: selected_count == 0,
                            onclick: {
                                let keys: Vec<String> = selected_keys.read().iter().cloned().collect();
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

                            "批量删除"
                        }

                        button {
                            padding: "6px 10px",
                            background: "transparent",
                            color: COLOR_TEXT_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            cursor: "pointer",
                            font_size: "12px",
                            onclick: move |_| {
                                selection_mode.set(false);
                                selected_keys.write().clear();
                            },

                            "关闭多选"
                        }
                    }
                }
            }

            div {
                flex: "1",
                min_height: "0",
                padding: "14px 20px 12px",
                background: COLOR_BG,
                overflow: "hidden",
                display: "flex",
                flex_direction: "column",
                gap: "12px",

                div {
                    display: "flex",
                    align_items: "center",
                    justify_content: "space_between",
                    gap: "12px",
                    flex_wrap: "wrap",

                    label {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "12px",
                        cursor: "pointer",

                        input {
                            r#type: "checkbox",
                            checked: selection_mode(),
                            onchange: move |e| {
                                selection_mode.set(e.checked());
                                if !e.checked() {
                                    selected_keys.write().clear();
                                }
                            },
                        }

                        "多选模式"
                    }

                    if type_filter() != "all" {
                        span {
                            padding: "4px 8px",
                            background: COLOR_SURFACE_HIGH,
                            color: COLOR_TEXT_SECONDARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "999px",
                            font_size: "11px",

                            "类型筛选：{type_filter_display(&type_filter())}"
                        }
                    }
                }

                if rows.is_empty() {
                    div {
                        flex: "1",
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        flex_direction: "column",
                        gap: "10px",
                        background: COLOR_BG_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "10px",
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "13px",

                        if loading() {
                            "正在加载 key..."
                        } else {
                            div {
                                display: "flex",
                                flex_direction: "column",
                                align_items: "center",
                                gap: "10px",

                                span { "没有找到匹配的 key" }

                                if !search_pattern().trim().is_empty() || type_filter() != "all" {
                                    button {
                                        padding: "8px 12px",
                                        background: COLOR_SURFACE_HIGHEST,
                                        color: COLOR_TEXT,
                                        border: "1px solid {COLOR_BORDER}",
                                        border_radius: "8px",
                                        cursor: "pointer",
                                        onclick: move |_| {
                                            search_pattern.set(String::new());
                                            type_filter.set("all".to_string());
                                            current_page.set(0);
                                            refresh_trigger.set(refresh_trigger() + 1);
                                        },

                                        "清空搜索与筛选"
                                    }
                                }
                            }
                        }
                    }
                } else {
                    KeyTable {
                        rows: rows,
                        selected_key: selected_key(),
                        selection_mode: selection_mode(),
                        on_select: {
                    let pool = connection_pool.clone();
                    let key_type_cache = key_type_cache.clone();
                    let tree_nodes = tree_nodes.clone();
                            let on_key_select = on_key_select.clone();
                            move |key: String| {
                                if !key_type_cache.read().contains_key(&key) {
                                    let pool = pool.clone();
                                    let mut key_type_cache = key_type_cache.clone();
                                    let mut tree_nodes = tree_nodes.clone();
                                    let fetch_key = key.clone();
                                    spawn(async move {
                                        match pool.get_key_info(&fetch_key).await {
                                            Ok(info) => {
                                                key_type_cache
                                                    .write()
                                                    .insert(fetch_key.clone(), info.key_type.clone());
                                                let updated = {
                                                    let mut nodes = tree_nodes.write();
                                                    update_node_key_info(&mut nodes, &fetch_key, info)
                                                };
                                                if updated {
                                                    let snapshot = tree_nodes.read().clone();
                                                    tree_nodes.set(snapshot);
                                                }
                                            }
                                            Err(e) => tracing::error!("Failed to get key info: {}", e),
                                        }
                                    });
                                }

                                on_key_select.call(key);
                            }
                        },
                        on_toggle_select: move |key: String| {
                            let mut keys = selected_keys.write();
                            if keys.contains(&key) {
                                keys.remove(&key);
                            } else {
                                keys.insert(key);
                            }
                        },
                        on_copy_key: move |key: String| {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(key);
                            }
                        },
                        on_request_delete: move |key: String| {
                            show_delete_dialog.set(Some(vec![DeleteTarget {
                                key,
                                is_folder: false,
                            }]));
                        },
                    }
                }
            }

            div {
                padding: "0 20px 16px",
                background: COLOR_BG,
                display: "flex",
                align_items: "center",
                justify_content: "space_between",
                gap: "12px",
                flex_wrap: "wrap",

                div {
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "12px",

                    if total_filtered == 0 {
                        "显示 0 / 0 个 Key"
                    } else {
                        "显示 {start_index + 1}-{end_index} / {total_filtered} 个 Key"
                    }
                }

                div {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",

                    button {
                        padding: "6px 10px",
                        background: COLOR_SURFACE_HIGHEST,
                        color: COLOR_TEXT_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        cursor: "pointer",
                        disabled: page == 0,
                        onclick: move |_| {
                            if page > 0 {
                                current_page.set(page - 1);
                            }
                        },

                        "上一页"
                    }

                    span {
                        color: COLOR_ACCENT,
                        font_size: "12px",
                        font_family: "Consolas, 'Courier New', monospace",

                        "第 {page + 1} / {total_pages} 页"
                    }

                    button {
                        padding: "6px 10px",
                        background: COLOR_SURFACE_HIGHEST,
                        color: COLOR_TEXT_SECONDARY,
                        border: "1px solid {COLOR_BORDER}",
                        border_radius: "6px",
                        cursor: "pointer",
                        disabled: page + 1 >= total_pages,
                        onclick: move |_| {
                            if page + 1 < total_pages {
                                current_page.set(page + 1);
                            }
                        },

                        "下一页"
                    }
                }
            }
        }

        if let Some(targets) = show_delete_dialog() {
            DeleteConfirmDialog {
                connection_pool: connection_pool.clone(),
                targets: targets.clone(),
                on_confirm: move |_| {
                    show_delete_dialog.set(None);
                    selected_keys.write().clear();
                    selected_key.set(String::new());
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                on_cancel: move |_| show_delete_dialog.set(None),
            }
        }

        if show_add_key_dialog() {
            AddKeyDialog {
                connection_pool: connection_pool.clone(),
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

        if let Some(keys) = show_batch_ttl_dialog() {
            BatchTtlDialog {
                connection_pool: connection_pool.clone(),
                keys: keys.clone(),
                on_confirm: move |_| {
                    show_batch_ttl_dialog.set(None);
                    selected_keys.write().clear();
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                on_cancel: move |_| show_batch_ttl_dialog.set(None),
            }
        }
    }
}
