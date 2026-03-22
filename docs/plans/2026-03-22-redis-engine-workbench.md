# Redis Engine Workbench Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the desktop app shell into a design-driven Redis workbench with top navigation, functional left rail, a table-first key explorer, and a split value editor while reusing the existing Redis data flow.

**Architecture:** Keep the current Dioxus/Freya data layer and editor logic, but split the UI into reusable shell components and semantic theme tokens. Refactor `App` to own the new layout, migrate `KeyBrowser` from a tree-led explorer into a workbench surface, and reorganize `ValueViewer` into a lower editor area with side cards.

**Tech Stack:** Rust, Dioxus 0.7, Freya 0.3, existing Redis connection and UI modules

---

### Task 1: 扩展主题语义 Token

**Files:**
- Modify: `src/theme/colors.rs`
- Modify: `src/theme/css_vars.rs`
- Modify: `src/theme/mod.rs`
- Modify: `src/ui/app.rs`

**Step 1: 扩展 `ThemeColors` 字段**

在 `src/theme/colors.rs` 为深色和浅色主题补充更多语义字段：

```rust
pub struct ThemeColors {
    pub background: &'static str,
    pub background_secondary: &'static str,
    pub background_tertiary: &'static str,
    pub surface_lowest: &'static str,
    pub surface_low: &'static str,
    pub surface_high: &'static str,
    pub surface_highest: &'static str,
    pub border: &'static str,
    pub outline_variant: &'static str,
    pub text: &'static str,
    pub text_secondary: &'static str,
    pub text_subtle: &'static str,
    pub primary: &'static str,
    pub accent: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
}
```

**Step 2: 为新增字段补充 CSS 变量常量**

在 `src/theme/css_vars.rs` 增加对应常量：

```rust
pub const COLOR_SURFACE_LOWEST: &str = "var(--theme-surface-lowest)";
pub const COLOR_SURFACE_LOW: &str = "var(--theme-surface-low)";
pub const COLOR_SURFACE_HIGH: &str = "var(--theme-surface-high)";
pub const COLOR_SURFACE_HIGHEST: &str = "var(--theme-surface-highest)";
pub const COLOR_OUTLINE_VARIANT: &str = "var(--theme-outline-variant)";
pub const COLOR_TEXT_SUBTLE: &str = "var(--theme-text-subtle)";
```

**Step 3: 更新主题桥接脚本输出**

在 `src/ui/app.rs` 的主题桥接构建逻辑中输出新增字段，保证 DOM 和 Freya 样式都能消费同一套语义值。

**Step 4: 运行编译检查**

Run: `cargo check`

Expected: PASS，只有因字段新增导致的编译错误被修复，没有遗留未使用或缺失字段。

**Step 5: 手动确认旧界面未崩坏**

Run: `cargo run`

Expected: 应用仍能启动，现有界面在未开始布局重构前外观不必最终正确，但不能出现明显空白或崩溃。

### Task 2: 新增顶层导航和左侧功能侧栏组件

**Files:**
- Create: `src/ui/top_nav.rs`
- Create: `src/ui/left_rail.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/sidebar.rs`

**Step 1: 创建 `TopNav` 组件骨架**

在 `src/ui/top_nav.rs` 中定义独立组件：

```rust
#[component]
pub fn TopNav(
    colors: ThemeColors,
    on_open_settings: EventHandler<()>,
) -> Element {
    rsx! {
        div { "TODO" }
    }
}
```

**Step 2: 创建 `LeftRail` 组件骨架**

在 `src/ui/left_rail.rs` 中定义新侧栏组件，参数先沿用当前连接和回调模型：

```rust
#[component]
pub fn LeftRail(
    width: f64,
    colors: ThemeColors,
    selected_connection: Option<Uuid>,
) -> Element {
    rsx! {
        div { "TODO" }
    }
}
```

**Step 3: 在 `src/ui/mod.rs` 中导出新组件**

补充：

```rust
mod left_rail;
mod top_nav;

pub use left_rail::*;
pub use top_nav::*;
```

**Step 4: 保留旧 `Sidebar` 作为过渡层**

不要立即删除 `src/ui/sidebar.rs`。先把它缩减为连接管理视图的复用组件，避免第一次重构时把连接选择、右键菜单和回调全部打散。

**Step 5: 运行编译检查**

Run: `cargo check`

Expected: PASS，新组件已接入导出但尚未实际替换主布局。

### Task 3: 用新壳层重构 `App`

**Files:**
- Modify: `src/ui/app.rs`
- Modify: `src/ui/resizable_divider.rs`

**Step 1: 为 `App` 增加顶层壳层布局**

把当前主体从：

```rust
Sidebar + ResizableDivider + KeyBrowser + ResizableDivider + TabContent
```

改为：

```rust
TopNav + LeftRail + Workspace
```

**Step 2: 将当前内容区 tab 导航改为工作台二级导航**

保留 `Tab::Data`、`Tab::Terminal`、`Tab::Monitor`、`Tab::SlowLog`、`Tab::Clients`，但去掉 emoji 标签，改成统一图标和文案。

**Step 3: 让分割线只服务工作区内部**

左侧功能侧栏宽度固定或半固定，`ResizableDivider` 仅在工作区内部用于 explorer 区和辅助区域的调节。

**Step 4: 修复连接切换逻辑**

确保当前连接切换、重连中状态和空连接状态在新壳层下继续可见。

**Step 5: 运行编译检查和启动验证**

Run: `cargo check`

Expected: PASS

Run: `cargo run`

Expected: 能看到顶部导航和新的左侧侧栏骨架，旧功能仍然可达。

### Task 4: 将 `KeyBrowser` 重构为 Explorer 工作台上半区

**Files:**
- Modify: `src/ui/key_browser.rs`
- Create: `src/ui/key_table.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/lazy_tree_node.rs`
- Modify: `src/ui/virtual_key_list.rs`

**Step 1: 新建 `KeyTable` 组件**

在 `src/ui/key_table.rs` 中定义表格组件，至少接收以下数据：

```rust
#[component]
pub fn KeyTable(
    rows: Vec<KeyInfo>,
    selected_key: String,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        table { "TODO" }
    }
}
```

**Step 2: 在 `KeyBrowser` 中拆出标题区和操作区**

把当前 `DB select + 搜索 + 新增/刷新` 区块拆成：

- `ExplorerHeader`
- `ExplorerActions`
- `ScanBar`

先用现有 `current_db`、`keys_count`、`search_pattern`、`scan_progress` 填数据。

**Step 3: 把 tree 从主展示退居辅助角色**

保留 `TreeBuilder` 和 `TreeNode` 产物，用于：

- 生成 key patterns
- 生成过滤条件
- 为后续可折叠 namespace 面板提供数据

第一阶段主区域先渲染平面表格，不再直接把 tree 作为默认主视图。

**Step 4: 增加分页状态条**

即使第一阶段仍使用一次 SCAN 后本地分页，也先补齐分页控件和“Showing X-Y of Z”文案区。

**Step 5: 运行编译检查**

Run: `cargo check`

Expected: PASS，Explorer 上半区已经具有标题、按钮、查询条、表格和分页的结构。

### Task 5: 让 `ValueViewer` 进入同屏编辑模式

**Files:**
- Modify: `src/ui/value_viewer.rs`
- Create: `src/ui/key_meta_panel.rs`
- Create: `src/ui/danger_zone_card.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/ttl_editor.rs`

**Step 1: 新建右侧辅助卡片组件**

创建 `KeyMetaPanel`：

```rust
#[component]
pub fn KeyMetaPanel(
    key_info: Option<KeyInfo>,
) -> Element {
    rsx! {
        div { "TODO" }
    }
}
```

创建 `DangerZoneCard`：

```rust
#[component]
pub fn DangerZoneCard(
    selected_key: String,
) -> Element {
    rsx! {
        div { "TODO" }
    }
}
```

**Step 2: 调整 `ValueViewer` 布局层次**

把 `ValueViewer` 从“占满整个内容区的详情视图”改为“编辑器工作台下半区”，结构改成：

```rust
EditorMain + EditorSideCards
```

**Step 3: 保留各类型编辑能力**

不要重写 `String`、`Hash`、`List`、`Set`、`ZSet` 的数据逻辑。优先保留现有分支，只抽离布局容器和卡片式外围结构。

**Step 4: 优先对齐 Hash 视图**

因为 Hash 表格已经最接近设计稿，优先把它嵌入新的 Value Editor 主体，确保搜索、新增、编辑、删除继续可用。

**Step 5: 运行编译检查与手动验证**

Run: `cargo check`

Expected: PASS

Manual: 选中一个 Hash key，确认列表和编辑区可同屏看到，新增行、搜索、删除仍正常。

### Task 6: 统一图标、文案和状态样式

**Files:**
- Modify: `src/ui/app.rs`
- Modify: `src/ui/icons.rs`
- Modify: `src/ui/sidebar.rs`
- Modify: `src/ui/key_browser.rs`
- Modify: `src/ui/value_viewer.rs`
- Modify: `src/ui/server_info.rs`

**Step 1: 去掉 emoji 和不统一的标题样式**

把 `📊 Data`、`💻 Terminal` 这类标签替换成统一 SVG 图标加中文或一致英文文案。

**Step 2: 统一按钮视觉层级**

明确主按钮、次按钮、危险按钮、幽灵按钮四种样式，不再在各文件里继续写散装颜色。

**Step 3: 统一状态反馈**

统一 loading、empty、error、selected、hover、disabled 的视觉语义，优先消费新的主题 token。

**Step 4: 统一字体使用**

正文统一 body 字体，key、命令、TTL、JSON、终端输出等内容统一 mono 字体，不再分散写默认系统字体或 `Consolas`。

**Step 5: 运行编译检查**

Run: `cargo check`

Expected: PASS，界面上不再出现明显的旧版视觉残留。

### Task 7: 回归验证和收尾

**Files:**
- Modify: `README.md`
- Modify: `docs/PROGRESS.md`

**Step 1: 运行基础检查**

Run: `cargo check`

Expected: PASS

**Step 2: 运行测试**

Run: `cargo test`

Expected: PASS；如果没有测试覆盖 UI 变更，记录这一点，不要假设安全。

**Step 3: 手动验证主流程**

手动验证：

- 连接切换
- 重连中状态
- DB 切换
- 搜索与扫描
- Key 选择
- Hash 编辑
- TTL 修改
- 危险操作入口
- Terminal、Monitor、SlowLog、Clients 可达

**Step 4: 更新文档**

在 `README.md` 或 `docs/PROGRESS.md` 简要记录新的界面结构和已完成阶段，避免后续继续按旧布局开发。

**Step 5: 提交**

```bash
git add docs/plans/2026-03-22-redis-engine-workbench-design.md docs/plans/2026-03-22-redis-engine-workbench.md
git commit -m "docs: add redis engine workbench design and plan"
```
