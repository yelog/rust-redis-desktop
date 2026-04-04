# Value Viewer Refactor Design

**Date**: 2026-04-03  
**Status**: Approved by default recommendation  
**Author**: Sisyphus  
**Phase**: Phase 3 - Value Viewer Structural Refactor  

---

## Overview

This document defines the first refactor pass for `src/ui/value_viewer.rs`, a **6448-line** UI file. The goal is to reduce file size and improve maintainability without destabilizing the most tightly coupled branch of the component.

The chosen strategy is intentionally conservative: split out leaf components, pure formatting/style helpers, and async data-loading functions first, while keeping the main `ValueViewer` state graph and the high-risk string/binary rendering path intact.

---

## Problem Statement

`src/ui/value_viewer.rs` currently contains multiple categories of logic in a single file:

1. A global image preview signal and modal component
2. Binary formatting and image/serialization detection helpers
3. Styling helpers for toolbars, buttons, tables, overlays, and status banners
4. Async Redis data loading functions for hash/list/set/zset/stream pagination and search
5. The monolithic `ValueViewer` component itself
6. Embedded specialized viewers (`BitmapViewer`, `ProtobufViewer`)

Even before touching the central `ValueViewer` body, the file has several self-contained pieces that can be safely extracted. Keeping all of them inline makes the main viewer harder to navigate and obscures the real coupling hotspots.

---

## Main Design Goal

Reduce the size and cognitive load of `value_viewer.rs` while preserving:

- the current `ValueViewer` public API
- current Dioxus signal ownership inside `ValueViewer`
- current async loading behavior
- current binary/serialization behavior
- current image preview behavior

This pass is structural only. No user-visible behavior should change.

---

## Non-Goals

This design explicitly does **not** include:

1. Splitting the entire `ValueViewer` by data type in one pass
2. Reworking the string/binary/serialization branch
3. Introducing a new state model or context-based architecture
4. Changing Redis pagination/search behavior
5. Changing the external `ValueViewer(connection_pool, connection_version, selected_key, on_refresh)` interface

---

## Internal Structure Map

Direct code inspection and exploration found these major regions:

- `ImagePreview` + preview signal (`45–221`)
- binary/format helpers (`223–643`)
- async Redis data loaders and pagination/search helpers (`665–1292`)
- monolithic `ValueViewer` (`1295–6012`)
- embedded `BitmapViewer` (`6015`)
- embedded `ProtobufViewer` (`6293`)

Inside `ValueViewer`, the strongest coupling is concentrated in:

- the large signal graph initialized at the top
- the string/binary/serialization branch
- repeated `load_key_data(...)` calls with many signal parameters
- modal and editing state shared across multiple value types

---

## Options Considered

### Option A: Conservative split (**Chosen**)

Extract only:

- `ImagePreview`
- `BitmapViewer`
- `ProtobufViewer`
- style helpers
- formatting helpers
- async data loader helpers

Keep the main `ValueViewer` render branches in place.

**Why chosen:** lowest risk, immediate file-size reduction, and establishes a directory module structure for later per-type extraction.

### Option B: Medium split

Do Option A, then also extract:

- `hash_panel.rs`
- `list_panel.rs`
- `set_panel.rs`
- `zset_panel.rs`
- `stream_panel.rs`

**Why not chosen first:** too much signal/closure plumbing for the first pass, increasing regression risk.

### Option C: Full type-based split

Immediately split the whole file by data type, including the string/binary path.

**Why not chosen first:** the string/binary branch is the highest-coupling part of the file and should be deferred until safer module boundaries already exist.

---

## Chosen Design

### New Module Layout

The flat file becomes a directory module:

```text
src/ui/value_viewer/
├── mod.rs
├── image_preview.rs
├── bitmap_viewer.rs
├── protobuf_viewer.rs
├── styles.rs
├── formatters.rs
└── data_loader.rs
```

### Module Responsibilities

#### `src/ui/value_viewer/mod.rs`

Keeps:

- `ValueViewer`
- the current signal graph
- existing render branches
- current public interface

Acts as the orchestration layer after extraction.

#### `src/ui/value_viewer/image_preview.rs`

Moves:

- `PreviewImageData`
- `PREVIEW_IMAGE`
- `ImagePreview`

This is a strong first seam because it is already a leaf component with a dedicated global signal.

#### `src/ui/value_viewer/bitmap_viewer.rs`

Moves:

- `BitmapViewer`

This viewer is already a specialized leaf component with limited dependencies.

#### `src/ui/value_viewer/protobuf_viewer.rs`

Moves:

- `ProtobufViewer`

This is another self-contained leaf that can leave the main file with very low risk.

#### `src/ui/value_viewer/styles.rs`

Moves pure UI styling helpers:

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

These functions are pure and are excellent low-risk extraction candidates.

#### `src/ui/value_viewer/formatters.rs`

Moves pure data formatting/helpers:

- `base64_decode`
- `is_binary_data`
- `format_bytes`
- `detect_image_format`
- `copy_value_to_clipboard`
- `sorted_hash_entries`
- `format_memory_usage`
- `format_ttl_label`
- `value_metric_label`

These functions reduce noise in `mod.rs` and help clarify what logic is actually stateful.

#### `src/ui/value_viewer/data_loader.rs`

Moves async Redis access helpers, keeping signatures unchanged initially:

- `load_key_data`
- `load_more_hash_server`
- `load_more_list_server`
- `load_more_set_server`
- `load_more_zset_server`
- `search_hash_server`
- `search_set_server`
- `search_zset_server`

This module is intentionally allowed to be “ugly but stable” in the first pass. The purpose is separation, not redesign.

---

## Data Flow Strategy

### State Ownership

All `use_signal(...)` state remains in `ValueViewer` during this pass.

That means:

- no context migration
- no store extraction
- no grouped state struct yet

The goal is to separate leaf modules and helper layers before touching the central state machine.

### Loader Interfaces

`data_loader.rs` should keep the current function signatures initially, even if they are verbose. This preserves behavior and makes the extraction easy to verify.

### Global Image Preview

`PREVIEW_IMAGE` must move together with `ImagePreview`. Splitting those apart would make the extraction more confusing and would increase the chance of broken imports.

---

## Extraction Order

To minimize risk, extraction should happen in this order:

1. `image_preview.rs`
2. `bitmap_viewer.rs`
3. `protobuf_viewer.rs`
4. `styles.rs`
5. `formatters.rs`
6. `data_loader.rs`
7. convert flat file into `value_viewer/mod.rs`

This ordering starts with leaf modules, then pure helpers, then async helper functions. The central `ValueViewer` body remains in place throughout.

---

## Why the String/Binary Branch Is Deferred

The string branch is the highest-risk area because it combines:

- binary detection
- serialization detection
- formatter switching
- preview/image logic
- JSON/Java/Protobuf/Bitmap sub-view dispatch
- reload-on-format-change effect
- clipboard/export/edit interactions

Trying to split that branch in the same pass as the helper/data-layer extraction would mix structural change with behavioral risk.

Deferring it creates a safer second pass once the file already has cleaner boundaries.

---

## Risks

### Risk 1: Import churn

Extracting several helper modules will temporarily increase import count and make `mod.rs` look noisier.

**Mitigation:** accept this in pass one. Clean import grouping later.

### Risk 2: Overly long data-loader function signatures

These functions already take many signals and values.

**Mitigation:** keep signatures unchanged in pass one. Do not redesign them during extraction.

### Risk 3: Hidden coupling between helpers and main component

Some formatting/style helpers may reference theme constants or shared types in ways that are easy to miss.

**Mitigation:** extract in small verified batches and run build/tests after each stage.

---

## Testing Strategy

This is a behavioral-parity refactor. Validation should focus on ensuring nothing changed.

Required validation:

1. `cargo check`
2. `cargo test --lib`
3. `cargo build --release`

Recommended manual checks after implementation:

1. Key selection still loads values
2. Image preview still opens/closes and saves correctly
3. Bitmap viewer still displays and updates bits
4. Protobuf viewer still imports schema and renders output
5. Pagination and search still work for hash/list/set/zset

---

## Success Criteria

This refactor is successful if:

1. `src/ui/value_viewer.rs` becomes a directory module
2. Leaf viewers and helper layers are extracted cleanly
3. The `ValueViewer` public API remains unchanged
4. Binary/image/protobuf functionality still behaves the same
5. Build, tests, and release build all pass

---

## Planned Follow-up After This Pass

Once this conservative pass is stable, the next pass should extract per-type panels:

- `hash_panel.rs`
- `list_panel.rs`
- `set_panel.rs`
- `zset_panel.rs`
- `stream_panel.rs`

Only after those boundaries exist should the string/binary branch be split into its own panel/dispatcher.
