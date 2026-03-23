# Redis Desktop Tokyo Night Theme Design

**日期**

- 2026-03-23

**背景**

- 当前主题系统已经具备基础语义 token，但“跟随系统”和“具体主题”仍耦合在同一个枚举里。
- 现有多个界面区域仍依赖历史硬编码颜色，新增第三套主题时会越来越依赖 bridge 的颜色别名替换，扩展成本会持续上升。
- 用户希望新增 `Tokyo Night` 主题，并要求 JSON 高亮等结构化内容也随主题切换。

**目标**

- 新增一套可手动选择的 `Tokyo Night` 主题。
- 保持“跟随系统”仅在经典亮色/经典暗色之间切换。
- 抽象主题定义，降低后续新增多个主题的成本。
- 将 JSON / Java Viewer / Terminal / 关键状态底色等高影响区域收口到统一 token。

**结论**

- 将主题配置拆成两层：`ThemePreference` 与 `ThemeId`。
- 用 `ThemeSpec` 统一描述基础色、派生色和语法高亮色。
- `src/ui/app.rs` 只负责解析当前主题与注入 CSS 变量，组件消费语义 token，不感知具体主题名字。
- 保留旧颜色别名映射作为兼容兜底，但新主题能力以 token 为主，不再继续扩散硬编码。

## 一、主题模型

- `ThemePreference`
  - `System`
  - `Manual(ThemeId)`
- `ThemeId`
  - `ClassicDark`
  - `ClassicLight`
  - `TokyoNight`

解析规则：

- `System` 时，只在 `ClassicDark` 和 `ClassicLight` 之间根据系统深浅色解析。
- `Manual(ThemeId::TokyoNight)` 时，始终使用 `Tokyo Night`，不受系统外观影响。

## 二、主题定义抽象

主题注册项统一为 `ThemeSpec`：

- `id`
- `label`
- `kind`
- `colors`
- `derived`
- `syntax`

其中：

- `colors` 存基础语义色，如背景层级、边框、文本、主色、强调色、成功/警告/错误色。
- `derived` 存覆盖层、控件底色、选中底色、状态底色、表格行状态、类型徽标底色等组件级高频派生色。
- `syntax` 存结构化内容高亮色，如 key/string/number/boolean/null/keyword/type/function/comment/operator/constant。

## 三、Tokyo Night 映射

本次采用 `folke/tokyonight.nvim` 的 `night` 变体作为主参考，核心映射为：

- 背景：`#1a1b26` / `#16161e` / `#24283b` / `#292e42`
- 边框：`#3b4261`
- 主文本：`#c0caf5`
- 次文本：`#a9b1d6`
- 弱文本 / 注释：`#565f89`
- 主色：`#7aa2f7`
- 强调色：`#7dcfff`
- 成功：`#9ece6a`
- 警告：`#e0af68`
- 错误：`#f7768e`
- 语法紫：`#bb9af7`
- 常量橙：`#ff9e64`

## 四、兼容与迁移

- 配置文件中的旧字段 `theme_mode` 继续兼容读取。
- 迁移映射：
  - `System` -> `ThemePreference::System`
  - `Light` -> `ThemePreference::Manual(ThemeId::ClassicLight)`
  - `Dark` -> `ThemePreference::Manual(ThemeId::ClassicDark)`
- 新写回配置使用 `theme_preference`。

## 五、本次收口范围

- `src/theme/colors.rs`
- `src/theme/css_vars.rs`
- `src/config/storage.rs`
- `src/ui/app.rs`
- `src/ui/settings_dialog.rs`
- `src/ui/json_viewer.rs`
- `src/ui/java_viewer.rs`
- `src/ui/terminal.rs`
- `src/ui/key_table.rs`
- `src/ui/value_viewer.rs`

## 六、验收标准

- 设置中可以手动选择 `Tokyo Night`。
- `跟随系统` 仍只在经典亮/暗主题之间切换。
- `Tokyo Night` 下主界面、弹窗、表格、终端、JSON/Java 结构化展示颜色统一，不出现旧暗色残留。
- 旧配置文件可以正常读取，不需要手动删除配置。
