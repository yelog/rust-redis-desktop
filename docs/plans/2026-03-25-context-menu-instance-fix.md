# Context Menu Instance Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make context menus close deterministically across the connection list and key browser so only the newest right-click menu remains visible.

**Architecture:** Keep the existing distributed menu ownership in each UI region, but store menu state as instance-aware `ContextMenuState<T>`. Route close events through the menu instance ID so stale menus cannot clear newer state.

**Tech Stack:** Rust, Dioxus desktop UI, async tasks with Tokio

---

### Task 1: Convert menu state to instance-aware data

**Files:**
- Modify: `src/ui/context_menu.rs`
- Modify: `src/ui/left_rail.rs`
- Modify: `src/ui/key_browser.rs`
- Modify: `src/ui/lazy_tree_node.rs`

**Step 1: Replace tuple-based menu state**

Use `ContextMenuState<T>` in the connection list and key browser so each open action gets a fresh unique ID.

**Step 2: Route close callbacks by menu ID**

Change `ContextMenu` close callbacks to pass `menu_id`, and make parents clear state only if the current state matches that ID.

**Step 3: Key the rendered menu by instance ID**

Render each `ContextMenu` with a unique key so a new menu gets a fresh lifecycle and watcher task.

### Task 2: Preserve existing user actions

**Files:**
- Modify: `src/ui/left_rail.rs`
- Modify: `src/ui/key_browser.rs`

**Step 1: Keep existing action handlers**

Menu actions like edit, reconnect, delete, TTL, and export should still close the current menu and trigger the same behavior as before.

**Step 2: Keep global close broadcasts**

Continue calling `close_all_context_menus()` before opening a new menu so different regions still coordinate.

### Task 3: Validate the fix

**Files:**
- Modify: none

**Step 1: Run compile validation**

Run: `cargo check`

Expected: build succeeds without type errors.

**Step 2: Manual interaction validation**

Verify:
- connection menu closes when opening a key menu
- repeated right-clicks only leave the latest menu visible
