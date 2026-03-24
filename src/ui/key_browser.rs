use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType, TreeBuilder, TreeNode};
use crate::theme::{
    ThemeColors, COLOR_ACCENT, COLOR_BG, COLOR_BG_LOWEST, COLOR_BG_SECONDARY, COLOR_BG_TERTIARY,
    COLOR_BORDER, COLOR_ERROR, COLOR_OUTLINE_VARIANT, COLOR_PRIMARY, COLOR_SURFACE_HIGHEST,
    COLOR_TEXT, COLOR_TEXT_CONTRAST, COLOR_TEXT_SECONDARY, COLOR_TEXT_SUBTLE,
};
use crate::ui::add_key_dialog::AddKeyDialog;
use crate::ui::batch_ttl_dialog::BatchTtlDialog;
use crate::ui::delete_confirm_dialog::{DeleteConfirmDialog, DeleteTarget};
use crate::ui::export_dialog::{ExportDialog, ExportTarget};
use crate::ui::icons::*;
use crate::ui::{
    copy_text_to_clipboard, LazyTreeNode, ResizableDivider, ToastManager, TreeState, ValueViewer,
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

fn collect_leaf_keys_by_node_id(nodes: &[TreeNode], node_id: &str) -> Vec<String> {
    fn find_node<'a>(nodes: &'a [TreeNode], node_id: &str) -> Option<&'a TreeNode> {
        for node in nodes {
            if node.node_id == node_id {
                return Some(node);
            }
            if let found @ Some(_) = find_node(&node.children, node_id) {
                return found;
            }
        }
        None
    }

    fn collect_leaves(node: &TreeNode) -> Vec<String> {
        let mut keys = Vec::new();
        if node.is_leaf {
            keys.push(node.path.clone());
        }
        for child in &node.children {
            keys.extend(collect_leaves(child));
        }
        keys
    }

    if let Some(node) = find_node(nodes, node_id) {
        collect_leaves(node)
    } else {
        Vec::new()
    }
}

fn key_match_pattern(search_pattern: &str) -> String {
    if search_pattern.trim().is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", search_pattern.trim())
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
    connection_id: Uuid,
    connection_pool: ConnectionPool,
    connection_version: u32,
    selected_key: Signal<String>,
    current_db: Signal<u8>,
    refresh_trigger: Signal<u32>,
    colors: ThemeColors,
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
    let mut tree_state = use_signal(TreeState::default);
    let mut context_menu = use_signal(|| None::<(String, bool, (i32, i32))>);
    let mut key_list_width = use_signal(|| 320.0);
    let mut show_export_dialog = use_signal(|| None::<Vec<ExportTarget>>);
    let mut toast_manager = use_context::<Signal<ToastManager>>();

    {
        let mut show_delete_dialog = show_delete_dialog.clone();
        let mut show_add_key_dialog = show_add_key_dialog.clone();
        let mut show_batch_ttl_dialog = show_batch_ttl_dialog.clone();
        let mut show_export_dialog = show_export_dialog.clone();

        use_future(move || {
            let mut show_delete_dialog = show_delete_dialog.clone();
            let mut show_add_key_dialog = show_add_key_dialog.clone();
            let mut show_batch_ttl_dialog = show_batch_ttl_dialog.clone();
            let mut show_export_dialog = show_export_dialog.clone();
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
        move || {
            let pool = pool.clone();
            let match_pattern = key_match_pattern(&search_pattern.read());
            let mut loading = loading.clone();
            let mut tree_nodes = tree_nodes.clone();
            let mut keys_count = keys_count.clone();
            let load_keyspace = load_keyspace.clone();
            let mut scan_progress = scan_progress.clone();
            let mut tree_state = tree_state.clone();
            let cancel_flag = Arc::new(AtomicBool::new(false));
            cancel_scan.set(cancel_flag.clone());
            let mut key_type_cache = key_type_cache.clone();

            spawn(async move {
                loading.set(true);
                let preserved_expanded = tree_state.read().expanded_nodes.clone();
                tree_nodes.set(Vec::new());
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
                let new_nodes = builder.build(all_keys);
                let new_node_ids = collect_all_node_ids(&new_nodes);
                tree_nodes.set(new_nodes);

                {
                    let mut state = tree_state.write();
                    let valid_expanded: HashSet<String> = preserved_expanded
                        .into_iter()
                        .filter(|id| new_node_ids.contains(id))
                        .collect();
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

    let select_db = {
        let pool = connection_pool.clone();
        let mut refresh_trigger = refresh_trigger.clone();
        let mut selected_key = selected_key.clone();
        move |db: u8| {
            let pool = pool.clone();
            spawn(async move {
                match pool.select_database(db).await {
                    Ok(_) => {
                        selected_key.set(String::new());
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
    let pattern_nodes = tree_nodes.read().clone();
    let active_match_pattern = key_match_pattern(&search_pattern());
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
                    padding: "14px 16px",
                    border_bottom: "1px solid {COLOR_BORDER}",
                    background: COLOR_BG_SECONDARY,

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "8px",
                        margin_bottom: "12px",

                        select {
                            width: "100px",
                            padding: "7px 10px",
                            background: COLOR_BG_TERTIARY,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            color: COLOR_TEXT,
                            font_size: "12px",
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
                            display: "flex",
                            align_items: "center",
                            gap: "6px",
                            padding: "0 10px",
                            height: "32px",
                            background: COLOR_BG_LOWEST,
                            border: "1px solid {COLOR_OUTLINE_VARIANT}",
                            border_radius: "6px",

                            IconSearch { size: Some(14) }

                            input {
                                flex: "1",
                                background: "transparent",
                                border: "none",
                                color: COLOR_TEXT,
                                font_size: "12px",
                                placeholder: "搜索 key",
                                value: "{search_pattern}",
                                oninput: move |e| search_pattern.set(e.value()),
                                onkeydown: move |e| {
                                    if e.data().key() == Key::Enter {
                                        refresh_trigger.set(refresh_trigger() + 1);
                                    }
                                },
                            }
                        }
                    }

                    div {
                        display: "flex",
                        align_items: "center",
                        gap: "6px",

                        button {
                            padding: "6px 10px",
                            background: COLOR_PRIMARY,
                            color: COLOR_TEXT_CONTRAST,
                            border: "none",
                            border_radius: "6px",
                            cursor: "pointer",
                            display: "flex",
                            align_items: "center",
                            gap: "6px",
                            font_size: "12px",
                            onclick: move |_| show_add_key_dialog.set(true),

                            IconPlus { size: Some(12) }
                            "新增"
                        }

                        button {
                            padding: "6px 10px",
                            background: COLOR_SURFACE_HIGHEST,
                            color: COLOR_TEXT,
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            cursor: "pointer",
                            display: "flex",
                            align_items: "center",
                            gap: "6px",
                            font_size: "12px",
                            onclick: move |_| refresh_trigger.set(refresh_trigger() + 1),

                            IconRefresh { size: Some(12) }
                            "刷新"
                        }

                        if scan_progress.read().is_scanning {
                            button {
                                padding: "6px 10px",
                                background: "#c53030",
                                color: COLOR_TEXT_CONTRAST,
                                border: "none",
                                border_radius: "6px",
                                cursor: "pointer",
                                display: "flex",
                                align_items: "center",
                                gap: "6px",
                                font_size: "12px",
                                onclick: {
                                    let cancel_scan = cancel_scan.clone();
                                    move |_| cancel_scan.read().store(true, Ordering::Relaxed)
                                },

                                IconX { size: Some(12) }
                                "取消"
                            }
                        }

                        div {
                            flex: "1",
                        }

                        label {
                            display: "flex",
                            align_items: "center",
                            gap: "4px",
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",
                            cursor: "pointer",

                            input {
                                r#type: "checkbox",
                                checked: current_selection_mode,
                                onchange: move |e| {
                                    tree_state.write().selection_mode = e.checked();
                                    if !e.checked() {
                                        tree_state.write().selected_keys.clear();
                                    }
                                },
                            }

                            "多选"
                        }
                    }

                    if scan_progress.read().is_scanning {
                        div {
                            margin_top: "10px",
                            color: COLOR_TEXT_SECONDARY,
                            font_size: "11px",

                            "已扫描 {scan_progress.read().scanned} 个 key"
                        }
                    }

                    if current_selection_mode {
                        div {
                            margin_top: "10px",
                            padding: "8px 10px",
                            background: "rgba(48, 209, 88, 0.10)",
                            border: "1px solid {COLOR_BORDER}",
                            border_radius: "6px",
                            display: "flex",
                            align_items: "center",
                            justify_content: "space_between",
                            gap: "8px",

                            span {
                                color: COLOR_ACCENT,
                                font_size: "11px",

                                "已选 {selected_count} 项"
                            }

                            div {
                                display: "flex",
                                align_items: "center",
                                gap: "4px",

                                button {
                                    padding: "4px 8px",
                                    background: "#38a169",
                                    color: COLOR_TEXT_CONTRAST,
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "10px",
                                    onclick: move |_| {
                                        let all_keys = collect_all_keys(&tree_nodes());
                                        tree_state.write().selected_keys.extend(all_keys);
                                    },

                                    "全选"
                                }

                                button {
                                    padding: "4px 8px",
                                    background: COLOR_SURFACE_HIGHEST,
                                    color: COLOR_TEXT,
                                    border: "1px solid {COLOR_BORDER}",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "10px",
                                    onclick: move |_| tree_state.write().selected_keys.clear(),

                                    "清空"
                                }

                                button {
                                    padding: "4px 8px",
                                    background: COLOR_PRIMARY,
                                    color: COLOR_TEXT_CONTRAST,
                                    border: "none",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "10px",
                                    disabled: selected_count == 0,
                                    onclick: {
                                        let keys: Vec<String> = tree_state.read().selected_keys.iter().cloned().collect();
                                        move |_| show_batch_ttl_dialog.set(Some(keys.clone()))
                                    },

                                    "TTL"
                                }

                                button {
                                    padding: "4px 8px",
                                    background: "rgba(255, 180, 171, 0.10)",
                                    color: "#ffb4ab",
                                    border: "1px solid rgba(255, 180, 171, 0.24)",
                                    border_radius: "4px",
                                    cursor: "pointer",
                                    font_size: "10px",
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

                                    "删除"
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
                                "正在加载..."
                            } else {
                                "没有找到 key"
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

                    "共 {keys_count()} 个 Key"
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
                background: COLOR_BG,

                if selected_key.read().is_empty() {
                    div {
                        flex: "1",
                        display: "flex",
                        flex_direction: "column",
                        align_items: "center",
                        justify_content: "center",
                        gap: "12px",
                        color: COLOR_TEXT_SECONDARY,

                        IconDatabase { size: Some(48) }

                        div {
                            font_size: "16px",
                            color: COLOR_TEXT,

                            "选择一个 Key 查看内容"
                        }

                        div {
                            font_size: "13px",

                            "从左侧树形列表中选择 key，此处将显示其值"
                        }
                    }
                } else {
                    div {
                        flex: "1",
                        min_height: "0",
                        display: "flex",
                        flex_direction: "column",
                        overflow: "hidden",

                        ValueViewer {
                            key: "{connection_id}",
                            connection_pool: connection_pool.clone(),
                            selected_key: selected_key,
                            on_refresh: move |_| {
                                refresh_trigger.set(refresh_trigger() + 1);
                            },
                        }
                    }
                }
            }
        }

        if let Some((node_path, is_leaf, (x, y))) = context_menu() {
            div {
                position: "fixed",
                left: "{x}px",
                top: "{y}px",
                background: COLOR_BG_SECONDARY,
                border: "1px solid {COLOR_BORDER}",
                border_radius: "6px",
                box_shadow: "0 4px 12px rgba(0, 0, 0, 0.4)",
                z_index: "1000",
                min_width: "120px",
                padding: "4px 0",

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: COLOR_TEXT,
                    font_size: "12px",
                    onmouseenter: |_| {},
                    onmouseleave: |_| {},

                    onclick: {
                        let node_path = node_path.clone();
                        move |_| {
                            context_menu.set(None);
                            if copy_text_to_clipboard(&node_path).is_ok() {
                                toast_manager.write().success("路径已复制");
                            }
                        }
                    },

                    "复制路径"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: COLOR_TEXT,
                    font_size: "12px",
                    onmouseenter: |_| {},
                    onmouseleave: |_| {},

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

                    "导出"
                }

                div {
                    padding: "8px 12px",
                    cursor: "pointer",
                    color: COLOR_ERROR,
                    font_size: "12px",

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

                    "删除"
                }
            }

            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                z_index: "999",

                onclick: move |_| context_menu.set(None),
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

        if let Some(keys) = show_batch_ttl_dialog() {
            BatchTtlDialog {
                connection_pool: connection_pool.clone(),
                keys: keys.clone(),
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

fn update_node_key_type(nodes: &mut [TreeNode], key_path: &str, key_type: KeyType) -> bool {
    for node in nodes.iter_mut() {
        if node.is_leaf && node.path == key_path {
            if let Some(ref mut info) = node.key_info {
                info.key_type = key_type;
            } else {
                node.key_info = Some(KeyInfo {
                    name: key_path.to_string(),
                    key_type,
                    ttl: None,
                    size: None,
                });
            }
            return true;
        }
        if update_node_key_type(&mut node.children, key_path, key_type.clone()) {
            return true;
        }
    }
    false
}
