# Redis Desktop Transparent Titlebar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 macOS 标题栏区域视觉上融入应用主题，同时保留原生窗口按钮和行为。

**Architecture:** 在窗口层启用透明标题栏与 full-size content view，让 WebView 内容延伸到标题栏区域；在应用根布局中增加一条 macOS 专用顶栏，负责承接主题背景和窗口拖拽交互。现有业务面板和主题桥接逻辑保持不变。

**Tech Stack:** Rust, Dioxus Desktop, tao macOS window extensions, theme tokens

---

### Task 1: 调整 macOS 窗口配置

**Files:**
- Modify: `src/main.rs`

**Steps:**
1. 引入 `WindowBuilderExtMacOS`。
2. 在 macOS 上开启透明标题栏、隐藏标题文本和 full-size content view。
3. 保留当前窗口尺寸和主题初始化逻辑。

### Task 2: 增加主题顶栏

**Files:**
- Modify: `src/ui/app.rs`

**Steps:**
1. 为 macOS 顶栏定义固定高度和 traffic lights 预留区。
2. 在根布局顶部增加主题顶栏背景层。
3. 让顶栏支持拖拽窗口和双击最大化。
4. 保持现有主内容区布局不变，只把内容区下移到顶栏之后。

### Task 3: 验证

**Files:**
- Verify: `src/main.rs`
- Verify: `src/ui/app.rs`

**Steps:**
1. 运行 `cargo check`。
2. 确认没有因平台扩展或事件闭包引入编译错误。
3. 记录未做的人工视觉验证范围。
