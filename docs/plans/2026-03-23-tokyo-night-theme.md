# Tokyo Night Theme Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为当前应用新增可手动选择的 `Tokyo Night` 主题，并将主题系统重构为可扩展模型。

**Architecture:** 将“主题偏好”和“主题身份”拆分，引入 `ThemeSpec` 统一注册主题。`App` 负责解析主题并注入 CSS 变量，组件改为消费语义 token；保留旧颜色别名映射作为兼容兜底，避免一次性重写所有历史颜色。

**Tech Stack:** Rust, Dioxus, serde, inline style theme bridge

---

### Task 1: 重构主题配置模型

**Files:**
- Modify: `src/theme/colors.rs`
- Modify: `src/theme/mod.rs`
- Modify: `src/config/storage.rs`

**Steps:**
1. 新增 `ThemeId`、`ThemePreference`、`ThemeSpec`、`ThemeDerivedColors`、`ThemeSyntaxColors`。
2. 注册 `ClassicDark`、`ClassicLight`、`TokyoNight` 三套主题。
3. 为 `ThemePreference` 实现兼容旧 `theme_mode` 值的序列化/反序列化。

### Task 2: 改造主题桥接与设置 UI

**Files:**
- Modify: `src/theme/css_vars.rs`
- Modify: `src/ui/app.rs`
- Modify: `src/ui/settings_dialog.rs`

**Steps:**
1. 补充派生 CSS 变量与语法高亮变量。
2. 让 `App` 基于 `ThemePreference` 解析最终主题，并向桥接脚本注入三套主题调色盘。
3. 将设置面板改成“跟随系统 / 手动选择”与“手动主题”两层。

### Task 3: 收口关键颜色区域

**Files:**
- Modify: `src/ui/json_viewer.rs`
- Modify: `src/ui/java_viewer.rs`
- Modify: `src/ui/terminal.rs`
- Modify: `src/ui/key_table.rs`
- Modify: `src/ui/value_viewer.rs`

**Steps:**
1. 将 JSON 高亮、Java Viewer、Terminal 等高影响区域改为消费语义 token。
2. 将状态底色、选中底色、类型徽标底色迁移到新的派生 token。
3. 保留 bridge 的旧颜色别名映射以覆盖未收口的历史硬编码区域。

### Task 4: 验证

**Files:**
- Verify: `src/theme/colors.rs`
- Verify: `src/ui/app.rs`
- Verify: `src/ui/settings_dialog.rs`
- Verify: `src/ui/json_viewer.rs`
- Verify: `src/ui/java_viewer.rs`
- Verify: `src/ui/terminal.rs`
- Verify: `src/ui/key_table.rs`
- Verify: `src/ui/value_viewer.rs`

**Steps:**
1. 运行 `cargo fmt`。
2. 运行 `cargo check`。
3. 记录未覆盖的人工视觉验证范围，特别是旧硬编码颜色仍由 bridge 兜底的区域。
