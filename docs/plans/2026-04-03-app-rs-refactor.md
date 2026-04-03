# App.rs Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `src/ui/app.rs` into smaller responsibility-based modules without changing runtime behavior.

**Architecture:** Keep all Dioxus state ownership inside `App()` during this first pass. Extract pure helpers first, then effect setup code, then render helpers, and only then move closure-heavy action code. The refactor is structural, not architectural.

**Tech Stack:** Rust, Dioxus, Freya desktop runtime, existing `ConfigStorage`, `UpdateManager`, `ConnectionPool` app flow.

---

### Task 1: Create the app module directory shell

**Files:**
- Create: `src/ui/app/mod.rs`
- Create: `src/ui/app/state.rs`
- Create: `src/ui/app/theme.rs`
- Create: `src/ui/app/effects.rs`
- Create: `src/ui/app/render.rs`
- Create: `src/ui/app/actions.rs`
- Modify: `src/ui/mod.rs`
- Delete later: `src/ui/app.rs` (only after module replacement is stable)

**Step 1: Create the directory and empty module files**

Create these exact files with module placeholders:

```rust
// src/ui/app/mod.rs
mod actions;
mod effects;
mod render;
mod state;
mod theme;

pub use self::state::{FormMode, Tab};
```

```rust
// src/ui/app/state.rs
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
    Edit(crate::connection::ConnectionConfig),
}
```

```rust
// src/ui/app/theme.rs
// placeholder
```

```rust
// src/ui/app/effects.rs
// placeholder
```

```rust
// src/ui/app/render.rs
// placeholder
```

```rust
// src/ui/app/actions.rs
// placeholder
```

**Step 2: Point `src/ui/mod.rs` at the new module path**

Ensure `src/ui/mod.rs` continues to export `App` from the module system using the same public API.

**Step 3: Run build to confirm skeleton wiring**

Run: `cargo check`

Expected: It fails because `App` is not implemented yet, but module resolution should point at `src/ui/app/mod.rs` rather than the old flat file.

**Step 4: Commit**

```bash
git add src/ui/app src/ui/mod.rs
git commit -m "refactor: create modular app/ directory shell"
```

---

### Task 2: Move state types into `state.rs`

**Files:**
- Modify: `src/ui/app/state.rs`
- Modify: `src/ui/app/mod.rs`
- Modify: `src/ui/app.rs` content moved into `src/ui/app/mod.rs`

**Step 1: Copy `Tab` and `FormMode` exactly from old `src/ui/app.rs`**

They currently live near the top of the file. Move them into `src/ui/app/state.rs` and remove the duplicates from `mod.rs`.

**Step 2: Import them in `mod.rs`**

```rust
use self::state::{FormMode, Tab};
```

**Step 3: Run build**

Run: `cargo check`

Expected: Pass or fail only on unrelated placeholder sections, but no duplicate type definitions.

**Step 4: Commit**

```bash
git add src/ui/app/state.rs src/ui/app/mod.rs
git commit -m "refactor: move app state enums into state.rs"
```

---

### Task 3: Move pure theme helpers into `theme.rs`

**Files:**
- Modify: `src/ui/app/theme.rs`
- Modify: `src/ui/app/mod.rs`

**Step 1: Move these exact functions from old app file into `theme.rs`**

- `system_theme_is_dark()`
- `load_initial_settings()`
- `build_theme_palette()`
- `build_theme_bridge_script()`

Preserve existing imports and signatures.

**Step 2: Export them from `theme.rs`**

Use `pub(crate)` or `pub(super)` visibility as needed, but keep usage minimal.

**Step 3: Update `mod.rs` imports**

Example:

```rust
use self::theme::{
    build_theme_bridge_script, build_theme_palette, load_initial_settings, system_theme_is_dark,
};
```

**Step 4: Run build**

Run: `cargo check`

Expected: The app still compiles with the helpers now living in `theme.rs`.

**Step 5: Commit**

```bash
git add src/ui/app/theme.rs src/ui/app/mod.rs
git commit -m "refactor: extract app theme helpers into theme.rs"
```

---

### Task 4: Extract startup/effect setup helpers into `effects.rs`

**Files:**
- Modify: `src/ui/app/effects.rs`
- Modify: `src/ui/app/mod.rs`

**Step 1: Group effect logic into callable setup helpers**

Create helpers with narrow responsibility, for example:

```rust
pub(crate) fn use_load_saved_connections(/* exact signals */) {
    use_effect(move || { /* existing block */ });
}

pub(crate) fn use_theme_bridge(/* exact signals */) {
    use_effect(move || { /* existing block */ });
}

pub(crate) fn use_window_theme_sync(/* exact args */) {
    use_effect(move || { /* existing block */ });
}

pub(crate) fn use_keyboard_shortcuts(/* exact args */) {
    use_future(move || async move { /* existing block */ });
}

pub(crate) fn use_manual_update_check(/* exact args */) {
    use_future(move || async move { /* existing block */ });
}

pub(crate) fn use_system_theme_listener(/* exact args */) {
    use_future(move || async move { /* existing block */ });
}
```

**Step 2: Replace inline blocks in `mod.rs` with helper calls**

Do not rewrite logic. Move it as-is first.

**Step 3: Run build**

Run: `cargo check`

Expected: Pass.

**Step 4: Run targeted tests**

Run: `cargo test --lib`

Expected: Pass.

**Step 5: Commit**

```bash
git add src/ui/app/effects.rs src/ui/app/mod.rs
git commit -m "refactor: extract app side-effect setup into effects.rs"
```

---

### Task 5: Extract render helpers for top-level placeholders and shell

**Files:**
- Modify: `src/ui/app/render.rs`
- Modify: `src/ui/app/mod.rs`

**Step 1: Extract loading/error/empty-state render helpers**

Start with the easiest large repeated UI sections:

- reconnecting placeholder
- connection error placeholder
- connection loading placeholder
- no-connection empty state

These should become plain helper functions returning `Element` or `rsx!` fragments.

**Step 2: Extract title bar helper**

Move the macOS title bar UI block into a helper.

**Step 3: Extract connected tab shell helper**

Move the tab-strip + content switch shell into a helper, but keep callback behavior unchanged.

**Step 4: Replace inline `rsx!` sections in `mod.rs`**

The goal is to shorten the main render tree significantly without changing logic.

**Step 5: Run build**

Run: `cargo check`

Expected: Pass.

**Step 6: Commit**

```bash
git add src/ui/app/render.rs src/ui/app/mod.rs
git commit -m "refactor: extract app render helpers into render.rs"
```

---

### Task 6: Extract connection form and dialog orchestration helpers

**Files:**
- Modify: `src/ui/app/render.rs`
- Modify: `src/ui/app/mod.rs`

**Step 1: Move dialog rendering blocks into helpers**

Extract helpers for:

- connection form
- settings dialog
- delete connection dialog
- flush dialog
- import dialog
- export/import connections dialog
- update dialog

**Step 2: Keep all state ownership in `mod.rs`**

Helpers should receive the signals and immutable values they need. Do not move signal creation.

**Step 3: Run build and tests**

Run:

```bash
cargo check
cargo test --lib
```

Expected: Pass.

**Step 4: Commit**

```bash
git add src/ui/app/render.rs src/ui/app/mod.rs
git commit -m "refactor: extract app dialog composition into render helpers"
```

---

### Task 7: Extract the heaviest action closures into `actions.rs`

**Files:**
- Modify: `src/ui/app/actions.rs`
- Modify: `src/ui/app/mod.rs`

**Step 1: Start with the lowest-risk large closures**

Extract builders for:

- save settings
- delete connection confirm
- reorder connection

**Step 2: Then extract connection lifecycle closures**

Extract builders for:

- select connection
- reconnect connection
- close connection
- edit connection
- save connection

Each builder should return a closure and receive the exact signals it needs.

**Step 3: Do not over-generalize**

If a builder takes too many arguments, keep it local. Prefer a smaller win over bad abstraction.

**Step 4: Run build**

Run: `cargo check`

Expected: Pass.

**Step 5: Run tests**

Run: `cargo test --lib`

Expected: Pass.

**Step 6: Commit**

```bash
git add src/ui/app/actions.rs src/ui/app/mod.rs
git commit -m "refactor: extract app action builders into actions.rs"
```

---

### Task 8: Remove the old flat `src/ui/app.rs`

**Files:**
- Delete: `src/ui/app.rs`

**Step 1: Verify module resolution fully uses `src/ui/app/mod.rs`**

Run: `cargo check`

Expected: Pass without the old flat file.

**Step 2: Delete the old file**

Only do this once all code is living in the directory module.

**Step 3: Run build again**

Run: `cargo check`

Expected: Pass.

**Step 4: Commit**

```bash
git add src/ui/app src/ui/mod.rs
git rm src/ui/app.rs
git commit -m "refactor: replace flat app.rs with modular app/ directory"
```

---

### Task 9: Final verification and manual QA

**Files:**
- No new files required unless test notes are documented

**Step 1: Run full checks**

Run:

```bash
cargo check
cargo test --lib
cargo build --release
```

Expected: All pass.

**Step 2: Manual smoke test**

Run: `cargo run`

Verify:

- app launches
- existing connections load
- selecting a connection still renders the correct tab shell
- settings dialog still opens
- update UI still renders if pending update state is set

**Step 3: Commit final verification notes if needed**

If you add any notes file, commit it separately. Otherwise no extra commit is required.

---

## Notes for Execution

- Keep the refactor behavioral-neutral.
- Prefer moving exact code first, then simplifying.
- Do not combine this with `value_viewer.rs` refactoring.
- Do not introduce a new global app store in this pass.
- If an extraction causes excessive closure argument sprawl, stop and keep that block in `mod.rs` for now.

---

Plan complete and saved to `docs/plans/2026-04-03-app-rs-refactor.md`.

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration
2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
