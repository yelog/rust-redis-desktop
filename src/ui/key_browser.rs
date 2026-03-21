use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType, TreeBuilder, TreeNode};
use crate::theme::{COLOR_ACCENT, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY, COLOR_BORDER, COLOR_PRIMARY, COLOR_TEXT, COLOR_TEXT_SECONDARY, COLOR_TEXT_CONTRAST};
use crate::ui::add_key_dialog::AddKeyDialog;
use crate::ui::batch_ttl_dialog::BatchTtlDialog;
use crate::ui::context_menu::{ContextMenu, ContextMenuItem};
use crate::ui::delete_confirm_dialog::{DeleteConfirmDialog, DeleteTarget};
use crate::ui::icons::*;
use crate::ui::lazy_tree_node::{ContextMenuState, LazyTreeNode, TreeState};
use arboard::Clipboard;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

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

fn collect_all_folder_ids(nodes: &[TreeNode]) -> Vec<String> {
    let mut ids = Vec::new();
    for node in nodes {
        if !node.is_leaf {
            ids.push(node.node_id.clone());
            ids.extend(collect_all_folder_ids(&node.children));
        }
    }
    ids
}

fn collect_leaf_keys_in_node(nodes: &[TreeNode], node_id: &str) -> Vec<String> {
    for node in nodes {
        if node.node_id == node_id {
            return collect_all_keys(&node.children);
        }
        let keys = collect_leaf_keys_in_node(&node.children, node_id);
        if !keys.is_empty() {
            return keys;
        }
    }
    Vec::new()
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

#[derive(Clone, Default)]
pub struct ScanProgress {
    pub scanned: usize,
    pub current_batch: usize,
    pub is_scanning: bool,
}

#[component]
pub fn KeyBrowser(
    width: f64,
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
    let mut tree_state = use_signal(TreeState::default);
    let mut context_menu = use_signal(|| None::<ContextMenuState>);
    let mut show_delete_dialog = use_signal(|| None::<Vec<DeleteTarget>>);
    let mut show_add_key_dialog = use_signal(|| false);
    let db_keys_count = use_signal(HashMap::<u8, u64>::new);
    let mut show_batch_ttl_dialog = use_signal(|| None::<Vec<String>>);
    let mut scan_progress = use_signal(ScanProgress::default);
    let mut cancel_scan = use_signal(|| Arc::new(AtomicBool::new(false)));
    let key_type_cache = use_signal(HashMap::<String, KeyType>::new);

    let selection_mode = tree_state.read().selection_mode;
    let selected_keys_count = tree_state.read().selected_keys.len();
    let selected_keys: Vec<String> = tree_state.read().selected_keys.iter().cloned().collect();

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
        let tree_state = tree_state.clone();
        let scan_progress = scan_progress.clone();
        let mut cancel_scan = cancel_scan.clone();
        move || {
            let pool = pool.clone();
            let pattern = if search_pattern.read().is_empty() {
                "*".to_string()
            } else {
                format!("*{}*", search_pattern.read())
            };
            let mut loading = loading.clone();
            let mut tree_nodes = tree_nodes.clone();
            let mut keys_count = keys_count.clone();
            let load_keyspace = load_keyspace.clone();
            let mut tree_state = tree_state.clone();
            let mut scan_progress = scan_progress.clone();
            let cancel_flag = Arc::new(AtomicBool::new(false));
            cancel_scan.set(cancel_flag.clone());

            spawn(async move {
                loading.set(true);
                tree_nodes.set(Vec::new());
                tree_state.write().selected_keys.clear();
                scan_progress.write().is_scanning = true;
                scan_progress.write().scanned = 0;
                scan_progress.write().current_batch = 0;

                let mut all_keys = Vec::new();
                let mut cursor: u64 = 0;
                let batch_size = 500usize;

                loop {
                    if cancel_flag.load(Ordering::Relaxed) {
                        tracing::info!("Scan cancelled by user");
                        break;
                    }

                    match pool
                        .scan_keys_with_cursor(&pattern, cursor, batch_size)
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
                let tree = builder.build(all_keys);
                
                let search_is_active = !search_pattern.read().is_empty();
                if search_is_active {
                    let folder_ids = collect_all_folder_ids(&tree);
                    let mut state = tree_state.write();
                    state.expanded_nodes.clear();
                    for id in folder_ids {
                        state.expanded_nodes.insert(id);
                    }
                } else {
                    tree_state.write().expanded_nodes.clear();
                }
                
                tree_nodes.set(tree);

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
            load_keys();
        }
    });

    rsx! {
        div {
            width: "{width}px",
            height: "100%",
            background: COLOR_BG_SECONDARY,
            border_right: "1px solid {COLOR_BORDER}",
            display: "flex",
            flex_direction: "column",
            box_sizing: "border-box",

            div {
                padding: "8px",
                border_bottom: "1px solid {COLOR_BORDER}",

                select {
                    width: "100%",
                    padding: "6px 10px",
                    background: COLOR_BG_TERTIARY,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "4px",
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
            }

            div {
                padding: "8px",
                border_bottom: "1px solid {COLOR_BORDER}",

                input {
                    width: "100%",
                    padding: "6px 10px",
                    background: COLOR_BG_TERTIARY,
                    border: "1px solid {COLOR_BORDER}",
                    border_radius: "4px",
                    color: COLOR_TEXT,
                    font_size: "13px",
                    placeholder: "Search keys...",
                    value: "{search_pattern}",
                    oninput: move |e| search_pattern.set(e.value()),
                    onkeydown: move |e| {
                        if e.data().key() == Key::Enter {
                            refresh_trigger.set(refresh_trigger() + 1);
                        }
                    },
                }
            }

            div {
                padding: "8px",
                border_bottom: "1px solid {COLOR_BORDER}",
                display: "flex",
                gap: "8px",
                align_items: "center",
                flex_wrap: "wrap",

                button {
                    padding: "6px 12px",
                    background: COLOR_PRIMARY,
                    color: COLOR_TEXT_CONTRAST,
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "12px",
                    onclick: move |_| show_add_key_dialog.set(true),

                    "➕ 新增"
                }

                if scan_progress.read().is_scanning {
                    button {
                        flex: "1",
                        padding: "6px",
                        background: "#c53030",
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: {
                            let cancel_scan = cancel_scan.clone();
                            move |_| {
                                cancel_scan.read().store(true, Ordering::Relaxed);
                            }
                        },

                        div {
                        display: "flex",
                        align_items: "center",
                        gap: "4px",
                        
                        IconX { size: Some(12) }
                        " 取消扫描"
                    }
                    }
                } else {
                    button {
                        flex: "1",
                        padding: "6px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "12px",
                        onclick: move |_| {
                            refresh_trigger.set(refresh_trigger() + 1);
                        },

                        div {
                        display: "flex",
                        align_items: "center",
                        gap: "4px",
                        
                        IconRefresh { size: Some(12) }
                        " Refresh"
                    }
                    }
                }

                if keys_count() > 0 {
                    span {
                        color: COLOR_TEXT_SECONDARY,
                        font_size: "11px",

                        "{keys_count} keys"
                    }
                }
            }

            if scan_progress.read().is_scanning {
                div {
                    padding: "8px",
                    border_bottom: "1px solid {COLOR_BORDER}",
                    background: COLOR_BG_TERTIARY,

                    div {
                        display: "flex",
                        justify_content: "space_between",
                        align_items: "center",
                        margin_bottom: "4px",

                        span {
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",

                            "正在扫描..."
                        }

                        span {
                            color: COLOR_ACCENT,
                            font_size: "11px",

                            "已找到 {scan_progress.read().scanned} 个 key"
                        }
                    }

                    div {
                        width: "100%",
                        height: "4px",
                        background: COLOR_BORDER,
                        border_radius: "2px",
                        overflow: "hidden",

                        div {
                            width: "100%",
                            height: "100%",
                            background: "linear-gradient(90deg, {COLOR_ACCENT}, {COLOR_PRIMARY})",
                            animation: "pulse 1.5s ease-in-out infinite",
                        }
                    }
                }

                style { {r#"
                    @keyframes pulse {
                        0%, 100% { opacity: 0.4; transform: scaleX(0.3); }
                        50% { opacity: 1; transform: scaleX(1); }
                    }
                "#} }
            }

            if selection_mode {
                div {
                    padding: "8px",
                    border_bottom: "1px solid {COLOR_BORDER}",
                    background: "rgba(48, 209, 88, 0.15)",
                    display: "flex",
                    gap: "8px",
                    align_items: "center",
                    flex_wrap: "wrap",

                    span {
                        color: COLOR_ACCENT,
                        font_size: "12px",

                        "已选 {selected_keys_count} 项"
                    }

                    button {
                        padding: "4px 8px",
                        background: "#38a169",
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "11px",
                        onclick: move |_| {
                            let all_keys = collect_all_keys(&tree_nodes());
                            tree_state.write().selected_keys = all_keys.into_iter().collect();
                        },

                        "全选"
                    }

                    button {
                        padding: "4px 8px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "11px",
                        onclick: move |_| {
                            tree_state.write().selected_keys.clear();
                        },

                        "清空"
                    }

                    button {
                        padding: "4px 8px",
                        background: "#c53030",
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "11px",
                        disabled: selected_keys_count == 0,
                        onclick: {
                            let selected_keys = selected_keys.clone();
                            move |_| {
                                let targets: Vec<DeleteTarget> = selected_keys.iter()
                                    .map(|k| DeleteTarget { key: k.clone(), is_folder: false })
                                    .collect();
                                show_delete_dialog.set(Some(targets));
                            }
                        },

                        div {
                            display: "flex",
                            align_items: "center",
                            gap: "4px",
                            
                            IconTrash { size: Some(12) }
                            " 删除"
                        }
                    }

                    button {
                        padding: "4px 8px",
                        background: COLOR_PRIMARY,
                        color: COLOR_TEXT_CONTRAST,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "11px",
                        disabled: selected_keys_count == 0,
                        onclick: {
                            let selected_keys = selected_keys.clone();
                            move |_| {
                                show_batch_ttl_dialog.set(Some(selected_keys.clone()));
                            }
                        },

                        div {
                            display: "flex",
                            align_items: "center",
                            gap: "4px",
                            
                            IconRefresh { size: Some(12) }
                            " TTL"
                        }
                    }

                    button {
                        padding: "4px 8px",
                        background: COLOR_BG_TERTIARY,
                        color: COLOR_TEXT,
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        font_size: "11px",
                        onclick: move |_| {
                            tree_state.write().selection_mode = false;
                            tree_state.write().selected_keys.clear();
                        },

                        div {
                            display: "flex",
                            align_items: "center",
                            gap: "4px",
                            
                            IconX { size: Some(12) }
                            " 关闭"
                        }
                    }
                }
            }

            div {
                padding: "4px 8px",
                border_bottom: "1px solid {COLOR_BORDER}",
                display: "flex",
                justify_content: "flex_end",

                label {
                    display: "flex",
                    align_items: "center",
                    gap: "6px",
                    color: COLOR_TEXT_SECONDARY,
                    font_size: "11px",
                    cursor: "pointer",

                    input {
                        r#type: "checkbox",
                        checked: selection_mode,
                        onchange: move |e| {
                            tree_state.write().selection_mode = e.checked();
                            if !e.checked() {
                                tree_state.write().selected_keys.clear();
                            }
                        },
                    }

                    "多选模式"
                }
            }

            div {
                flex: "1",
                overflow_y: "auto",
                padding: "4px 0",

                if tree_nodes.read().is_empty() {
                    if loading() {
                        div {
                            padding: "20px",
                            text_align: "center",
                            color: COLOR_TEXT_SECONDARY,

                            "Loading keys..."
                        }
                    } else {
                        div {
                            padding: "20px",
                            text_align: "center",
                            color: COLOR_TEXT_SECONDARY,

                            "No keys found"
                        }
                    }
                } else {
                    for node in tree_nodes.read().iter() {
                        LazyTreeNode {
                            key: "{node.node_id}",
                            node: node.clone(),
                            depth: 0,
                            selected_key: selected_key(),
                            tree_state: tree_state,
                            on_select: {
                                let pool = connection_pool.clone();
                                let key_type_cache = key_type_cache.clone();
                                let tree_nodes = tree_nodes.clone();
                                let on_key_select = on_key_select.clone();
                                move |key: String| {
                                    let key_clone = key.clone();
                                    let needs_fetch = {
                                        let cache = key_type_cache.read();
                                        !cache.contains_key(&key_clone)
                                    };

                                    if needs_fetch {
                                        let pool = pool.clone();
                                        let mut key_type_cache = key_type_cache.clone();
                                        let mut tree_nodes = tree_nodes.clone();
                                        let key = key.clone();
                                        spawn(async move {
                                            match pool.get_key_info(&key).await {
                                                Ok(info) => {
                                                    key_type_cache.write().insert(key.clone(), info.key_type.clone());
                                                    let updated = {
                                                        let mut nodes = tree_nodes.write();
                                                        update_node_key_info(&mut nodes, &key, info)
                                                    };
                                                    if updated {
                                                        let nodes_clone = tree_nodes.read().clone();
                                                        tree_nodes.set(nodes_clone);
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::error!("Failed to get key type: {}", e);
                                                }
                                            }
                                        });
                                    }

                                    on_key_select.call(key_clone);
                                }
                            },
                            on_expand: {
                                let pool = connection_pool.clone();
                                let key_type_cache = key_type_cache.clone();
                                let tree_nodes = tree_nodes.clone();
                                move |node_id: String| {
                                    let is_expanding = {
                                        let state = tree_state.read();
                                        !state.expanded_nodes.contains(&node_id)
                                    };

                                    {
                                        let mut state = tree_state.write();
                                        if state.expanded_nodes.contains(&node_id) {
                                            state.expanded_nodes.remove(&node_id);
                                        } else {
                                            state.expanded_nodes.insert(node_id.clone());
                                        }
                                    }

                                    if is_expanding {
                                        let leaf_keys = {
                                            let nodes = tree_nodes.read();
                                            collect_leaf_keys_in_node(&nodes, &node_id)
                                        };

                                        let keys_to_fetch: Vec<String> = leaf_keys
                                            .into_iter()
                                            .filter(|k| !key_type_cache.read().contains_key(k))
                                            .collect();

                                        if !keys_to_fetch.is_empty() {
                                            let pool = pool.clone();
                                            let mut key_type_cache = key_type_cache.clone();
                                            let mut tree_nodes = tree_nodes.clone();
                                            spawn(async move {
                                                let mut updates = Vec::new();
                                                for key in keys_to_fetch {
                                                    match pool.get_key_info(&key).await {
                                                        Ok(info) => {
                                                            key_type_cache.write().insert(key.clone(), info.key_type.clone());
                                                            updates.push((key, info));
                                                        }
                                                        Err(e) => {
                                                            tracing::error!("Failed to get key type: {}", e);
                                                        }
                                                    }
                                                }

                                                if !updates.is_empty() {
                                                    let mut nodes = tree_nodes.write();
                                                    update_multiple_key_info(&mut nodes, &updates);
                                                    drop(nodes);
                                                    let nodes_clone = tree_nodes.read().clone();
                                                    tree_nodes.set(nodes_clone);
                                                }
                                            });
                                        }
                                    }
                                }
                            },
                            context_menu: context_menu,
                        }
                    }
                }
            }
        }

        if let Some(menu_state) = context_menu() {
            ContextMenu {
                x: menu_state.x,
                y: menu_state.y,
                on_close: move |_| context_menu.set(None),

                ContextMenuItem {
                    icon: Some(rsx! { IconCopy { size: Some(14) } }),
                    label: "复制Key".to_string(),
                    danger: false,
                    onclick: {
                        let menu_state = menu_state.clone();
                        move |_| {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(menu_state.node_path.clone());
                            }
                            context_menu.set(None);
                        }
                    },
                }

                ContextMenuItem {
                    icon: Some(rsx! { IconCheck { size: Some(14) } }),
                    label: "多选模式".to_string(),
                    danger: false,
                    onclick: move |_| {
                        tree_state.write().selection_mode = true;
                        context_menu.set(None);
                    },
                }

                ContextMenuItem {
                    icon: Some(rsx! { IconTrash { size: Some(14) } }),
                    label: "删除".to_string(),
                    danger: true,
                    onclick: {
                        let menu_state = menu_state.clone();
                        move |_| {
                            context_menu.set(None);
                            show_delete_dialog.set(Some(vec![DeleteTarget {
                                key: menu_state.node_path.clone(),
                                is_folder: !menu_state.is_leaf,
                            }]));
                        }
                    },
                }
            }
        }

        if let Some(targets) = show_delete_dialog() {
            DeleteConfirmDialog {
                connection_pool: connection_pool.clone(),
                targets: targets.clone(),
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
                    tree_state.write().selected_keys.clear();
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                on_cancel: move |_| show_batch_ttl_dialog.set(None),
            }
        }
    }
}
