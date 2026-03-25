# Redis Desktop Memory Analysis Dialog Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 优化内存分析弹窗布局，修复筛选区错乱问题，并补一个清晰可见的右上角关闭叉叉按钮。

**Architecture:** 复用现有 `AnimatedDialog` 容器，只在 `MemoryAnalysisDialog` 内重排内容结构，同时增强通用关闭按钮的视觉样式。扫描、排序、结果选择等数据逻辑保持不变，避免把 UI 问题扩散到 Redis 数据流。

**Tech Stack:** Rust, Dioxus, Freya, existing dialog and theme components

---

### Task 1: 记录设计并锁定改动边界

**Files:**
- Create: `docs/plans/2026-03-25-memory-analysis-dialog-design.md`
- Create: `docs/plans/2026-03-25-memory-analysis-dialog.md`

**Steps:**
1. 写明当前错乱来源是头部、表单和主按钮横向耦合过深。
2. 明确只调整 `AnimatedDialog` 和 `MemoryAnalysisDialog`，不改扫描数据逻辑。
3. 记录验收标准，便于后续编译和视觉回归。

### Task 2: 重排内存分析弹窗结构

**Files:**
- Modify: `src/ui/memory_analysis_dialog.rs`

**Steps:**
1. 新增标题区和说明文案，给右上角关闭按钮留出空间。
2. 将筛选区改为卡片式布局，输入项与主操作按钮拆分为两个区块。
3. 优化状态提示、结果摘要条和表格行间距，保持点击选中行为不变。

### Task 3: 强化通用关闭按钮

**Files:**
- Modify: `src/ui/animated_dialog.rs`

**Steps:**
1. 保留现有焦点与退出动画逻辑。
2. 增大右上角关闭按钮的点击热区，补背景、边框和悬浮感。
3. 为 `IconX` 显式传入颜色，避免图标在不同主题下不可见。

### Task 4: 编译验证

**Files:**
- Verify: `src/ui/memory_analysis_dialog.rs`
- Verify: `src/ui/animated_dialog.rs`

**Steps:**
1. 运行 `cargo check`。
2. 如有样式字符串或类型错误，修正到通过为止。
3. 记录本次未做的人工窗口级视觉验证范围。
