# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-03-22

### Added

- Initial project setup with Rust, Dioxus, and Freya
- Redis connection management
- Key browsing and searching
- Data visualization for Redis data types
- Cross-platform support (macOS, Windows, Linux)
- add application icon
- add Java serialization parser and resizable panels
- add large key detection and pagination support
- add batch operations for keys
- add List/Set/ZSet visual editors
- add progressive scan with progress indicator
- add real-time monitoring panel
- add slowlog viewer panel
- add client connection management panel
- add command auto-completion in terminal
- add flush database feature and key type caching
- add theme mode support with light/dark/system options
- add search and JSON export for Java viewer
- add edit functionality for Set type members
- add indeterminate checkbox state for partial selection in tree view
- TTL editor and remove right panel
- action buttons to value viewer header
- prioritize key list workspace
- rebuild redis engine workbench
- 检测到 Java 序列化时自动选中 Java 解析格式
- 添加应用设置界面

### Changed

- apply cargo fmt formatting
- replace emoji icons with Lucide SVG icons
- unify accent color to system blue and improve syntax highlighting
- auto-expand all nodes when searching
- improve connection list visual hierarchy with hover effects
- adopt design spec dark mode color scheme
- change key list from table to tree layout with value panel
- extract and display field values for Java objects
- use jaded library for Java serialization parsing
- improve primary button contrast in light mode
- simplify workspace layout and sync redis db context
- 移除双击重连功能
- 添加连接选中状态高亮

### Fixed

- 修复连接失败时右侧仍显示加载中的问题
- 修复树形视图无法正确解析 jaded 序列化结构
- wrap long values like base64 in multiline mode
- cascade selection when selecting folders in multi-select mode
- allow folder selection in multi-select mode and enable expand toggle via arrow click
- 修复切换连接时右侧面板不更新的问题
- 为趋势图添加坐标轴和悬停提示
- 修复 FlushDB 后 Key 列表不刷新的问题
- 修复实时监控数据刷新问题
- refresh system theme without manual selection
- unify panel colors across light and dark modes