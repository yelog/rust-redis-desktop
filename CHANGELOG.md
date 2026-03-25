# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1-beta.0] - 2026-03-25

### Added
- auto-start export in connection export dialog
- add export to file in connection export dialog
- show ServerInfoPanel when no key selected
- add drag-and-drop reordering for connections
- add enter/exit animations to all dialogs and modals
- add lazy loading for large dataset pagination
- add scale fade animation to context menu
- add context menu for key tree with icons
- add context menu for connection list
- add multi-language support framework
- add custom formatter support
- add consumer group management UI
- add Stream consumer group command support
- add memory analysis tool for scanning large keys
- add import/export connection configurations
- add pattern delete dialog for batch key deletion
- add readonly mode for connections
- implement HELP command for Redis command docs
- improve TTL editor with -1 for permanent
- add confirmation dialog before deleting connection
- redesign theme preference with separate light/dark themes
- add global toast notification system for copy operations
- add SSL/TLS support, data export, and enhancements
- add Stream type visualization support
- add Bitmap visualization support
- add connection test and fix cluster mode UI
- add advanced features and security enhancements
- add Lua script management panel
- add data import functionality
- add SSH tunnel support for Redis connections
- add Pub/Sub panel for Redis pub/sub operations
- add Redis Cluster support
- improve key type selector and inline TTL editor
- add window size and position persistence
- improve add key dialog type selector and fix overflow
- add dynamic modal animation origin and ESC key support
- add modal dialog open/close animations
- add 5 new themes (Tokyo Night Light, Atom One Light, GitHub Light, One Dark Pro, Dracula)
- add Kryo/FST serialization support
- add Python Pickle serialization support
- add MessagePack serialization support
- add PHP serialization support
- add animation system with theme-aware transitions
- add resizable panels for connection and key list areas
- add Tokyo Night theme support
- add Settings menu item with Cmd+, shortcut on macOS
- add copy buttons for all value types
- batch load key types on node expand

### Changed
- use toast notifications and improve TTL input styling in value viewer
- use theme color for button text in pubsub and script panels
- rename Classic themes to 酒红亮/酒红暗
- increase dialog width by 100px
- centralize clipboard access
- update flush and import buttons with icons and consistent style
- add close icon and remove cancel button in dialogs
- change connection mode selector to pill buttons
- format code with consistent indentation and line breaks
- improve settings dialog UX
- remove window position/size persistence
- optimize key type fetching and add connection pool support
- add docker-compose for Redis testing environments
- unify copy button style and position across all data types
- unify copy button style with icon and text
- replace connection mode dropdown with radio buttons
- 将 Java 序列化解析函数移至 serialization 模块

### Fixed
- show ServerInfoPanel when no key selected
- isolate context menu instances across panels
- use polling only for version detection, reduce delay to 16ms
- only set my_close_version on menu initialization
- reset mounted flag when context menu is hidden by version change
- prevent context menu from disappearing when right-clicking same area
- prevent multiple context menus from showing simultaneously
- improve server-side search for hash/set/zset data
- add backdrop animation and global ESC handler for dialogs
- align JSON viewer copy button to the right
- unify dialog titlebars
- improve context menu handling for right-click switching
- improve memory analysis dialog layout
- auto-focus dialog on open for immediate ESC key response
- properly close context menu before opening new one
- close old context menu before opening new one
- fetch TTL from Redis when opening TTL dialog
- improve TTL dialog title and show current TTL
- restore missing add key button in key browser
- correct PHP string parsing using byte slicing
- auto-select correct binary format for MsgPack/Pickle/Kryo/Fst
- improve editable field layout and table column width
- prevent textarea overflow in add key dialog
- prevent horizontal scrollbar in dialog
- blend macOS titlebar with app theme
- position close button at dialog's top-right corner using absolute positioning
- sync native titlebar with app theme
- use use_future for toast auto-dismiss with periodic cleanup
- change toast text color to dark for better readability
- use container names for cluster init in docker-compose
- set longer connection timeout for Cluster mode
- improve connection test reliability with timeout and logging
- resolve compilation errors in Redis command execution
- add ESC key support for closing all dialogs
- fix modal close animation not playing
- disable key type loading to prevent connection lock contention
- unify connection status dot colors with state_* theme colors
- remove duplicate key display in value viewer
- 保留刷新后 key 树的展开状态
- 修复切换 Redis 连接时右侧面板不刷新的问题
- add scrollbar for overflow content in value viewer
- include macOS app icon in dmg bundle

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