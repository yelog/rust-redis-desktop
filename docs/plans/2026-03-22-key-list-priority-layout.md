# Redis Engine Key-List-First Layout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 `Data` 页默认优先展示 key 列表，详情区和服务器概览改为按需展开。

**Architecture:** 调整 `App` 中 `Data` 页的布局状态管理，把当前固定上下分栏改为“列表主区 + 可折叠详情区 + 状态条”。`KeyBrowser` 改为填充父容器，不再依赖固定像素高度；`ServerInfoPanel` 只在用户主动请求时渲染。

**Tech Stack:** Rust, Dioxus, Freya layout, theme tokens

---

### Task 1: 设计状态与文档

**Files:**
- Create: `docs/plans/2026-03-22-key-list-priority-layout-design.md`
- Create: `docs/plans/2026-03-22-key-list-priority-layout.md`

**Steps:**
1. 固定三种状态：折叠、编辑器、概览。
2. 明确默认态和切换规则。
3. 确认详情区高度约束和状态条职责。

### Task 2: 重构 Data 页布局

**Files:**
- Modify: `src/ui/app.rs`

**Steps:**
1. 新增详情区显示状态信号。
2. 将 `KeyBrowser` 区改为主区 `flex: 1`。
3. 在未展开详情时，仅渲染底部状态条。
4. 在展开详情时，渲染可拖动分隔条和详情区。

### Task 3: 调整 KeyBrowser 容器模型

**Files:**
- Modify: `src/ui/key_browser.rs`

**Steps:**
1. 移除对固定像素高度的强依赖。
2. 改为填满父容器。
3. 保持现有搜索、筛选、分页和表格逻辑不变。

### Task 4: 接入服务器概览入口

**Files:**
- Modify: `src/ui/app.rs`
- Verify: `src/ui/server_info.rs`

**Steps:**
1. 把 `ServerInfoPanel` 改成显式入口打开。
2. 在状态条中放置“服务器概览”入口。
3. 保证从概览切回列表是无损的。

### Task 5: 验证

**Files:**
- Verify: `src/ui/app.rs`
- Verify: `src/ui/key_browser.rs`

**Steps:**
1. 运行 `cargo check`。
2. 检查编译是否通过。
3. 记录未完成的人工视觉验证项。
