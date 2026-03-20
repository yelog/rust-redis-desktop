# Data Editor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement data editing capabilities for String, Hash, List, Set, ZSet with TTL management and key operations.

**Architecture:** Extend ValueViewer with edit mode, add inline editors for each data type, implement CRUD operations with optimistic updates.

**Tech Stack:** Dioxus 0.7, Freya 0.3, redis 1.0, tokio

---

## Task 1: String Editor Component

**Files:**
- Create: `src/ui/string_editor.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/value_viewer.rs`

**Step 1: Create String editor component**

Create `src/ui/string_editor.rs`:
```rust
use dioxus::prelude::*;
use crate::connection::ConnectionPool;

#[component]
pub fn StringEditor(
    connection_pool: ConnectionPool,
    key: String,
    initial_value: String,
    on_save: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut value = use_signal(|| initial_value.clone());
    let mut saving = use_signal(|| false);
    
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100%",
            
            textarea {
                flex: "1",
                padding: "12px",
                background: "#1e1e1e",
                border: "1px solid #3c3c3c",
                border_radius: "4px",
                color: "white",
                font_family: "Consolas, 'Courier New', monospace",
                font_size: "14px",
                resize: "none",
                value: "{value}",
                oninput: move |e| value.set(e.value()),
            }
            
            div {
                display: "flex",
                gap: "8px",
                padding: "12px 0",
                
                button {
                    flex: "1",
                    padding: "8px",
                    background: "#0e639c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    disabled: saving(),
                    onclick: move |_| {
                        let pool = connection_pool.clone();
                        let key = key.clone();
                        let val = value();
                        
                        spawn(async move {
                            saving.set(true);
                            
                            match pool.set_string_value(&key, &val).await {
                                Ok(_) => {
                                    on_save.call(val);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to save: {}", e);
                                }
                            }
                            
                            saving.set(false);
                        });
                    },
                    
                    if saving() { "Saving..." } else { "💾 Save" }
                }
                
                button {
                    flex: "1",
                    padding: "8px",
                    background: "#5a5a5a",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    onclick: move |_| on_cancel.call(()),
                    
                    "✖ Cancel"
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
mod string_editor;

pub use app::*;
pub use sidebar::*;
pub use connection_form::*;
pub use key_browser::*;
pub use key_item::*;
pub use value_viewer::*;
pub use string_editor::*;
```

**Step 3: Integrate editor into ValueViewer**

Modify `src/ui/value_viewer.rs` to add edit mode:
```rust
// Add edit mode state
let mut editing = use_signal(|| false);

// In the content div, add edit/view toggle
if editing() {
    StringEditor {
        connection_pool: connection_pool.clone(),
        key: selected_key.clone(),
        initial_value: value(),
        on_save: move |new_value| {
            value.set(new_value);
            editing.set(false);
        },
        on_cancel: move |_| editing.set(false),
    }
} else {
    // Existing view mode
    pre { ... }
}

// Add Edit button in header
button {
    padding: "4px 12px",
    background: "#0e639c",
    color: "white",
    border: "none",
    border_radius: "4px",
    cursor: "pointer",
    font_size: "12px",
    onclick: move |_| editing.set(true),
    
    "✏️ Edit"
}
```

**Step 4: Commit String editor**

```bash
git add src/ui/string_editor.rs src/ui/mod.rs src/ui/value_viewer.rs
git commit -m "feat: implement String editor with save/cancel

- Add StringEditor component with textarea
- Add edit mode toggle in ValueViewer
- Support save and cancel operations
- Add loading state during save"
```

---

## Task 2: Hash Editor Component

**Files:**
- Create: `src/ui/hash_editor.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/value_viewer.rs`
- Modify: `src/redis/commands.rs`

**Step 1: Add Hash operations to Redis commands**

Modify `src/redis/commands.rs`:
```rust
pub async fn hash_set_field(&self, key: &str, field: &str, value: &str) -> Result<()> {
    let mut connection = self.connection.lock().await;
    
    if let Some(ref mut conn) = *connection {
        conn.hset(key, field, value)
            .await
            .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
    } else {
        Err(ConnectionError::Closed)
    }
}

pub async fn hash_delete_field(&self, key: &str, field: &str) -> Result<bool> {
    let mut connection = self.connection.lock().await;
    
    if let Some(ref mut conn) = *connection {
        let deleted: i32 = conn
            .hdel(key, field)
            .await
            .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
        
        Ok(deleted > 0)
    } else {
        Err(ConnectionError::Closed)
    }
}
```

**Step 2: Create Hash editor component**

Create `src/ui/hash_editor.rs`:
```rust
use dioxus::prelude::*;
use crate::connection::ConnectionPool;
use std::collections::HashMap;

#[component]
pub fn HashEditor(
    connection_pool: ConnectionPool,
    key: String,
    initial_fields: HashMap<String, String>,
    on_save: EventHandler<HashMap<String, String>>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut fields = use_signal(|| initial_fields.clone());
    let mut new_field_name = use_signal(String::new);
    let mut new_field_value = use_signal(String::new);
    let mut saving = use_signal(|| false);
    
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "100%",
            
            // Field list
            div {
                flex: "1",
                overflow_y: "auto",
                padding: "12px",
                
                for (field_name, field_value) in fields.read().iter() {
                    div {
                        key: "{field_name}",
                        display: "flex",
                        gap: "8px",
                        margin_bottom: "8px",
                        align_items: "center",
                        
                        input {
                            flex: "1",
                            padding: "6px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{field_name}",
                            readonly: true,
                        }
                        
                        input {
                            flex: "2",
                            padding: "6px",
                            background: "#3c3c3c",
                            border: "1px solid #555",
                            border_radius: "4px",
                            color: "white",
                            value: "{field_value}",
                            oninput: {
                                let field_name = field_name.clone();
                                move |e| {
                                    fields.write().insert(field_name.clone(), e.value());
                                }
                            },
                        }
                        
                        button {
                            padding: "6px 12px",
                            background: "#c53030",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            onclick: {
                                let field_name = field_name.clone();
                                move |_| {
                                    fields.write().remove(&field_name);
                                }
                            },
                            
                            "🗑️"
                        }
                    }
                }
            }
            
            // Add new field
            div {
                padding: "12px",
                border_top: "1px solid #3c3c3c",
                
                div {
                    display: "flex",
                    gap: "8px",
                    
                    input {
                        flex: "1",
                        padding: "6px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        placeholder: "Field name",
                        value: "{new_field_name}",
                        oninput: move |e| new_field_name.set(e.value()),
                    }
                    
                    input {
                        flex: "2",
                        padding: "6px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        placeholder: "Field value",
                        value: "{new_field_value}",
                        oninput: move |e| new_field_value.set(e.value()),
                    }
                    
                    button {
                        padding: "6px 12px",
                        background: "#38a169",
                        color: "white",
                        border: "none",
                        border_radius: "4px",
                        cursor: "pointer",
                        onclick: move |_| {
                            if !new_field_name().is_empty() {
                                fields.write().insert(new_field_name(), new_field_value());
                                new_field_name.set(String::new());
                                new_field_value.set(String::new());
                            }
                        },
                        
                        "+ Add"
                    }
                }
            }
            
            // Action buttons
            div {
                display: "flex",
                gap: "8px",
                padding: "12px 0",
                
                button {
                    flex: "1",
                    padding: "8px",
                    background: "#0e639c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    disabled: saving(),
                    onclick: move |_| {
                        let pool = connection_pool.clone();
                        let key = key.clone();
                        let fields_to_save = fields();
                        
                        spawn(async move {
                            saving.set(true);
                            
                            // Save all fields
                            for (field, value) in fields_to_save.iter() {
                                let _ = pool.hash_set_field(&key, field, value).await;
                            }
                            
                            on_save.call(fields_to_save);
                            saving.set(false);
                        });
                    },
                    
                    if saving() { "Saving..." } else { "💾 Save" }
                }
                
                button {
                    flex: "1",
                    padding: "8px",
                    background: "#5a5a5a",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    onclick: move |_| on_cancel.call(()),
                    
                    "✖ Cancel"
                }
            }
        }
    }
}
```

**Step 3: Update UI module**

Modify `src/ui/mod.rs`:
```rust
mod hash_editor;

pub use hash_editor::*;
```

**Step 4: Commit Hash editor**

```bash
git add src/ui/hash_editor.rs src/ui/mod.rs src/redis/commands.rs
git commit -m "feat: implement Hash editor with field management

- Add hash_set_field and hash_delete_field commands
- Add HashEditor component with field list
- Support add/edit/delete fields
- Integrate with ValueViewer"
```

---

## Task 3: TTL Management Component

**Files:**
- Create: `src/ui/ttl_editor.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/value_viewer.rs`
- Modify: `src/redis/commands.rs`

**Step 1: Add TTL operations to Redis commands**

Modify `src/redis/commands.rs`:
```rust
pub async fn set_ttl(&self, key: &str, ttl: i64) -> Result<bool> {
    let mut connection = self.connection.lock().await;
    
    if let Some(ref mut conn) = *connection {
        let result: i32 = conn
            .expire(key, ttl)
            .await
            .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
        
        Ok(result > 0)
    } else {
        Err(ConnectionError::Closed)
    }
}

pub async fn remove_ttl(&self, key: &str) -> Result<bool> {
    let mut connection = self.connection.lock().await;
    
    if let Some(ref mut conn) = *connection {
        let result: i32 = conn
            .persist(key)
            .await
            .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))?;
        
        Ok(result > 0)
    } else {
        Err(ConnectionError::Closed)
    }
}
```

**Step 2: Create TTL editor component**

Create `src/ui/ttl_editor.rs`:
```rust
use dioxus::prelude::*;
use crate::connection::ConnectionPool;

#[component]
pub fn TTLEditor(
    connection_pool: ConnectionPool,
    key: String,
    current_ttl: Option<i64>,
    on_save: EventHandler<Option<i64>>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut ttl_value = use_signal(|| current_ttl.unwrap_or(3600).to_string());
    let mut no_expiry = use_signal(|| current_ttl.is_none());
    let mut saving = use_signal(|| false);
    
    rsx! {
        div {
            padding: "16px",
            background: "#252526",
            border_radius: "8px",
            
            h3 {
                color: "white",
                margin_bottom: "16px",
                
                "⏱️ TTL Settings"
            }
            
            div {
                margin_bottom: "16px",
                
                label {
                    display: "flex",
                    align_items: "center",
                    gap: "8px",
                    color: "white",
                    cursor: "pointer",
                    
                    input {
                        r#type: "checkbox",
                        checked: no_expiry(),
                        onchange: move |e| no_expiry.set(e.checked()),
                    }
                    
                    "No expiry (persist)"
                }
            }
            
            if !no_expiry() {
                div {
                    margin_bottom: "16px",
                    
                    label {
                        display: "block",
                        color: "#888",
                        margin_bottom: "8px",
                        
                        "TTL (seconds)"
                    }
                    
                    input {
                        width: "100%",
                        padding: "8px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        r#type: "number",
                        min: "1",
                        value: "{ttl_value}",
                        oninput: move |e| ttl_value.set(e.value()),
                    }
                }
            }
            
            div {
                display: "flex",
                gap: "8px",
                
                button {
                    flex: "1",
                    padding: "8px",
                    background: "#0e639c",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    disabled: saving(),
                    onclick: move |_| {
                        let pool = connection_pool.clone();
                        let key = key.clone();
                        let no_exp = no_expiry();
                        let ttl = ttl_value().parse::<i64>().unwrap_or(3600);
                        
                        spawn(async move {
                            saving.set(true);
                            
                            if no_exp {
                                match pool.remove_ttl(&key).await {
                                    Ok(_) => on_save.call(None),
                                    Err(e) => tracing::error!("Failed to remove TTL: {}", e),
                                }
                            } else {
                                match pool.set_ttl(&key, ttl).await {
                                    Ok(_) => on_save.call(Some(ttl)),
                                    Err(e) => tracing::error!("Failed to set TTL: {}", e),
                                }
                            }
                            
                            saving.set(false);
                        });
                    },
                    
                    if saving() { "Saving..." } else { "💾 Save" }
                }
                
                button {
                    flex: "1",
                    padding: "8px",
                    background: "#5a5a5a",
                    color: "white",
                    border: "none",
                    border_radius: "4px",
                    cursor: "pointer",
                    onclick: move |_| on_cancel.call(()),
                    
                    "✖ Cancel"
                }
            }
        }
    }
}
```

**Step 3: Commit TTL editor**

```bash
git add src/ui/ttl_editor.rs src/ui/mod.rs src/redis/commands.rs
git commit -m "feat: implement TTL management component

- Add set_ttl and remove_ttl commands
- Add TTLEditor with persist option
- Support setting custom TTL
- Integrate with ValueViewer"
```

---

## Task 4: Key Operations (Delete, Rename)

**Files:**
- Create: `src/ui/key_actions.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/key_browser.rs`
- Modify: `src/redis/commands.rs`

**Step 1: Add rename operation to Redis commands**

Modify `src/redis/commands.rs`:
```rust
pub async fn rename_key(&self, old_key: &str, new_key: &str) -> Result<()> {
    let mut connection = self.connection.lock().await;
    
    if let Some(ref mut conn) = *connection {
        conn.rename(old_key, new_key)
            .await
            .map_err(|e| ConnectionError::ConnectionFailed(e.to_string()))
    } else {
        Err(ConnectionError::Closed)
    }
}
```

**Step 2: Create key actions component**

Create `src/ui/key_actions.rs`:
```rust
use dioxus::prelude::*;
use crate::connection::ConnectionPool;

#[component]
pub fn KeyActions(
    connection_pool: ConnectionPool,
    key: String,
    on_delete: EventHandler<()>,
    on_rename: EventHandler<String>,
) -> Element {
    let mut show_delete_confirm = use_signal(|| false);
    let mut show_rename_dialog = use_signal(|| false);
    let mut new_key_name = use_signal(String::new);
    let mut processing = use_signal(|| false);
    
    rsx! {
        div {
            display: "flex",
            gap: "8px",
            
            button {
                padding: "4px 12px",
                background: "#c53030",
                color: "white",
                border: "none",
                border_radius: "4px",
                cursor: "pointer",
                font_size: "12px",
                onclick: move |_| show_delete_confirm.set(true),
                
                "🗑️ Delete"
            }
            
            button {
                padding: "4px 12px",
                background: "#805ad5",
                color: "white",
                border: "none",
                border_radius: "4px",
                cursor: "pointer",
                font_size: "12px",
                onclick: move |_| {
                    new_key_name.set(key.clone());
                    show_rename_dialog.set(true);
                },
                
                "✏️ Rename"
            }
        }
        
        // Delete confirmation dialog
        if show_delete_confirm() {
            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                background: "rgba(0, 0, 0, 0.7)",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                z_index: "1000",
                
                div {
                    background: "#252526",
                    padding: "24px",
                    border_radius: "8px",
                    max_width: "400px",
                    
                    h3 {
                        color: "white",
                        margin_bottom: "16px",
                        
                        "⚠️ Confirm Delete"
                    }
                    
                    p {
                        color: "#888",
                        margin_bottom: "24px",
                        
                        "Are you sure you want to delete '{key}'?"
                    }
                    
                    div {
                        display: "flex",
                        gap: "8px",
                        
                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#c53030",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            disabled: processing(),
                            onclick: move |_| {
                                let pool = connection_pool.clone();
                                let key = key.clone();
                                
                                spawn(async move {
                                    processing.set(true);
                                    
                                    match pool.delete_key(&key).await {
                                        Ok(_) => {
                                            show_delete_confirm.set(false);
                                            on_delete.call(());
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to delete: {}", e);
                                        }
                                    }
                                    
                                    processing.set(false);
                                });
                            },
                            
                            if processing() { "Deleting..." } else { "🗑️ Delete" }
                        }
                        
                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#5a5a5a",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            onclick: move |_| show_delete_confirm.set(false),
                            
                            "Cancel"
                        }
                    }
                }
            }
        }
        
        // Rename dialog
        if show_rename_dialog() {
            div {
                position: "fixed",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                background: "rgba(0, 0, 0, 0.7)",
                display: "flex",
                align_items: "center",
                justify_content: "center",
                z_index: "1000",
                
                div {
                    background: "#252526",
                    padding: "24px",
                    border_radius: "8px",
                    max_width: "400px",
                    
                    h3 {
                        color: "white",
                        margin_bottom: "16px",
                        
                        "✏️ Rename Key"
                    }
                    
                    input {
                        width: "100%",
                        padding: "8px",
                        background: "#3c3c3c",
                        border: "1px solid #555",
                        border_radius: "4px",
                        color: "white",
                        margin_bottom: "16px",
                        value: "{new_key_name}",
                        oninput: move |e| new_key_name.set(e.value()),
                    }
                    
                    div {
                        display: "flex",
                        gap: "8px",
                        
                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#805ad5",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            disabled: processing() || new_key_name().is_empty(),
                            onclick: move |_| {
                                let pool = connection_pool.clone();
                                let old_key = key.clone();
                                let new_key = new_key_name();
                                
                                spawn(async move {
                                    processing.set(true);
                                    
                                    match pool.rename_key(&old_key, &new_key).await {
                                        Ok(_) => {
                                            show_rename_dialog.set(false);
                                            on_rename.call(new_key);
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to rename: {}", e);
                                        }
                                    }
                                    
                                    processing.set(false);
                                });
                            },
                            
                            if processing() { "Renaming..." } else { "✓ Rename" }
                        }
                        
                        button {
                            flex: "1",
                            padding: "8px",
                            background: "#5a5a5a",
                            color: "white",
                            border: "none",
                            border_radius: "4px",
                            cursor: "pointer",
                            onclick: move |_| show_rename_dialog.set(false),
                            
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}
```

**Step 3: Commit key actions**

```bash
git add src/ui/key_actions.rs src/ui/mod.rs src/redis/commands.rs
git commit -m "feat: implement key operations (delete, rename)

- Add rename_key command
- Add KeyActions component with delete/rename
- Add confirmation dialogs
- Integrate with KeyBrowser"
```

---

## Summary

This plan implements:
- ✅ String editor with textarea
- ✅ Hash editor with field management
- ✅ TTL editor with persist option
- ✅ Key delete with confirmation
- ✅ Key rename dialog
- ✅ Integration with existing components

Next steps:
- Add List/Set/ZSet editors
- Add CLI terminal
- Add key import/export
- Add batch operations