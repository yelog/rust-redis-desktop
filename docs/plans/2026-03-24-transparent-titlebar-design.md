# Redis Desktop Transparent Titlebar Design

**日期**

- 2026-03-24

**背景**

- 当前应用已经把 macOS 原生标题栏切到亮/暗外观，但标题栏背景仍是系统材质色。
- 内容区使用的是自定义主题色，导致标题栏和下方界面在 Tokyo Night、Dracula 等主题下仍然割裂。

**目标**

- 保留 macOS 原生 traffic lights 和窗口行为。
- 让标题栏区域视觉上融入应用主题，而不是停留在系统默认材质。
- 尽量只改窗口配置和根布局，不动现有业务组件。

**结论**

- 启用 macOS 透明标题栏和 full-size content view。
- 隐藏原生标题文本，保留系统按钮。
- 在应用根部增加一条可拖拽的主题顶栏，让应用自己的背景色延伸到标题栏区域。

## 一、窗口层策略

`src/main.rs`

- 在 macOS 上开启 `with_titlebar_transparent(true)`。
- 开启 `with_fullsize_content_view(true)` 让内容延伸到标题栏下。
- 开启 `with_title_hidden(true)`，避免原生标题文本和自绘顶栏重复。

## 二、界面层策略

`src/ui/app.rs`

- 在根布局最顶部增加 macOS 专用顶栏。
- 顶栏背景直接使用当前主题色，并加一条轻微底边增强层次。
- 顶栏左侧预留 traffic lights 区域，避免内容与系统按钮重叠。
- 顶栏支持拖动窗口，双击时切换最大化。

## 三、验收标准

- 标题栏区域和内容区背景在当前主题下视觉连贯。
- 原生关闭/缩小/放大按钮仍可正常使用。
- 拖拽窗口和双击顶栏的交互正常。
- `cargo check` 通过。
