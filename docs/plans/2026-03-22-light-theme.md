# Redis Engine Light Theme Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让亮色模式继承暗色模式的品牌配色与层级语义，去掉当前偏系统蓝灰的视觉风格。

**Architecture:** 优先修改主题 token 和亮色派生调色盘，让已有组件继续消费语义色值；只在确实必要时补局部样式。这样能最小化对现有 UI 结构的扰动，并保证亮暗两套主题通过同一主题桥接脚本生效。

**Tech Stack:** Rust, Dioxus, theme tokens, inline style theme bridge

---

### Task 1: 更新亮色主题 token

**Files:**
- Modify: `src/theme/colors.rs`

**Steps:**
1. 将亮色模式的背景、surface、边框、文本、副文本统一切到暖白/暖灰系。
2. 将亮色 `primary` 改为深珊瑚红，`accent` 改为深青色。
3. 保留成功、警告、错误色的语义，但让饱和度和明度与新主题协调。

### Task 2: 同步亮色派生色

**Files:**
- Modify: `src/ui/app.rs`

**Steps:**
1. 更新 `build_theme_palette` 里的 light 分支派生值。
2. 让 `infoBg`、`selectionBg`、`successBg`、`errorBg`、`syntax*` 等颜色与新 token 对齐。
3. 保证主题桥接脚本切换 light/system 时能正确注入新的 CSS 变量。

### Task 3: 基础验证

**Files:**
- Verify: `src/theme/colors.rs`
- Verify: `src/ui/app.rs`

**Steps:**
1. 运行 `cargo check`。
2. 检查是否有类型或格式错误。
3. 记录未做的人工视觉验证范围。
