# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-04-30

### Added
- lazy-load key type icons in tree based on visible range
- highlight search keyword in key tree nodes

### Fixed
- require Enter to refresh key tree when search is cleared
- resolve key tree stuck in filtered state after clearing search
- ensure NSIS VIProductVersion has 4 segments for stable releases
- compare prerelease versions correctly

## [0.1.1] - 2026-04-16

### Added
- add bilingual UI and language preference
- add launch at startup toggle
- redesign about page
- improve settings dialog and search expansion
- export error_reporting module
- add error reporting infrastructure (dialog + log + terminal)
- export error module in lib.rs
- add unified error types (AppError, StartupError, ConfigError)
- configure EdDSA public key for auto updates
- implement macOS installer with Autoupdate.app bridge
- add appcast generation and release workflow integration
- integrate Sparkle framework into app bundle
- add EdDSA key generation and signing scripts
- integrate complete update flow with UI
- add update notification dialog UI
- add manual update check menu item
- integrate startup update check
- implement unified update manager
- add unified installer interface
- implement platform installers (Windows complete)
- implement update config management
- implement download manager with progress
- implement version checker with GitHub API
- add module structure and error types
- enhance drag-and-drop interaction with preview card and smooth animations
- add lazy loading for large dataset pagination
- add scale fade animation to context menu
- add drag-and-drop reordering for connections
- add enter/exit animations to all dialogs and modals
- add VirtualTreeList component with virtual scrolling
- add prefix grouping, TTL analysis and sample ratio support
- add RedisJSON support with JSON.MERGE command
- add Protobuf detection and raw parsing support
- add BSON, CBOR formats and YAML, TOML formatters
- add system tray support for macOS and Windows
- add comprehensive Redis command completion with JSON and more
- add Zstd decompression support
- add image preview support for PNG/JPEG/GIF/WEBP formats
- add memory analysis tool for scanning large keys
- add pattern delete dialog for batch key deletion
- add import/export connection configurations
- add readonly mode for connections
- add consumer group management UI
- add Stream consumer group command support
- add multi-language support framework
- add custom formatter support
- implement HELP command for Redis command docs
- redesign theme preference with separate light/dark themes
- add SSL/TLS support
- add Stream type visualization support
- add Bitmap visualization support
- add Redis Cluster support
- add SSH tunnel support for Redis connections
- add Pub/Sub panel for Redis pub/sub operations
- add Lua script management panel
- add data import functionality
- add animation system with theme-aware transitions
- add Tokyo Night theme support
- add 5 new themes (Tokyo Night Light, Atom One Light, GitHub Light, One Dark Pro, Dracula)
- add MessagePack serialization support
- add Python Pickle serialization support
- add PHP serialization support
- add Kryo/FST serialization support
- add resizable panels for connection and key list areas
- add copy buttons for all value types
- add launch at startup toggle

### Changed
- sync lockfile and format ui modules
- format remaining source files
- remove about resources card
- align docs with current app state
- sync package version
- clean up value_viewer module imports
- integrate set, zset, and stream value panels
- extract set, zset, and stream value panels
- extract hash and list value viewer panels
- remove duplicated value_viewer implementations
- split value_viewer into modular directory
- format extracted app actions module
- extract app select and reconnect workflows
- extract app save and delete workflows
- extract app titlebar and connected tab shell
- extract app list actions into action builders
- extract app dialog wrappers and simple actions
- extract app effects and common status panels
- convert app.rs to modular app/ structure
- replace unsafe unwraps in UI panels and protobuf parser
- add fallbacks to guarded unwraps, clean main.rs
- main.rs with unified error handling
- clean up ConfigStorage::new() error handling
- add NSIS installer for Windows release
- unify button styles and refactor key browser toolbar

### Fixed
- compare prerelease versions correctly
- recover stale redis connections during reads
- use writable webview2 data dir
- handle missing webview2 runtime
- preserve dialog close animation
- keep settings header fixed while scrolling
- embed app icon in release builds
- package desktop installer assets correctly
- default to tokyo night themes
- use pages update manifest
- correct dialog header alignment
- encrypt stored connection credentials with AES
- handle mutex poisoning in tray.rs, return Result
- remove unsafe Default impl from UpdateDownloader
- remove unsafe Default impl from UpdateManager
- add error and error_reporting modules to main.rs
- resolve JSON preview overflow without scrollbar
- improve connection drag sorting
- add context menu support to VirtualTreeList and fix menu item click

## [0.1.1-beta.7] - 2026-04-15

### Fixed
- recover stale redis connections during reads

## [0.1.1-beta.6] - 2026-04-11

### Changed
- sync lockfile and format ui modules

### Fixed
- use writable webview2 data dir

## [0.1.1-beta.5] - 2026-04-09

### Added
- add bilingual UI and language preference

### Fixed
- handle missing webview2 runtime
- preserve dialog close animation

## [0.1.1-beta.4] - 2026-04-08

### Added
- add launch at startup toggle
- redesign about page

### Changed
- format remaining source files
- remove about resources card
- update README with features and current status
- align docs with current app state
- sync package version

### Fixed
- keep settings header fixed while scrolling
- embed app icon in release builds
- package desktop installer assets correctly
- default to tokyo night themes

## [0.1.1-beta.3] - 2026-04-08

### Added
- add unified error types and error reporting infrastructure
- improve settings dialog and search expansion

### Changed
- refactor app.rs into modular app components and workflows
- refactor value viewer panels into extracted modules
- refactor UI safety checks and startup error handling

### Fixed
- fix update checks to use the GitHub Pages update manifest instead of the GitHub Releases API
- fix dialog header alignment and JSON preview overflow
- encrypt stored connection credentials with AES

## [0.1.1-beta.2] - 2026-03-29

### Added
- configure EdDSA public key for auto updates
- implement macOS installer with Autoupdate.app bridge
- add appcast generation and release workflow integration
- integrate Sparkle framework into app bundle
- add EdDSA key generation and signing scripts
- integrate complete update flow with UI
- add update notification dialog UI
- add manual update check menu item
- integrate startup update check
- implement unified update manager
- add unified installer interface
- implement platform installers (Windows complete)
- implement update config management
- implement download manager with progress
- implement version checker with GitHub API
- add module structure and error types

### Changed
- add Frameworks to gitignore (Sparkle downloaded by CI)
- add NSIS installer for Windows release

## [0.1.1-beta.1] - 2026-03-28

### Added
- add context menu support to VirtualTreeList for key tree right-click operations
- add comprehensive test framework for connection modes and value parsing
- add BSON, CBOR serialization formats and YAML, TOML formatters
- add protobuf schema support for decoding
- add fullscreen image preview with zoom and improved UI
- add comprehensive Redis command completion with JSON support
- add RedisJSON support with JSON.MERGE command
- add Zstd decompression support
- add system tray support for macOS and Windows
- add memory analysis tool with prefix grouping, TTL analysis and sample ratio support
- add virtual scrolling with VirtualTreeList for large key lists
- add FlatTreeAdapter for virtual scrolling key list

### Fixed
- fix context menu item click not working due to capture phase event handling
- fix connection drag sorting with enhanced preview card and smooth animations
- fix MsgPack encoding for test data
- fix serialization/bitmap detection order when UTF-8 decode fails
- fix false positive detection for binary data in MsgPack and CBOR
- fix glob pattern matching in test data
- correct protobuf type mismatches and borrow issues
- improve image detection to prioritize over serialization formats
- fix protobuf detection to avoid false positives with Bitmap data
- fix virtual scrolling integration compilation errors

### Changed
- unify button styles across UI components
- refactor key browser toolbar with improved database selector
- right-align drag handle in connection list
- improve value viewer formatting
- improve image preview to use temp files instead of data URLs
- enhance key browser with database selection dropdown and more actions menu

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
