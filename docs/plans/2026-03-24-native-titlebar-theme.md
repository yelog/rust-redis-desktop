# Redis Desktop Native Titlebar Theme Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 macOS 原生标题栏在应用主题切换时同步亮暗外观，并减少启动阶段的标题栏闪白。

**Architecture:** 在主题模块中补一个“主题偏好到原生窗口主题”的小映射函数，启动时用于 `WindowBuilder::with_theme`，运行时用于桌面窗口的 `set_theme`。应用内容区主题桥接脚本保持不变，只前移初始设置读取，避免窗口壳层和内容区出现明显时序差。

**Tech Stack:** Rust, Dioxus Desktop, tao window theme, app settings

---

### Task 1: 补主题到原生窗口主题的映射

**Files:**
- Modify: `src/theme/colors.rs`
- Modify: `src/theme/mod.rs`

**Steps:**
1. 新增 `preferred_window_theme` 辅助函数。
2. 让 `System` 返回 `None`，手动主题根据 `ThemeSpec.kind` 映射到 `Light/Dark`。
3. 从 `theme` 模块重新导出该函数。

### Task 2: 启动阶段初始化原生窗口主题

**Files:**
- Modify: `src/main.rs`

**Steps:**
1. 读取已保存设置，不再丢弃结果。
2. 将设置中的 `theme_preference` 映射到窗口 builder 的 `with_theme`。
3. 保持现有菜单和窗口尺寸逻辑不变。

### Task 3: 运行时同步标题栏主题

**Files:**
- Modify: `src/ui/app.rs`

**Steps:**
1. 将初始设置读取前移到 signal 初始化阶段。
2. 使用当前窗口句柄在 `theme_preference` 变化时调用 `set_theme`。
3. 保留现有页面内主题桥接逻辑，只补原生窗口同步。

### Task 4: 基础验证

**Files:**
- Verify: `src/main.rs`
- Verify: `src/ui/app.rs`
- Verify: `src/theme/colors.rs`

**Steps:**
1. 运行 `cargo check`。
2. 检查新增导入、平台类型和 effect 是否编译通过。
3. 记录未做的人工视觉验证范围。
