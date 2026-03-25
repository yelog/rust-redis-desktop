# Context Menu Instance Fix Design

**Date:** 2026-03-25

## Problem

The connection list and key browser each manage their own local context-menu state. They both broadcast a global close signal before opening a new menu, but the menu component does not safely clear the parent-owned state when a close is triggered by a version change. This allows stale menu instances to survive cross-region right clicks.

## Chosen Approach

Use per-menu instance IDs and only clear state for the matching instance.

- Store menu state as `ContextMenuState<T>` instead of raw tuples.
- Pass the menu instance ID into `ContextMenu`.
- When a close is triggered by an outside click or global version change, invoke `on_close(menu_id)`.
- In each parent, clear local state only when the current menu ID matches the closing menu ID.
- Key the rendered `ContextMenu` by its instance ID so a newly opened menu gets a fresh component lifecycle.

## Why This Fix

This preserves the existing local-state architecture while removing the race where an old menu closes after a new menu has already been opened. It also handles cross-region and same-region repeated right-clicks with the same mechanism.

## Validation

- Run `cargo check`.
- Manually verify:
  - Right-click a connection, then right-click a key: only the key menu remains.
  - Right-click different keys repeatedly: only the latest menu remains.
  - Right-click the same region repeatedly: the latest menu stays open.
