# Redis Engine Workbench Design

**日期**

- 2026-03-22

**背景**

- 设计稿已经明确了新的产品气质：顶部全局导航、左侧功能导向侧栏、主工作台中的 Key Explorer 与 Value Editor 同屏协作。
- 当前实现虽然已经引入接近设计稿的暗色主题配色，但页面骨架仍然沿用旧版三栏结构：连接侧栏、Key Browser、右侧 Tab 内容。
- 代码上，这个结构主要集中在 `src/ui/app.rs`、`src/ui/sidebar.rs`、`src/ui/key_browser.rs`、`src/ui/value_viewer.rs`。

**目标**

- 让桌面端主界面从“连接列表 + Tree + 详情页”升级为“全局壳层 + 左侧功能导航 + 数据工作台”。
- 保留现有 Redis 数据链路、扫描逻辑和已完成的编辑能力，重点重构布局、信息架构和视觉层级。
- 在不破坏现有 Data、Terminal、Monitor、SlowLog、Clients 功能的前提下，统一风格和交互。

**非目标**

- 本次设计不改 Redis 命令层接口，不重写扫描、TTL、Hash 编辑等底层数据逻辑。
- 本次设计不要求 1:1 复刻 HTML/Tailwind 实现，只要求在 Dioxus/Freya 中达到相同的信息层次和视觉语义。
- 本次设计不在第一阶段解决完整 i18n，只要求先统一当前主路径文案风格。

**备选方案**

**方案 A：只做样式微调**

- 做法：保留现有结构，仅调整颜色、圆角、边框、字体和按钮样式。
- 优点：改动最小，风险最低。
- 缺点：无法解决顶部导航缺失、左栏职责错误、Key Explorer 交互模型不一致的问题。

**方案 B：完全按设计稿重写**

- 做法：直接按设计稿重新组织所有主界面组件，弱化甚至移除现有 tree 模型。
- 优点：视觉还原度最高。
- 缺点：会破坏当前已适应 namespace 的 key 浏览方式，重写成本高，回归风险大。

**方案 C：设计稿驱动的混合重构**

- 做法：保留现有数据模型、Tab 内容和编辑能力，用新的壳层与工作台布局重新组织展示方式。
- 优点：兼顾设计还原、实现成本和已有功能复用。
- 缺点：需要先做一次壳层和 token 重构，再逐步迁移中间内容。

**结论：采用方案 C。**

这是当前代码库最稳妥的方案。它允许我们保留现有 `KeyBrowser` 和 `ValueViewer` 中成熟的数据流，同时逐步把视觉和信息架构对齐到设计稿。

## 一、总体布局

主界面改为四层结构：

1. `TopNav`
2. `AppBody`
3. `LeftRail`
4. `Workspace`

布局关系：

- 顶部固定 `TopNav`，承担品牌、全局导航、全局搜索和系统入口。
- 主体区域为左右结构，左侧是固定宽度 `LeftRail`，右侧是 `Workspace`。
- `Workspace` 内根据一级功能切换内容，其中默认进入 `ExplorerWorkspace`。

## 二、顶部导航

顶部导航提供以下信息与操作：

- 品牌标题：`Redis Engine`
- 一级导航：`Cluster`、`Backups`、`Metrics`
- 全局搜索框
- 设置、帮助、通知入口

实现原则：

- 顶部导航是产品级壳层，不再把这些能力散落在局部面板内。
- 当前 `Data`、`Terminal`、`Monitor`、`SlowLog`、`Clients` 这类与连接强相关的视图保留，但改为工作台内二级导航，而不是产品级导航。
- 去掉 emoji 标签，统一走 SVG 图标体系。

## 三、左侧功能侧栏

左侧栏从“连接列表容器”改为“功能型侧栏”，分四块：

1. 当前连接状态卡
2. 主操作按钮
3. 功能导航
4. Key Patterns 与底部工具区

具体内容：

- 顶部显示当前连接名称、地址、连接状态和耗时。
- 保留 `New Connection` 按钮作为主要 CTA。
- 主导航显示 `Explorer`、`Connections`、`Streams`、`CLI`、`Monitoring`。
- 中部显示 `Key Patterns`，作为常用命名空间入口。
- 底部保留 `Logs`、`Settings`。

连接列表处理策略：

- 多连接能力保留。
- 连接列表从当前整栏占位，降级为连接管理视图的一部分，或在状态卡下方以折叠列表展示。
- 默认左栏应服务于“当前选中连接的工作流”，而不是长期只承担连接管理。

## 四、Explorer 工作台

`ExplorerWorkspace` 是本次重构的中心区域，结构如下：

1. 页面标题与统计
2. 操作按钮区
3. SCAN 查询条
4. Key 表格
5. 分页状态条
6. 底部 Value Editor 区

### 页面标题与统计

- 显示 `Key Explorer`
- 显示当前 DB 和 key 总数
- 把当前散落在 key browser 的 DB 信息、搜索状态、keys count 汇总到更清晰的位置

### 操作按钮区

- 提供 `Reload`
- 提供类型过滤或模式过滤入口
- 后续可接入批量操作入口

### SCAN 查询条

- 不是做真正 CLI，而是展示用户当前筛选条件对应的查询意图
- 例如：`SCAN 0 MATCH auth:* COUNT 100`
- 当前 `search_pattern`、DB、批量大小等状态可复用现有逻辑生成展示文案

### Key 表格

- 以表格替代当前“tree-only 主视图”
- 列建议为：选择框、Key Name、Type、TTL、Actions
- 支持 hover、选中态、快捷操作显隐

tree 模型处理策略：

- 不删除现有 `TreeBuilder`
- tree 继续用于：
  - 提取 key patterns
  - namespace 分组
  - 作为可折叠筛选来源
- 默认主浏览区域改为平面表格，避免中间区域继续承担树状导航主职责

### 分页状态条

- 显示当前区间与总数
- 保留页码、上一页、下一页
- 如果仍使用 SCAN 流式加载，可先实现“视图分页”，不要求第一阶段就完全后端分页

## 五、Value Editor 区

设计稿要求 Key 列表和编辑区同屏存在，这是和当前实现差异最大的部分。

新的 Value Editor 区分为左右两块：

1. 左侧主编辑区
2. 右侧辅助卡片区

### 左侧主编辑区

- 展示当前选中的 key
- 根据 key 类型渲染已有编辑器
- Hash 类型直接复用当前已完成的表格编辑能力
- String、List、Set、ZSet 等类型逐步沿用当前 `ValueViewer` 内的编辑分支

### 右侧辅助卡片区

- `Key Settings`
- `Danger Zone`
- 后续可扩展元数据卡片

卡片职责：

- `Key Settings` 展示和编辑 TTL、导出、复制路径等
- `Danger Zone` 单独承接删除、unlink 等高风险操作

### 复用策略

- 当前 `ValueViewer` 的 Hash 表格编辑已经具备“搜索 + 新增行 + 表格 + 行级操作”的主框架，可以直接作为新编辑区主体能力。
- 这部分只需要重新组织布局和视觉，不需要推翻现有 Redis 交互。

## 六、设计 Token 与视觉语义

当前主题层只有基础颜色，不足以支撑设计稿中的层级感。

需要补齐以下语义：

- `surface_lowest`
- `surface_low`
- `surface`
- `surface_high`
- `surface_highest`
- `outline`
- `outline_variant`
- `text_primary`
- `text_secondary`
- `text_subtle`
- `primary`
- `accent`
- `error`
- `radius_sm`
- `radius_md`
- `radius_lg`
- `shadow_panel`
- `font_body`
- `font_mono`

设计原则：

- 组件不直接写死颜色名，应尽量消费语义 token。
- 字体也应改为语义配置，不再在各组件里零散写 `Consolas` 或默认字体。

## 七、文案与图标

- 主工作流文案先统一为中文，避免当前中英混排割裂。
- 可以保留设计稿的结构和排版气质，但不要求逐字照搬英文文案。
- 所有导航、按钮、状态入口统一使用现有 SVG 图标组件，不再使用 emoji。

## 八、分阶段落地建议

**第一阶段：壳层与 token**

- 先完成 `TopNav`、`LeftRail`、主题 token 扩展
- 目标是把页面骨架先搭起来

**第二阶段：Explorer 工作台**

- 重构 `KeyBrowser` 为标题区、操作区、查询条、表格、分页
- 让“看 key”体验先接近设计稿

**第三阶段：Value Editor 与右侧卡片**

- 将 `ValueViewer` 从全屏详情改为工作台下半区
- 引入 `Key Settings` 和 `Danger Zone`

**第四阶段：一致性收口**

- 文案统一
- 图标统一
- 状态统一
- hover/selected/loading/empty/error 全量补齐

## 九、验收标准

- 打开应用后能看到顶部导航和新的左侧功能侧栏
- 默认数据工作台能同时看到 key 列表和当前 key 编辑区
- Hash 编辑能力保持可用
- 连接切换、DB 切换、搜索、扫描、刷新、TTL 修改、危险操作入口都能继续使用
- UI 中不再出现 emoji tab 和明显的旧版布局残留
