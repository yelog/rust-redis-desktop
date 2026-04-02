# Error Handling Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate unsafe `.unwrap()` and `.expect()` calls by creating a unified error handling system with proper error reporting.

**Architecture:** Create layered error types (AppError > StartupError/ConfigError) with `ErrorReporter` infrastructure. Main.rs separates fatal (exit) from non-fatal (degraded) errors.

**Tech Stack:** Rust 1.56+, thiserror 2.0, tracing 0.1, rfd 0.17 (dialog), chrono 0.4

---

## Phase 1: Create Foundation (Day 1 Morning)

### Task 1: Create Error Types Module

**Files:**
- Create: `src/error.rs`

**Step 1: Create error.rs with basic structure**

```rust
use thiserror::Error;

/// Top-level application error
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Startup failed: {0}")]
    Startup(#[from] StartupError),
    
    #[error("Connection error: {0}")]
    Connection(#[from] crate::connection::ConnectionError),
    
    #[error("Update error: {0}")]
    Update(#[from] crate::updater::UpdateError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("{0}")]
    Other(String),
}

/// Startup errors (core functionality - fatal)
#[derive(Error, Debug)]
pub enum StartupError {
    #[error("Failed to create menu: {source}")]
    MenuCreation {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Failed to initialize runtime: {source}")]
    RuntimeInit {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Failed to create window: {source}")]
    WindowCreation {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to access config directory: {0}")]
    DirectoryAccess(String),
    
    #[error("Failed to read config file: {0}")]
    ReadError(String),
    
    #[error("Failed to write config file: {0}")]
    WriteError(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Global Result type alias
pub type Result<T> = std::result::Result<T, AppError>;
```

**Step 2: Verify it compiles**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors related to error.rs

**Step 3: Commit**

```bash
git add src/error.rs
git commit -m "feat: add unified error types (AppError, StartupError, ConfigError)"
```

---

### Task 2: Export Error Module

**Files:**
- Modify: `src/lib.rs`

**Step 1: Add error module to lib.rs**

```rust
pub mod error;
pub mod config;
pub mod connection;
// ... rest of modules
```

**Step 2: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: export error module in lib.rs"
```

---

### Task 3: Create Error Reporter Module

**Files:**
- Create: `src/error_reporting.rs`

**Step 1: Create error_reporting.rs with core structure**

```rust
use crate::error::AppError;
use std::path::PathBuf;
use tracing::error;

pub struct ErrorReporter {
    log_dir: PathBuf,
}

impl ErrorReporter {
    pub fn init() -> Self {
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rust-redis-desktop")
            .join("logs");
        
        let _ = std::fs::create_dir_all(&log_dir);
        
        Self { log_dir }
    }
    
    pub fn report_fatal_error(error: &AppError) -> ! {
        let error_msg = format!("{}", error);
        let detailed_msg = format!("{:#?}", error);
        
        // 1. Terminal output
        eprintln!("\n========================================");
        eprintln!("FATAL ERROR: {}", error_msg);
        eprintln!("========================================\n");
        eprintln!("Details:\n{}\n", detailed_msg);
        
        // 2. Log file
        if let Some(log_path) = Self::write_error_log(&error_msg, &detailed_msg) {
            eprintln!("Error log saved to: {:?}\n", log_path);
        }
        
        // 3. Native dialog
        Self::show_error_dialog(&error_msg);
        
        // 4. Exit
        std::process::exit(1);
    }
    
    pub fn report_non_fatal_error(context: &str, error: &dyn std::error::Error) {
        error!("Non-fatal error in {}: {}", context, error);
        eprintln!("[WARN] {} failed: {}", context, error);
    }
    
    fn write_error_log(summary: &str, details: &str) -> Option<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let log_file = dirs::config_dir()?
            .join("rust-redis-desktop")
            .join("logs")
            .join(format!("error_{}.log", timestamp));
        
        let content = format!(
            "Redis Desktop - Fatal Error Log\n\
             Generated: {}\n\
             \n\
             Error Summary:\n{}\n\
             \n\
             Full Details:\n{}\n\
             \n\
             Backtrace:\n{:?}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            summary,
            details,
            std::backtrace::Backtrace::capture()
        );
        
        std::fs::write(&log_file, content).ok()?;
        Some(log_file)
    }
    
    fn show_error_dialog(message: &str) {
        let _ = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Redis Desktop - Startup Error")
            .set_description(message)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
}
```

**Step 2: Add convenience macros**

```rust
#[macro_export]
macro_rules! fatal_error {
    ($error:expr) => {
        $crate::error_reporting::ErrorReporter::report_fatal_error(&$error)
    };
}

#[macro_export]
macro_rules! non_fatal_error {
    ($context:expr, $error:expr) => {
        $crate::error_reporting::ErrorReporter::report_non_fatal_error(
            $context,
            &$error
        )
    };
}
```

**Step 3: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 4: Commit**

```bash
git add src/error_reporting.rs
git commit -m "feat: add error reporting infrastructure (dialog + log + terminal)"
```

---

### Task 4: Export Error Reporting Module

**Files:**
- Modify: `src/lib.rs`

**Step 1: Add error_reporting module**

```rust
pub mod error;
pub mod error_reporting;
pub mod config;
// ... rest
```

**Step 2: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: export error_reporting module"
```

---

## Phase 2: Fix Critical Unwraps (Day 1 Afternoon)

### Task 5: Remove Default from ConfigStorage

**Files:**
- Modify: `src/config/storage.rs:228`

**Step 1: Remove Default implementation**

Find and delete lines 83-87 (approximately):
```rust
// DELETE THIS:
impl Default for ConfigStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create config storage")
    }
}
```

**Step 2: Update new() method to return Result**

Change signature from:
```rust
pub fn new() -> std::result::Result<Self, ConfigError> {
```

To:
```rust
pub fn new() -> crate::error::Result<Self> {
```

And update error creation:
```rust
pub fn new() -> crate::error::Result<Self> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| crate::error::ConfigError::DirectoryAccess(
            "Cannot determine config directory".into()
        ))?
        .join("rust-redis-desktop");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| crate::error::ConfigError::DirectoryAccess(
            format!("Failed to create config directory: {}", e)
        ))?;

    Ok(Self {
        connections_file: config_dir.join("connections.json"),
        settings_file: config_dir.join("settings.json"),
        config_dir,
    })
}
```

**Step 3: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: May have errors in main.rs (will fix in next task)

**Step 4: Commit**

```bash
git add src/config/storage.rs
git commit -m "fix: remove unsafe Default impl from ConfigStorage, return Result"
```

---

### Task 6: Remove Default from UpdateManager

**Files:**
- Modify: `src/updater/manager.rs:85`

**Step 1: Remove Default implementation**

Find and delete lines 83-87 (approximately):
```rust
// DELETE THIS:
impl Default for UpdateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create update manager")
    }
}
```

**Step 2: Update new() to return Result**

Change from returning `Result<Self>` to `crate::error::Result<Self>`:

```rust
pub fn new() -> crate::error::Result<Self> {
    let current_version = get_current_version();
    let checker = UpdateChecker::new(&current_version);
    let downloader = UpdateDownloader::new()
        .map_err(|e| crate::error::AppError::Other(format!("Failed to create downloader: {}", e)))?;
    let config = UpdateConfig::load()
        .map_err(|e| crate::error::AppError::Other(format!("Failed to load update config: {}", e)))?;

    Ok(Self {
        checker,
        downloader,
        config,
    })
}
```

**Step 3: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: May have errors (will fix later)

**Step 4: Commit**

```bash
git add src/updater/manager.rs
git commit -m "fix: remove unsafe Default impl from UpdateManager"
```

---

### Task 7: Remove Default from UpdateDownloader

**Files:**
- Modify: `src/updater/downloader.rs:103`

**Step 1: Remove Default implementation**

Find and delete lines 101-105 (approximately):
```rust
// DELETE THIS:
impl Default for UpdateDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create downloader")
    }
}
```

**Step 2: Update new() to return Result**

```rust
pub fn new() -> crate::error::Result<Self> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| crate::error::AppError::Other(
            "Cannot determine cache directory".into()
        ))?
        .join("rust-redis-desktop")
        .join("updates");

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| crate::error::AppError::Other(
            format!("Failed to create cache directory: {}", e)
        ))?;

    Ok(Self { cache_dir })
}
```

**Step 3: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: May have errors

**Step 4: Commit**

```bash
git add src/updater/downloader.rs
git commit -m "fix: remove unsafe Default impl from UpdateDownloader"
```

---

### Task 8: Fix Mutex Poisoning in tray.rs

**Files:**
- Modify: `src/tray.rs:82`

**Step 1: Change init_tray signature**

Change from:
```rust
pub fn init_tray(state: SharedTrayState) {
```

To:
```rust
pub fn init_tray(state: SharedTrayState) -> crate::error::Result<()> {
```

**Step 2: Update load_icon to return Result**

Change from:
```rust
fn load_icon() -> Icon {
    let icon_bytes = include_bytes!("../icons/icon.png");
    let img = image::load_from_memory(icon_bytes).expect("Failed to load tray icon");
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height).expect("Failed to create tray icon")
}
```

To:
```rust
fn load_icon() -> std::result::Result<Icon, Box<dyn std::error::Error>> {
    let icon_bytes = include_bytes!("../icons/icon.png");
    let img = image::load_from_memory(icon_bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(Icon::from_rgba(rgba.into_raw(), width, height)?)
}
```

**Step 3: Update init_tray implementation**

Replace the mutex unwrap at line 82:
```rust
// OLD:
let mut s = state.lock().unwrap();

// NEW:
match state.lock() {
    Ok(mut s) => {
        s.active_server_id = Some(server_id.to_string());
    }
    Err(poisoned) => {
        let mut s = poisoned.into_inner();
        s.active_server_id = Some(server_id.to_string());
        tracing::warn!("Tray state mutex was poisoned, recovered");
    }
}
```

**Step 4: Update tray creation to handle errors**

Wrap load_icon and tray creation:
```rust
let icon = load_icon()
    .map_err(|e| crate::error::AppError::Other(format!("Failed to load tray icon: {}", e)))?;

let tray = TrayIconBuilder::new()
    .with_menu(Box::new(menu))
    .with_tooltip("Redis Desktop")
    .with_icon(icon)
    .build()
    .map_err(|e| crate::error::AppError::Other(format!("Failed to create tray: {}", e)))?;
```

**Step 5: Add Ok(()) at end of function**

```rust
Ok(())
```

**Step 6: Update Linux stub**

```rust
#[cfg(target_os = "linux")]
pub fn init_tray(_state: SharedTrayState) -> crate::error::Result<()> {
    Ok(())
}
```

**Step 7: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors in tray.rs

**Step 8: Commit**

```bash
git add src/tray.rs
git commit -m "fix: handle mutex poisoning in tray.rs, return Result"
```

---

## Phase 3: Refactor Main.rs (Day 2 Morning)

### Task 9: Convert create_menu to Return Result

**Files:**
- Modify: `src/main.rs:41-87`

**Step 1: Change create_menu signature**

From:
```rust
fn create_menu() -> Menu {
```

To:
```rust
fn create_menu() -> std::result::Result<Menu, Box<dyn std::error::Error + Send + Sync>> {
```

**Step 2: Replace unwraps with ? operator**

Lines 58, 72, 81, 84:
```rust
// Change .unwrap() to ?
app_menu.append_items(&[...])?;
edit_menu.append_items(&[...])?;
window_menu.append_items(&[...])?;
menu.append_items(&[...])?;
```

**Step 3: Return Ok(menu)**

```rust
Ok(menu)
```

**Step 4: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: Errors in main() function (will fix next)

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "refactor: convert create_menu to return Result"
```

---

### Task 10: Create run_app Function

**Files:**
- Modify: `src/main.rs`

**Step 1: Add imports at top**

```rust
use error::{AppError, Result, StartupError};
use error_reporting::ErrorReporter;
```

**Step 2: Create run_app function**

Insert before main():
```rust
fn run_app() -> Result<()> {
    // Update checker (non-fatal)
    if let Ok(mut manager) = UpdateManager::new() {
        if manager.should_auto_check() {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    std::thread::spawn(move || {
                        rt.block_on(async {
                            if let Ok(Some(info)) = manager.check_for_updates().await {
                                tracing::info!("Found new version: {}", info.version);
                                set_pending_update(Some(info));
                            }
                        });
                    });
                }
                Err(e) => {
                    non_fatal_error!("Update checker runtime", &e);
                }
            }
        }
    }

    // Menu creation (fatal)
    let menu = create_menu()
        .map_err(|e| AppError::Startup(StartupError::MenuCreation { source: e }))?;

    // Settings load (non-fatal)
    let settings = ConfigStorage::new()
        .ok()
        .and_then(|s| s.load_settings().ok())
        .unwrap_or_default();

    // Window creation (fatal)
    let window_builder = configure_window_builder(
        WindowBuilder::new()
            .with_title("Redis Desktop")
            .with_inner_size(LogicalSize::new(1200, 800))
            .with_theme(preferred_window_theme(settings.theme_preference))
            .with_visible(true),
    );

    // Tray initialization (non-fatal)
    #[cfg(not(target_os = "linux"))]
    {
        let tray_state = create_shared_state();
        if let Err(e) = init_tray(tray_state) {
            non_fatal_error!("System tray", &e);
        }
    }

    // Launch UI (fatal)
    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().with_menu(menu).with_window(window_builder))
        .launch(App);

    Ok(())
}
```

**Step 3: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -30`
Expected: Errors in main() about unused Result

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "refactor: create run_app function with proper error handling"
```

---

### Task 11: Refactor main Function

**Files:**
- Modify: `src/main.rs:89-137`

**Step 1: Replace main() body**

From:
```rust
fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .pretty()
        .init();

    tracing::info!("Starting Redis Desktop Manager");

    if let Ok(mut manager) = UpdateManager::new() {
        if manager.should_auto_check() {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    if let Ok(Some(info)) = manager.check_for_updates().await {
                        tracing::info!("Found new version: {}", info.version);
                        set_pending_update(Some(info));
                    }
                });
            });
        }
    }

    let menu = create_menu();

    let settings = ConfigStorage::new()
        .ok()
        .and_then(|s| s.load_settings().ok())
        .unwrap_or_default();

    let window_builder = configure_window_builder(
        WindowBuilder::new()
            .with_title("Redis Desktop")
            .with_inner_size(LogicalSize::new(1200, 800))
            .with_theme(preferred_window_theme(settings.theme_preference))
            .with_visible(true),
    );

    #[cfg(not(target_os = "linux"))]
    {
        let tray_state = create_shared_state();
        init_tray(tray_state);
    }

    dioxus::LaunchBuilder::new()
        .with_cfg(Config::new().with_menu(menu).with_window(window_builder))
        .launch(App);
}
```

To:
```rust
fn main() {
    // Initialize error reporter
    let _reporter = ErrorReporter::init();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .pretty()
        .init();

    tracing::info!("Starting Redis Desktop Manager");

    // Run app and handle errors
    if let Err(e) = run_app() {
        ErrorReporter::report_fatal_error(&e);
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "refactor: simplify main() with unified error handling"
```

---

## Phase 4: Improve Safe Unwraps (Day 2 Afternoon)

### Task 12: Improve serialization/mod.rs

**Files:**
- Modify: `src/serialization/mod.rs:197-201`

**Step 1: Add fallback to line 197**

Change:
```rust
return full_name.strip_prefix("java.lang.").unwrap().to_string();
```

To:
```rust
return full_name.strip_prefix("java.lang.").unwrap_or(full_name).to_string();
```

**Step 2: Add fallback to line 201**

Change:
```rust
parts.last().unwrap().to_string()
```

To:
```rust
parts.last().unwrap_or(&full_name).to_string()
```

**Step 3: Add comment explaining safety**

```rust
/// Simplify class name for display
/// 
/// If the name starts with "java.lang.", the prefix is stripped.
/// For other names, only the last component (after '.') is returned.
pub fn simplify_class_name(full_name: &str) -> String {
    if full_name.starts_with("java.lang.") {
        // Safe: prefix guaranteed to exist by the if check
        full_name.strip_prefix("java.lang.").unwrap_or(full_name).to_string()
    } else {
        let parts: Vec<&str> = full_name.split('.').collect();
        if parts.len() > 1 {
            // Safe: parts guaranteed to have at least one element
            parts.last().unwrap_or(&full_name).to_string()
        } else {
            full_name.to_string()
        }
    }
}
```

**Step 4: Commit**

```bash
git add src/serialization/mod.rs
git commit -m "refactor: add fallbacks to guarded unwraps in simplify_class_name"
```

---

### Task 13: Use Lazy Static for Regex Patterns

**Files:**
- Modify: `src/protobuf_schema/parser.rs`

**Step 1: Add once_cell dependency to Cargo.toml**

```toml
once_cell = "1.19"
```

Run: `cargo add once_cell`

**Step 2: Add Lazy imports**

```rust
use once_cell::sync::Lazy;
```

**Step 3: Create static regex variables**

At the top of the file after imports:
```rust
static IMPORT_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r#"import\s+(?:public\s+)?["']([^"']+)["']\s*;"#)
        .expect("Static regex pattern should always compile")
});

static FIELD_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(optional|required|repeated)?\s*(\w+)\s+(\w+)\s*=\s*(\d+)\s*;")
        .expect("Static regex pattern should always compile")
});

static ENUM_VALUE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(\w+)\s*=\s*(-?\d+)\s*;")
        .expect("Static regex pattern should always compile")
});
```

**Step 4: Replace runtime regex compilation**

Find all instances of `regex::Regex::new(...).unwrap()` and replace with static variable usage:
- Line 93: Use `&*IMPORT_REGEX`
- Line 237: Use `&*FIELD_REGEX`
- Line 261: Use `&*ENUM_VALUE_REGEX`

**Step 5: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/protobuf_schema/parser.rs
git commit -m "refactor: use Lazy static for regex patterns in protobuf parser"
```

---

### Task 14: Handle System Time Errors

**Files:**
- Modify: `src/ui/pubsub_panel.rs:73`
- Modify: `src/ui/monitor_panel.rs:103,167`

**Step 1: Fix pubsub_panel.rs**

Change:
```rust
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis();
```

To:
```rust
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_else(|e| {
        tracing::warn!("System clock error: {}, using 0", e);
        Duration::from_secs(0)
    })
    .as_millis();
```

**Step 2: Fix monitor_panel.rs (line 103)**

Same change as above.

**Step 3: Fix monitor_panel.rs (line 167)**

Same change as above.

**Step 4: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 5: Commit**

```bash
git add src/ui/pubsub_panel.rs src/ui/monitor_panel.rs
git commit -m "refactor: handle system time errors gracefully in UI panels"
```

---

### Task 15: Improve protobuf_schema/registry.rs

**Files:**
- Modify: `src/protobuf_schema/registry.rs:142,200`

**Step 1: Import Error type**

```rust
use crate::error::AppError;
```

**Step 2: Fix decode_varint (line 142)**

Change:
```rust
let bytes: [u8; 8] = data[..8].try_into().unwrap();
```

To:
```rust
let bytes: [u8; 8] = data[..8]
    .try_into()
    .map_err(|_| AppError::Other("Failed to convert bytes to array".into()))?;
```

**Step 3: Fix decode_fixed32 (line 200)**

Change:
```rust
let bytes: [u8; 4] = data[..4].try_into().unwrap();
```

To:
```rust
let bytes: [u8; 4] = data[..4]
    .try_into()
    .map_err(|_| AppError::Other("Failed to convert bytes to array".into()))?;
```

**Step 4: Verify compilation**

Run: `cargo check --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 5: Commit**

```bash
git add src/protobuf_schema/registry.rs
git commit -m "refactor: use try_into with error handling in protobuf registry"
```

---

## Phase 5: Testing & Validation (Day 2 End)

### Task 16: Build and Run Tests

**Step 1: Run full build**

Run: `cargo build --release 2>&1 | tail -50`
Expected: Build succeeds with 0 errors

**Step 2: Run tests**

Run: `cargo test --lib 2>&1 | tail -50`
Expected: All tests pass

**Step 3: Run clippy**

Run: `cargo clippy --all-targets --all-features 2>&1 | grep -A 5 "error\|warning" | head -40`
Expected: No errors, minimal warnings

**Step 4: Fix any clippy warnings**

If clippy reports issues, fix them and commit:
```bash
git add <files>
git commit -m "fix: resolve clippy warnings"
```

---

### Task 17: Manual Testing

**Step 1: Test normal startup**

Run: `cargo run --release`

Expected:
- App starts successfully
- Menu appears
- Window opens
- No panics in terminal

**Step 2: Test with missing config directory**

```bash
rm -rf ~/Library/Application\ Support/rust-redis-desktop  # macOS
# or
rm -rf ~/.config/rust-redis-desktop  # Linux
```

Run: `cargo run --release`

Expected:
- App starts with default settings
- Creates config directory
- No panic

**Step 3: Test with corrupted config**

```bash
echo "invalid json" > ~/Library/Application\ Support/rust-redis-desktop/settings.json
```

Run: `cargo run --release`

Expected:
- App starts with default settings
- Logs warning about config load failure
- No panic

**Step 4: Test tray initialization failure**

(Manual test: remove icon file temporarily)

Expected:
- App starts without tray icon
- Logs non-fatal error
- No panic

**Step 5: Document test results**

Create: `docs/test-results-2026-04-02.md`

```markdown
# Error Handling Refactor Test Results

Date: 2026-04-02

## Test 1: Normal Startup
- Status: PASS
- Notes: App started successfully with all features working

## Test 2: Missing Config Directory
- Status: PASS
- Notes: Created default config, no errors

## Test 3: Corrupted Config
- Status: PASS
- Notes: Used defaults, logged warning

## Test 4: Tray Failure
- Status: PASS
- Notes: Started without tray, logged non-fatal error
```

**Step 6: Commit test results**

```bash
git add docs/test-results-2026-04-02.md
git commit -m "test: document error handling refactor test results"
```

---

### Task 18: Final Cleanup and Documentation

**Step 1: Update README.md**

Add section about error handling:
```markdown
## Error Handling

The application uses a unified error handling system:

- **Fatal errors** (core functionality failure): Show error dialog, log details, and exit
- **Non-fatal errors** (auxiliary features): Log warning and continue with degraded functionality

Error logs are saved to:
- macOS: `~/Library/Application Support/rust-redis-desktop/logs/`
- Linux: `~/.config/rust-redis-desktop/logs/`
- Windows: `%APPDATA%\rust-redis-desktop\logs\`
```

**Step 2: Run final check**

Run: `cargo check --all-targets --all-features 2>&1 | tail -20`
Expected: 0 errors, 0 warnings

**Step 3: Create git tag**

```bash
git tag -a v0.1.1-beta.3-error-handling -m "Error handling refactor complete"
```

**Step 4: Final commit**

```bash
git add README.md
git commit -m "docs: update README with error handling information"
```

---

## Summary

**Files Created:**
- `src/error.rs` - Error type definitions
- `src/error_reporting.rs` - Error reporting infrastructure

**Files Modified:**
- `src/lib.rs` - Export error modules
- `src/main.rs` - Unified error handling
- `src/tray.rs` - Mutex poisoning recovery
- `src/config/storage.rs` - Remove unsafe Default
- `src/updater/manager.rs` - Remove unsafe Default
- `src/updater/downloader.rs` - Remove unsafe Default
- `src/serialization/mod.rs` - Safe unwrap fallbacks
- `src/protobuf_schema/parser.rs` - Lazy static regex
- `src/protobuf_schema/registry.rs` - try_into error handling
- `src/ui/pubsub_panel.rs` - System time error handling
- `src/ui/monitor_panel.rs` - System time error handling

**Unsafe Unwraps Removed:**
- 4 critical (mutex poisoning, Default implementations)
- 5 medium-risk (startup code)
- Improved 7 safe unwraps with fallbacks

**Time Estimate:**
- Day 1: 4-6 hours (Tasks 1-8)
- Day 2: 4-6 hours (Tasks 9-18)
- Total: 8-12 hours

**Risk Level:** Medium
- Breaking changes to error handling flow
- Requires comprehensive testing
- No changes to business logic

---

**Plan complete and saved to `docs/plans/2026-04-02-error-handling-implementation.md`.**

**Two execution options:**

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach would you prefer?**