# Value Viewer Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split `src/ui/value_viewer.rs` into a directory module by extracting leaf viewers, pure helpers, and the async data-loading layer while keeping `ValueViewer` behavior unchanged.

**Architecture:** Convert the flat file into `src/ui/value_viewer/mod.rs`, move low-risk leaf components and pure helper functions into sibling modules first, and defer the tightly coupled string/binary rendering branch to a later pass. State ownership remains in `ValueViewer` during this phase.

**Tech Stack:** Rust, Dioxus, existing Redis connection pool APIs, existing serialization helpers, existing theme constants.

---

### Task 1: Create the `value_viewer/` module shell

**Files:**
- Create: `src/ui/value_viewer/mod.rs`
- Create: `src/ui/value_viewer/image_preview.rs`
- Create: `src/ui/value_viewer/bitmap_viewer.rs`
- Create: `src/ui/value_viewer/protobuf_viewer.rs`
- Create: `src/ui/value_viewer/styles.rs`
- Create: `src/ui/value_viewer/formatters.rs`
- Create: `src/ui/value_viewer/data_loader.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Move the flat file into a directory module**

Move:

```text
src/ui/value_viewer.rs -> src/ui/value_viewer/mod.rs
```

Do not change logic yet.

**Step 2: Create empty sibling files**

Create these files as valid Rust modules:

```rust
// image_preview.rs
```

```rust
// bitmap_viewer.rs
```

```rust
// protobuf_viewer.rs
```

```rust
// styles.rs
```

```rust
// formatters.rs
```

```rust
// data_loader.rs
```

**Step 3: Add module declarations to `mod.rs`**

At the top of `src/ui/value_viewer/mod.rs` add:

```rust
mod bitmap_viewer;
mod data_loader;
mod formatters;
mod image_preview;
mod protobuf_viewer;
mod styles;
```

**Step 4: Keep public API unchanged**

Ensure `pub fn ValueViewer(...) -> Element` remains publicly available from the same `pub use value_viewer::*;` export path in `src/ui/mod.rs`.

**Step 5: Run build**

Run: `cargo check`

Expected: Pass or only fail if imports are not yet wired correctly.

**Step 6: Commit**

```bash
git add src/ui/value_viewer src/ui/mod.rs
git commit -m "refactor: create modular value_viewer/ directory shell"
```

---

### Task 2: Extract image preview module

**Files:**
- Modify: `src/ui/value_viewer/mod.rs`
- Modify: `src/ui/value_viewer/image_preview.rs`

**Step 1: Move these exact items into `image_preview.rs`**

- `PreviewImageData`
- `PREVIEW_IMAGE`
- `ImagePreview`

Keep the implementation unchanged except for imports.

**Step 2: Export them from `image_preview.rs`**

Use visibility that keeps `ValueViewer` able to reference them:

```rust
pub struct PreviewImageData { ... }
pub static PREVIEW_IMAGE: GlobalSignal<Option<PreviewImageData>> = ...;
#[component]
pub fn ImagePreview() -> Element { ... }
```

**Step 3: Import from `mod.rs`**

In `mod.rs`:

```rust
use self::image_preview::{ImagePreview, PreviewImageData, PREVIEW_IMAGE};
```

**Step 4: Run build**

Run: `cargo check`

Expected: Pass.

**Step 5: Commit**

```bash
git add src/ui/value_viewer/image_preview.rs src/ui/value_viewer/mod.rs
git commit -m "refactor: extract image preview from value viewer"
```

---

### Task 3: Extract `BitmapViewer`

**Files:**
- Modify: `src/ui/value_viewer/mod.rs`
- Modify: `src/ui/value_viewer/bitmap_viewer.rs`

**Step 1: Move `BitmapViewer` into `bitmap_viewer.rs`**

Keep:

```rust
#[component]
pub fn BitmapViewer(...) -> Element { ... }
```

with minimal import fixes.

**Step 2: Import it back into `mod.rs`**

```rust
use self::bitmap_viewer::BitmapViewer;
```

**Step 3: Run build**

Run: `cargo check`

Expected: Pass.

**Step 4: Commit**

```bash
git add src/ui/value_viewer/bitmap_viewer.rs src/ui/value_viewer/mod.rs
git commit -m "refactor: extract bitmap viewer from value viewer"
```

---

### Task 4: Extract `ProtobufViewer`

**Files:**
- Modify: `src/ui/value_viewer/mod.rs`
- Modify: `src/ui/value_viewer/protobuf_viewer.rs`

**Step 1: Move `ProtobufViewer` into `protobuf_viewer.rs`**

Keep:

```rust
#[component]
fn ProtobufViewer(data: Vec<u8>) -> Element { ... }
```

You may make it `pub(super)` if that is the narrowest correct visibility.

**Step 2: Import it from `mod.rs`**

```rust
use self::protobuf_viewer::ProtobufViewer;
```

**Step 3: Run build**

Run: `cargo check`

Expected: Pass.

**Step 4: Commit**

```bash
git add src/ui/value_viewer/protobuf_viewer.rs src/ui/value_viewer/mod.rs
git commit -m "refactor: extract protobuf viewer from value viewer"
```

---

### Task 5: Extract style helpers into `styles.rs`

**Files:**
- Modify: `src/ui/value_viewer/mod.rs`
- Modify: `src/ui/value_viewer/styles.rs`

**Step 1: Move pure style helpers into `styles.rs`**

Move these exact functions:

- `secondary_action_button_style`
- `primary_action_button_style`
- `destructive_action_button_style`
- `data_section_toolbar_style`
- `data_section_controls_style`
- `data_section_count_style`
- `status_banner_style`
- `data_table_header_row_style`
- `data_table_header_cell_style`
- `compact_icon_action_button_style`
- `image_preview_button_style`
- `image_preview_info_chip_style`
- `overlay_modal_keyframes`
- `overlay_modal_backdrop_style`
- `overlay_modal_surface_style`
- `overlay_modal_title_style`
- `overlay_modal_body_style`
- `overlay_modal_actions_style`

**Step 2: Import them into `mod.rs`**

```rust
use self::styles::{ ... };
```

**Step 3: Run build**

Run: `cargo check`

Expected: Pass.

**Step 4: Commit**

```bash
git add src/ui/value_viewer/styles.rs src/ui/value_viewer/mod.rs
git commit -m "refactor: extract value viewer style helpers"
```

---

### Task 6: Extract formatting/data helper functions into `formatters.rs`

**Files:**
- Modify: `src/ui/value_viewer/mod.rs`
- Modify: `src/ui/value_viewer/formatters.rs`

**Step 1: Move these pure helpers**

- `base64_decode`
- `is_binary_data`
- `format_bytes`
- `detect_image_format`
- `copy_value_to_clipboard`
- `sorted_hash_entries`
- `format_memory_usage`
- `format_ttl_label`
- `value_metric_label`

**Step 2: Import them back into `mod.rs`**

```rust
use self::formatters::{ ... };
```

**Step 3: Run build and tests**

Run:

```bash
cargo check
cargo test --lib
```

Expected: Pass.

**Step 4: Commit**

```bash
git add src/ui/value_viewer/formatters.rs src/ui/value_viewer/mod.rs
git commit -m "refactor: extract value viewer formatting helpers"
```

---

### Task 7: Extract async loading functions into `data_loader.rs`

**Files:**
- Modify: `src/ui/value_viewer/mod.rs`
- Modify: `src/ui/value_viewer/data_loader.rs`

**Step 1: Move the async helper layer without redesigning signatures**

Move:

- `load_key_data`
- `load_more_hash_server`
- `load_more_list_server`
- `load_more_set_server`
- `load_more_zset_server`
- `search_hash_server`
- `search_set_server`
- `search_zset_server`

If helper dependencies must also move, move them with the smallest possible scope.

**Step 2: Keep function signatures unchanged initially**

Do **not** redesign them during this pass, even if they have many arguments.

**Step 3: Import into `mod.rs`**

```rust
use self::data_loader::{ ... };
```

**Step 4: Run build and tests**

Run:

```bash
cargo check
cargo test --lib
```

Expected: Pass.

**Step 5: Commit**

```bash
git add src/ui/value_viewer/data_loader.rs src/ui/value_viewer/mod.rs
git commit -m "refactor: extract value viewer async data loader layer"
```

---

### Task 8: Final verification of pass one

**Files:**
- No new files required

**Step 1: Run full validation**

Run:

```bash
cargo check
cargo test --lib
cargo build --release
```

Expected: All pass.

**Step 2: Manual smoke test**

Run: `cargo run`

Check at minimum:

- selecting a key still loads the viewer
- image preview still opens and closes
- protobuf viewer still renders/imports schema
- bitmap viewer still works
- hash/list/set/zset search and pagination still respond

**Step 3: Commit only if any final fix was needed**

If no additional fixes were necessary, no extra commit is required.

---

## Notes for Execution

- Keep `ValueViewer` state ownership unchanged.
- Do not split per-type panels in this pass.
- Do not touch the string/binary dispatcher except for import rewiring.
- Favor extraction with minimal code movement over cleanup.
- If an extracted helper becomes hard to wire, stop and keep it in `mod.rs` for now.

---

Plan complete and saved to `docs/plans/2026-04-03-value-viewer-refactor.md`.

Two execution options:

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration
2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
