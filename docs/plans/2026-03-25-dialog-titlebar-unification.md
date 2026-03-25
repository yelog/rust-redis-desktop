# Redis Desktop Dialog Titlebar Unification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将高频弹窗标题统一收敛到通用弹窗容器中，解决标题和右上角关闭按钮不对齐的问题。

**Architecture:** 扩展 `AnimatedDialog` 的能力，让它在收到 `title` 参数时负责渲染统一标题栏，再把目标弹窗从“内容区自定义标题”迁移到“容器标题栏”。这样布局约束只存在一处，后续新增弹窗也能沿用同一规则。

**Tech Stack:** Rust, Dioxus, Freya, existing dialog components

---

### Task 1: 记录设计和改动边界

**Files:**
- Create: `docs/plans/2026-03-25-dialog-titlebar-unification-design.md`
- Create: `docs/plans/2026-03-25-dialog-titlebar-unification.md`

**Steps:**
1. 记录标题区错位的根因是标题布局责任散落在各弹窗中。
2. 明确本次只处理通用标题栏和目标弹窗标题迁移，不重构各弹窗正文业务逻辑。
3. 写清验收标准，便于后续编译和视觉核对。

### Task 2: 给 AnimatedDialog 增加统一标题栏

**Files:**
- Modify: `src/ui/animated_dialog.rs`

**Steps:**
1. 新增可选 `title` 参数。
2. 当 `title` 存在时，渲染统一标题栏和正文内容区。
3. 统一标题字体、左右内边距、底部分隔和关闭按钮对齐。
4. 保持无标题旧弹窗兼容。

### Task 3: 迁移目标弹窗标题

**Files:**
- Modify: `src/ui/settings_dialog.rs`
- Modify: `src/ui/connection_form.rs`
- Modify: `src/ui/add_key_dialog.rs`
- Modify: `src/ui/memory_analysis_dialog.rs`
- Modify: `src/ui/pattern_delete_dialog.rs`
- Modify: `src/ui/batch_ttl_dialog.rs`

**Steps:**
1. 删除每个目标弹窗内容区中的 `h2/h3` 标题节点。
2. 统一通过 `AnimatedDialog { title: ... }` 传入标题文字。
3. 保持正文间距自然，不让标题迁移后出现额外空白或贴边。

### Task 4: 编译验证

**Files:**
- Verify: `src/ui/animated_dialog.rs`
- Verify: `src/ui/settings_dialog.rs`
- Verify: `src/ui/connection_form.rs`
- Verify: `src/ui/add_key_dialog.rs`
- Verify: `src/ui/memory_analysis_dialog.rs`
- Verify: `src/ui/pattern_delete_dialog.rs`
- Verify: `src/ui/batch_ttl_dialog.rs`

**Steps:**
1. 运行 `cargo check`。
2. 若遇到与本次无关的工作区已有改动报错，单独标明来源。
3. 记录本次未做的人工桌面截图级验证范围。
