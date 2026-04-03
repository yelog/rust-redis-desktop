# App.rs Refactor Design

**Date**: 2026-04-03  
**Status**: Approved by default recommendation  
**Author**: Sisyphus  
**Phase**: Phase 3 - UI Structure Refactor  

---

## Overview

This document defines the first large-file refactor in the UI layer: splitting `src/ui/app.rs` by responsibility while preserving existing behavior. The goal is to reduce cognitive load, isolate side effects and event handlers, and make future refactors—especially `src/ui/value_viewer.rs`—safer.

This refactor is intentionally structural, not behavioral. State ownership stays in `App()`, and no UI flow, error handling policy, or persistence behavior should change.

---

## Problem Statement

`src/ui/app.rs` is currently **1685 lines** and mixes several concerns in one file:

1. Top-level app state initialization (`use_signal`, shared state maps, dialog flags)
2. Theme helpers and browser bridge script generation
3. Multiple `use_effect` / `use_future` side effects
4. Menu and keyboard integration
5. Connection lifecycle actions (select, reconnect, close, delete, save)
6. Main content rendering (empty, loading, reconnecting, connected)
7. Dialog rendering (connection form, settings, import/export, flush, update)

The file is still understandable, but it is no longer easy to modify safely. Small changes require scanning hundreds of unrelated lines, and shared closures are difficult to reason about.

---

## Refactor Goal

Split `src/ui/app.rs` into smaller modules organized by responsibility while preserving:

- existing state ownership in `App()`
- existing Dioxus signal flow
- existing dialog behavior
- existing update flow and theme synchronization
- existing Phase 1 error handling behavior

The first refactor should create cleaner boundaries, not a new app architecture.

---

## Non-Goals

This design explicitly does **not** include:

1. Converting app state to a global store or context-driven architecture
2. Refactoring `value_viewer.rs` in the same step
3. Replacing signals with reducers or another state model
4. Rewriting UI text to i18n in this pass
5. Changing app behavior, feature set, or visual layout

---

## Options Considered

### Option A: Split `app.rs` by responsibility (**Chosen**)

Create an `src/ui/app/` module directory and extract helpers into submodules for:

- state-related types and initialization helpers
- theme helpers
- side effects
- action builders / event handlers
- render helpers

**Why chosen:** lowest risk, highest maintainability gain, and minimal behavioral drift.

### Option B: Split `app.rs` by visual regions

Separate title bar, left rail, content area, dialogs, and overlays into individual render files.

**Why not chosen first:** easier to create duplicated handler logic and state scattering before behavioral seams are clean.

### Option C: Refactor `value_viewer.rs` first

Attack the largest file first (`6448` lines).

**Why not chosen first:** it has higher fan-out, likely more shared editing/view logic, and a much larger regression surface.

---

## Chosen Design

### New Module Layout

`src/ui/app.rs` becomes `src/ui/app/mod.rs`, with the following siblings:

```text
src/ui/app/
├── mod.rs
├── state.rs
├── theme.rs
├── effects.rs
├── actions.rs
└── render.rs
```

### Module Responsibilities

#### `src/ui/app/mod.rs`

Owns:

- `App()` component
- top-level signal declarations
- high-level assembly of helpers from sibling modules

Must remain the single place where state is created.

#### `src/ui/app/state.rs`

Owns:

- `Tab`
- `FormMode`
- small helper types if needed
- light state initialization helpers if extraction is worthwhile

Should not own async logic.

#### `src/ui/app/theme.rs`

Owns the existing pure theme helpers:

- `system_theme_is_dark()`
- `load_initial_settings()`
- `build_theme_palette()`
- `build_theme_bridge_script()`

These are already natural extraction seams because they do not depend on Dioxus component state.

#### `src/ui/app/effects.rs`

Owns setup helpers for:

- loading persisted connections on startup
- syncing theme bridge script
- syncing desktop window theme
- keyboard shortcut listener
- update polling / manual update check loop
- system theme change listener

These should be extracted as functions that are called from `App()` and internally register `use_effect` / `use_future`.

#### `src/ui/app/actions.rs`

Owns closure builders / helper functions for:

- save connection
- select connection
- reconnect connection
- close connection
- reorder connection
- delete connection confirm flow
- settings save flow

This module should focus on reducing inline closure size in `App()`.

#### `src/ui/app/render.rs`

Owns render helpers for:

- title bar rendering
- empty state rendering
- connecting / reconnecting / error placeholders
- connected tab shell
- dialog composition blocks

This module should reduce the length of the main `rsx!` tree while keeping the data passed in explicit.

---

## Data Flow Strategy

This is the key constraint of the refactor.

### State Ownership

All `use_signal(...)` calls remain in `App()` for this first pass.

That means:

- no context migration
- no central store
- no new cross-module shared state object required unless it is purely a helper container

### Passing State to Helpers

Helpers in `effects.rs`, `actions.rs`, and `render.rs` should receive only the signals and immutable values they need.

This keeps the refactor explicit and minimizes surprises around ownership and signal capture.

### Why This Matters

If state ownership is moved at the same time as file splitting, the refactor becomes architectural rather than structural. That would dramatically increase risk and make regressions harder to localize.

---

## Extraction Order

To minimize risk, extraction should happen in this order:

1. **theme.rs**
   - already pure functions
   - lowest-risk move

2. **state.rs**
   - move `Tab` / `FormMode`
   - no behavior changes

3. **effects.rs**
   - move grouped setup code one block at a time
   - verify build after every move

4. **render.rs**
   - extract connected/empty/loading UI sections into helpers

5. **actions.rs**
   - extract the largest event closures last, after render boundaries are clearer

This order deliberately avoids moving the hardest closure-heavy logic too early.

---

## Concrete Extraction Seams Found in Current File

### Pure helper seam

Lines near the top of `app.rs` are strong extraction candidates:

- `system_theme_is_dark()`
- `load_initial_settings()`
- `build_theme_palette()`
- `build_theme_bridge_script()`

### Side-effect seam

The following blocks are grouped and separable:

- startup load of connections
- theme bridge eval
- desktop theme sync
- keyboard event future
- update check loop
- system theme media query listener

### Render seam

The `rsx!` tree has several visually distinct sections:

- macOS title bar
- left rail area
- connection state placeholders
- connected tab shell
- dialog stack
- update dialog block
- toast/image preview tail

These can be extracted without changing state structure.

### Handler seam

The heaviest inline closures today are attached to:

- `LeftRail`
- `ConnectionForm`
- delete dialog confirm
- import/export dialog callbacks
- update dialog callbacks

These should become named builders or helper functions once render extraction is in place.

---

## Error Handling Impact

Phase 1 introduced a clearer error model. This refactor should not bypass it.

Requirements:

- keep existing `ConfigStorage` / `UpdateManager` result handling intact
- do not introduce unwrap-based shortcuts during extraction
- preserve existing non-fatal behaviors (for example, update check failures still only surface through current UI/toast behavior)

---

## Testing Strategy

Because this is a structural refactor, the primary goal is behavioral parity.

Required validation:

1. `cargo check`
2. `cargo test --lib`
3. `cargo build --release`

Recommended manual checks after implementation:

1. App starts normally
2. Theme switching still works
3. Settings dialog opens and saves
4. Connection selection still updates left rail and content area correctly
5. Reconnect / delete / import / export flows still work
6. Update dialog still renders when pending update state exists

---

## Risks

### Risk 1: Closure capture breakage

Large Dioxus closures often capture many signals. Extracting them too aggressively can create borrow/move errors.

**Mitigation:** extract pure helpers and effect helpers first; leave closure-heavy action extraction for later.

### Risk 2: Render helper over-abstraction

If render helpers take too many arguments, readability may get worse instead of better.

**Mitigation:** prefer extracting whole UI regions, not tiny fragments.

### Risk 3: Mixed structural + architectural changes

Trying to introduce a new state model at the same time would make the change too large.

**Mitigation:** keep all signal ownership in `App()` for this phase.

---

## Success Criteria

This refactor is successful if:

1. `src/ui/app.rs` is converted into a directory module with smaller files
2. No user-visible behavior changes
3. `App()` becomes meaningfully shorter and easier to scan
4. Theme helpers and side effects are no longer mixed with large UI blocks
5. Build, tests, and release build all continue to pass

---

## Follow-up Work

Once `app.rs` is split cleanly, the next refactor should target `src/ui/value_viewer.rs` using the same principle:

- first identify pure helpers and render seams
- then split by responsibility or data-type view cluster
- avoid simultaneous architecture changes

This Phase 3 `app.rs` refactor is meant to be the lower-risk first step that prepares the codebase for that larger extraction.
