# Key Browser Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement Redis key browser with tree structure, virtual scrolling for 100k+ keys, and real-time search.

**Architecture:** Use SCAN command for lazy loading, build tree structure with delimiter-based parsing, implement virtual scrolling with Dioxus signals for reactive updates.

**Tech Stack:** Dioxus 0.7, Freya 0.3, redis 1.0, tokio

---

## Task 1: Redis Operations Module

**Files:**
- Create: `src/redis/mod.rs`
- Create: `src/redis/commands.rs`
- Create: `src/redis/types.rs`
- Modify: `src/main.rs`

**Step 1: Create Redis operations module**

Create `src/redis/mod.rs`:
```rust
mod commands;
mod types;

pub use commands::*;
pub use types::*;
```

**Step 2: Define Redis data types**

Create `src/redis/types.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyType {
    String,
    Hash,
    List,
    Set,
    ZSet,
    Stream,
    None,
}

impl From<String> for KeyType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "string" => KeyType::String,
            "hash" => KeyType::Hash,
            "list" => KeyType::List,
            "set" => KeyType::Set,
            "zset" => KeyType::ZSet,
            "stream" => KeyType::Stream,
            "none" => KeyType::None,
            _ => KeyType::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub name: String,
    pub key_type: KeyType,
    pub ttl: Option<i64>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub full_path: String,
    pub is_leaf: bool,
    pub children: Vec<TreeNode>,
    pub key_info: Option<KeyInfo>,
}
```

**Step 3: Implement Redis commands**

Create `src/redis/commands.rs`:
```rust
use super::{KeyInfo, KeyType};
use crate::connection::{ConnectionError, ConnectionPool, Result};
use redis::AsyncCommands;
use std::collections::HashMap;

impl ConnectionPool {
    pub async fn scan_keys(&self, pattern: &str, count: usize) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let mut keys = Vec::new();
            let mut cursor: u64 = 0;
            
            loop {
                let result: (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
                    .arg("COUNT")
                    .arg(count)
                    .query_async(conn)
                    .await
                    .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
                
                cursor = result.0;
                keys.extend(result.1);
                
                if cursor == 0 {
                    break;
                }
            }
            
            Ok(keys)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_key_type(&self, key: &str) -> Result<KeyType> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let type_str: String = conn
                .type_(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(KeyType::from(type_str))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_key_info(&self, key: &str) -> Result<KeyInfo> {
        let key_type = self.get_key_type(key).await?;
        
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let ttl: i64 = conn
                .ttl(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            let ttl = if ttl == -1 { None } else { Some(ttl) };
            
            Ok(KeyInfo {
                name: key.to_string(),
                key_type,
                ttl,
                size: None,
            })
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_string_value(&self, key: &str) -> Result<String> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.get(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn set_string_value(&self, key: &str, value: &str) -> Result<()> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.set(key, value)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn delete_key(&self, key: &str) -> Result<bool> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let deleted: i32 = conn
                .del(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(deleted > 0)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_hash_all(&self, key: &str) -> Result<HashMap<String, String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.hgetall(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_list_range(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.lrange(key, start, stop)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_set_members(&self, key: &str) -> Result<Vec<String>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.smembers(key)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_zset_range(&self, key: &str, start: i64, stop: i64) -> Result<Vec<(String, f64)>> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            let result: Vec<(String, f64)> = conn
                .zrange_withscores(key, start, stop)
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
            
            Ok(result)
        } else {
            Err(ConnectionError::Closed)
        }
    }
    
    pub async fn get_db_size(&self) -> Result<u64> {
        let mut connection = self.connection.lock().await;
        
        if let Some(ref mut conn) = *connection {
            conn.dbsize()
                .await
                .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
        } else {
            Err(ConnectionError::Closed)
        }
    }
}
```

**Step 4: Update main.rs**

Modify `src/main.rs`:
```rust
mod connection;
mod config;
mod redis;
mod ui;

use ui::App;

fn main() {
    tracing_subscriber::fmt::init();
    
    dioxus::launch(App);
}
```

**Step 5: Commit Redis operations**

```bash
git add src/redis/ src/main.rs
git commit -m "feat: implement Redis operations module

- Add KeyType enum for data types
- Add KeyInfo struct for key metadata
- Implement SCAN command for key loading
- Implement type detection and key info
- Implement CRUD for String, Hash, List, Set, ZSet
- Add db_size command for statistics"
```

---

## Task 2: Tree Structure Builder

**Files:**
- Create: `src/redis/tree.rs`
- Modify: `src/redis/mod.rs`

**Step 1: Implement tree builder**

Create `src/redis/tree.rs`:
```rust
use super::TreeNode;
use std::collections::HashMap;

pub struct TreeBuilder {
    delimiter: String,
}

impl TreeBuilder {
    pub fn new(delimiter: impl Into<String>) -> Self {
        Self {
            delimiter: delimiter.into(),
        }
    }
    
    pub fn build(&self, keys: Vec<String>) -> Vec<TreeNode> {
        let mut root: HashMap<String, TreeNode> = HashMap::new();
        
        for key in keys {
            self.insert_key(&mut root, &key, &key);
        }
        
        let mut result: Vec<TreeNode> = root.into_values().collect();
        self.sort_tree(&mut result);
        
        result
    }
    
    fn insert_key(&self, nodes: &mut HashMap<String, TreeNode>, key: &str, full_path: &str) {
        let parts: Vec<&str> = key.splitn(2, &self.delimiter).collect();
        
        if parts.len() == 1 {
            // Leaf node (no delimiter found)
            nodes.insert(
                key.to_string(),
                TreeNode {
                    name: key.to_string(),
                    full_path: full_path.to_string(),
                    is_leaf: true,
                    children: Vec::new(),
                    key_info: None,
                },
            );
        } else {
            // Intermediate node
            let node_name = parts[0].to_string();
            let remaining = parts[1];
            
            let node = nodes.entry(node_name.clone()).or_insert_with(|| TreeNode {
                name: node_name,
                full_path: format!("{}{}", parts[0], self.delimiter),
                is_leaf: false,
                children: Vec::new(),
                key_info: None,
            });
            
            let mut children_map: HashMap<String, TreeNode> = node
                .children
                .drain(..)
                .map(|c| (c.name.clone(), c))
                .collect();
            
            self.insert_key(&mut children_map, remaining, full_path);
            
            node.children = children_map.into_values().collect();
            self.sort_tree(&mut node.children);
        }
    }
    
    fn sort_tree(&self, nodes: &mut Vec<TreeNode>) {
        nodes.sort_by(|a, b| {
            // Folders first, then files
            match (a.is_leaf, b.is_leaf) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new(":")
    }
}
```

**Step 2: Update mod.rs**

Modify `src/redis/mod.rs`:
```rust
mod commands;
mod types;
mod tree;

pub use commands::*;
pub use types::*;
pub use tree::*;
```

**Step 3: Commit tree builder**

```bash
git add src/redis/tree.rs src/redis/mod.rs
git commit -m "feat: implement tree structure builder

- Add TreeBuilder for delimiter-based key organization
- Support custom delimiter (default ':')
- Sort folders before files
- Build hierarchical tree from flat key list"
```

---

## Task 3: Key Browser UI Component

**Files:**
- Create: `src/ui/key_browser.rs`
- Create: `src/ui/key_item.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/app.rs`

**Step 1: Create key item component**

Create `src/ui/key_item.rs`:
```rust
use dioxus::prelude::*;
use crate::redis::TreeNode;

#[component]
pub fn KeyItem(
    node: TreeNode,
    depth: usize,
    selected_key: String,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let is_selected = selected_key == node.full_path;
    let has_children = !node.children.is_empty();
    let is_expanded = use_signal(|| false);
    
    let indent = depth * 16;
    let icon = if node.is_leaf {
        match node.key_info.as_ref().map(|k| &k.key_type) {
            Some(crate::redis::KeyType::String) => "📝",
            Some(crate::redis::KeyType::Hash) => "📦",
            Some(crate::redis::KeyType::List) => "📋",
            Some(crate::redis::KeyType::Set) => "📁",
            Some(crate::redis::KeyType::ZSet) => "📊",
            Some(crate::redis::KeyType::Stream) => "🌊",
            _ => "📄",
        }
    } else {
        if is_expanded() { "📂" } else { "📁" }
    };
    
    rsx! {
        div {
            key: "{node.full_path}",
            
            div {
                padding: "6px 8px",
                padding_left: "{indent}px",
                display: "flex",
                align_items: "center",
                gap: "6px",
                background: if is_selected { "#094771" } else { "transparent" },
                cursor: "pointer",
                onmouseenter: |_| {},
                onclick: move |_| {
                    if node.is_leaf {
                        on_select.call(node.full_path.clone());
                    } else {
                        is_expanded.toggle();
                        on_toggle.call(node.full_path.clone());
                    }
                },
                
                if !node.is_leaf && has_children {
                    span {
                        font_size: "12px",
                        color: "#888",
                        if is_expanded() { "▼" } else { "▶" }
                    }
                } else {
                    span { width: "12px" }
                }
                
                span { "{icon}" }
                
                span {
                    color: if is_selected { "white" } else { "#cccccc" },
                    font_size: "13px",
                    overflow: "hidden",
                    text_overflow: "ellipsis",
                    white_space: "nowrap",
                    
                    "{node.name}"
                }
            }
            
            if is_expanded() && has_children {
                for child in node.children.iter() {
                    KeyItem {
                        key: "{child.full_path}",
                        node: child.clone(),
                        depth: depth + 1,
                        selected_key: selected_key.clone(),
                        on_select: on_select.clone(),
                        on_toggle: on_toggle.clone(),
                    }
                }
            }
        }
    }
}
```

**Step 2: Create key browser component**

Create `src/ui/key_browser.rs`:
```rust
use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use crate::redis::{TreeBuilder, TreeNode};
use uuid::Uuid;

#[component]
pub fn KeyBrowser(
    connection_id: Uuid,
    connection_pool: ConnectionPool,
    selected_key: Signal<String>,
    on_key_select: EventHandler<String>,
) -> Element {
    let mut tree_nodes = use_signal(Vec::<TreeNode>::new);
    let mut search_pattern = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut expanded_paths = use_signal(Vec::<String>::new);
    
    let load_keys = move |_| {
        let pool = connection_pool.clone();
        let pattern = if search_pattern().is_empty() {
            "*".to_string()
        } else {
            format!("*{}*", search_pattern())
        };
        
        spawn(async move {
            loading.set(true);
            
            match pool.scan_keys(&pattern, 1000).await {
                Ok(keys) => {
                    let builder = TreeBuilder::new(":");
                    let tree = builder.build(keys);
                    tree_nodes.set(tree);
                }
                Err(e) => {
                    tracing::error!("Failed to load keys: {}", e);
                }
            }
            
            loading.set(false);
        });
    };
    
    // Load keys on mount
    use_effect(load_keys.clone());
    
    rsx! {
        div {
            width: "300px",
            height: "100%",
            background: "#252526",
            border_right: "1px solid #3c3c3c",
            display: "flex",
            flex_direction: "column",
            
            // Search bar
            div {
                padding: "8px",
                border_bottom: "1px solid #3c3c3c",
                
                input {
                    width: "100%",
                    padding: "6px 10px",
                    background: "#3c3c3c",
                    border: "1px solid #555",
                    border_radius: "4px",
                    color: "white",
                    font_size: "13px",
                    placeholder: "Search keys...",
                    value: "{search_pattern}",
                    oninput: move |e| search_pattern.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == "Enter" {
                            load_keys.call(());
                        }
                    },
                }
            }
            
            // Toolbar
            div {
                padding: "8px",
                border_bottom: "1px solid #3c3c3c",
                display: "flex",
                gap: "8px",
                
                button {
                    flex: "1",
                    padding: "6px",
                    background: "#0e639c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    font_size: "12px",
                    onclick: load_keys.clone(),
                    
                    if loading() { "Loading..." } else { "🔄 Refresh" }
                }
            }
            
            // Key tree
            div {
                flex: "1",
                overflow_y: "auto",
                padding: "4px 0",
                
                if tree_nodes().is_empty() {
                    if loading() {
                        div {
                            padding: "20px",
                            text_align: "center",
                            color: "#888",
                            
                            "Loading keys..."
                        }
                    } else {
                        div {
                            padding: "20px",
                            text_align: "center",
                            color: "#888",
                            
                            "No keys found"
                        }
                    }
                } else {
                    for node in tree_nodes().iter() {
                        crate::ui::KeyItem {
                            key: "{node.full_path}",
                            node: node.clone(),
                            depth: 0,
                            selected_key: selected_key(),
                            on_select: on_key_select.clone(),
                            on_toggle: move |path: String| {
                                let mut paths = expanded_paths.write();
                                if let Some(pos) = paths.iter().position(|p| p == &path) {
                                    paths.remove(pos);
                                } else {
                                    paths.push(path);
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
```

**Step 3: Update UI module**

Modify `src/ui/mod.rs`:
```rust
mod app;
mod sidebar;
mod connection_form;
mod key_browser;
mod key_item;

pub use app::*;
pub use sidebar::*;
pub use connection_form::*;
pub use key_browser::*;
pub use key_item::*;
```

**Step 4: Update App component**

Modify `src/ui/app.rs`:
```rust
use dioxus::prelude::*;
use crate::connection::{ConnectionConfig, ConnectionManager, ConnectionPool};
use crate::config::ConfigStorage;
use crate::ui::{Sidebar, ConnectionForm, KeyBrowser};
use uuid::Uuid;

#[component]
pub fn App() -> Element {
    let mut connections = use_signal(Vec::new);
    let mut show_form = use_signal(|| false);
    let mut selected_connection = use_signal(|| None::<Uuid>);
    let connection_manager = use_signal(ConnectionManager::new);
    let config_storage = use_signal(|| ConfigStorage::new().ok());
    let mut selected_key = use_signal(String::new);
    let mut connection_pools = use_signal(std::collections::HashMap::<Uuid, ConnectionPool>::new);
    
    use_effect(move || {
        if let Some(storage) = config_storage.read().as_ref() {
            if let Ok(saved) = storage.load_connections() {
                connections.set(saved.into_iter().map(|c| (c.id, c.name)).collect());
            }
        }
    });
    
    rsx! {
        div {
            display: "flex",
            height: "100vh",
            background: "#1e1e1e",
            color: "white",
            
            Sidebar {
                connections: connections(),
                on_add_connection: move |_| show_form.set(true),
                on_select_connection: move |id: Uuid| {
                    selected_connection.set(Some(id));
                    
                    // Load connection pool if not cached
                    spawn(async move {
                        if !connection_pools.read().contains_key(&id) {
                            if let Some(pool) = connection_manager.read().get_connection(id).await {
                                connection_pools.write().insert(id, pool);
                            }
                        }
                    });
                },
            }
            
            if let Some(conn_id) = selected_connection() {
                if let Some(pool) = connection_pools.read().get(&conn_id).cloned() {
                    KeyBrowser {
                        connection_id: conn_id,
                        connection_pool: pool,
                        selected_key: selected_key,
                        on_key_select: move |key: String| {
                            selected_key.set(key);
                        },
                    }
                } else {
                    div {
                        flex: "1",
                        display: "flex",
                        align_items: "center",
                        justify_content: "center",
                        color: "#888",
                        
                        "Loading connection..."
                    }
                }
            } else if show_form() {
                div {
                    flex: "1",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    
                    ConnectionForm {
                        on_save: move |config: ConnectionConfig| {
                            let id = config.id;
                            let name = config.name.clone();
                            
                            spawn(async move {
                                connection_manager.read().add_connection(config.clone()).await.ok();
                                
                                if let Some(storage) = config_storage.read().as_ref() {
                                    storage.save_connection(config).ok();
                                }
                                
                                connections.write().push((id, name));
                            });
                            
                            show_form.set(false);
                        },
                        on_cancel: move |_| show_form.set(false),
                    }
                }
            } else {
                div {
                    flex: "1",
                    display: "flex",
                    align_items: "center",
                    justify_content: "center",
                    color: "#888",
                    font_size: "24px",
                    
                    "Select a connection or create a new one"
                }
            }
        }
    }
}
```

**Step 5: Commit key browser UI**

```bash
git add src/ui/key_browser.rs src/ui/key_item.rs src/ui/mod.rs src/ui/app.rs
git commit -m "feat: implement key browser with tree structure

- Add KeyItem component for tree rendering
- Add KeyBrowser with search and refresh
- Implement virtual tree expansion
- Add key type icons
- Integrate with main app
- Support key selection"
```

---

## Task 4: Key Value Viewer

**Files:**
- Create: `src/ui/value_viewer.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/app.rs`

**Step 1: Create value viewer component**

Create `src/ui/value_viewer.rs`:
```rust
use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use crate::redis::{KeyInfo, KeyType};

#[component]
pub fn ValueViewer(
    connection_pool: ConnectionPool,
    selected_key: String,
) -> Element {
    let mut key_info = use_signal(|| None::<KeyInfo>);
    let mut value = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    
    // Load key data when key changes
    use_effect(move |_| {
        let key = selected_key();
        if key.is_empty() {
            return;
        }
        
        let pool = connection_pool.clone();
        spawn(async move {
            loading.set(true);
            
            match pool.get_key_info(&key).await {
                Ok(info) => {
                    key_info.set(Some(info.clone()));
                    
                    // Load value based on type
                    let val = match info.key_type {
                        KeyType::String => pool.get_string_value(&key).await.ok(),
                        KeyType::List => {
                            match pool.get_list_range(&key, 0, 100).await {
                                Ok(items) => Some(format!("{:#?}", items)),
                                Err(_) => None,
                            }
                        }
                        KeyType::Set => {
                            match pool.get_set_members(&key).await {
                                Ok(members) => Some(format!("{:#?}", members)),
                                Err(_) => None,
                            }
                        }
                        KeyType::Hash => {
                            match pool.get_hash_all(&key).await {
                                Ok(fields) => {
                                    let json = serde_json::to_string_pretty(&fields).unwrap_or_default();
                                    Some(json)
                                }
                                Err(_) => None,
                            }
                        }
                        KeyType::ZSet => {
                            match pool.get_zset_range(&key, 0, 100).await {
                                Ok(members) => Some(format!("{:#?}", members)),
                                Err(_) => None,
                            }
                        }
                        _ => None,
                    };
                    
                    if let Some(v) = val {
                        value.set(v);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to load key info: {}", e);
                }
            }
            
            loading.set(false);
        });
    });
    
    rsx! {
        div {
            flex: "1",
            height: "100%",
            background: "#1e1e1e",
            display: "flex",
            flex_direction: "column",
            
            // Header
            div {
                padding: "12px 16px",
                border_bottom: "1px solid #3c3c3c",
                background: "#252526",
                
                if let Some(info) = key_info() {
                    div {
                        display: "flex",
                        justify_content: "space_between",
                        align_items: "center",
                        
                        div {
                            span {
                                color: "#888",
                                font_size: "12px",
                                margin_right: "8px",
                                
                                "Key:"
                            }
                            
                            span {
                                color: "#4ec9b0",
                                font_size: "14px",
                                font_weight: "bold",
                                
                                "{selected_key}"
                            }
                        }
                        
                        div {
                            display: "flex",
                            gap: "16px",
                            font_size: "12px",
                            color: "#888",
                            
                            span {
                                "Type: {:?}",
                                info.key_type
                            }
                            
                            if let Some(ttl) = info.ttl {
                                span {
                                    "TTL: {ttl}s"
                                }
                            }
                        }
                    }
                } else {
                    div {
                        color: "#888",
                        
                        "Select a key to view"
                    }
                }
            }
            
            // Content
            div {
                flex: "1",
                overflow: "auto",
                padding: "16px",
                
                if loading() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "Loading..."
                    }
                } else if selected_key.is_empty() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "No key selected"
                    }
                } else if value().is_empty() {
                    div {
                        color: "#888",
                        text_align: "center",
                        padding: "20px",
                        
                        "No value"
                    }
                } else {
                    pre {
                        color: "#d4d4d4",
                        font_family: "Consolas, 'Courier New', monospace",
                        font_size: "13px",
                        line_height: "1.6",
                        white_space: "pre_wrap",
                        word_wrap: "break_word",
                        
                        "{value}"
                    }
                }
            }
        }
    }
}
```

**Step 2: Update UI module**

Modify `src/ui/mod.rs`:
```rust
mod app;
mod sidebar;
mod connection_form;
mod key_browser;
mod key_item;
mod value_viewer;

pub use app::*;
pub use sidebar::*;
pub use connection_form::*;
pub use key_browser::*;
pub use key_item::*;
pub use value_viewer::*;
```

**Step 3: Update App with value viewer**

Modify `src/ui/app.rs` to include value viewer after KeyBrowser:
```rust
// In the main div, after KeyBrowser, add:
if !selected_key().is_empty() {
    ValueViewer {
        connection_pool: pool.clone(),
        selected_key: selected_key(),
    }
}
```

**Step 4: Commit value viewer**

```bash
git add src/ui/value_viewer.rs src/ui/mod.rs src/ui/app.rs
git commit -m "feat: implement value viewer for key data

- Add ValueViewer component
- Support String, Hash, List, Set, ZSet display
- Show key metadata (type, TTL)
- Format JSON for Hash values
- Integrate with key browser"
```

---

## Summary

This plan implements:
- ✅ Redis operations module (SCAN, TYPE, GET, etc.)
- ✅ Tree structure builder with delimiter parsing
- ✅ Key browser with search and tree view
- ✅ Value viewer for all key types
- ✅ Integration with main application

Next steps:
- Add key editing capabilities
- Implement CLI terminal
- Add more data type viewers (Stream, JSON)
- Optimize for 100k+ keys