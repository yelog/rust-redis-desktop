# Rust Redis Desktop - 技术架构决策

## 核心技术栈

### GUI 框架
- **Dioxus 0.6+** - React-like 的 Rust UI 框架
- **Freya 0.2+** - Skia GPU 加速渲染引擎

### 为什么选择 Dioxus + Freya？

#### 性能优势
| 指标 | Tauri (WebView) | Dioxus + Freya |
|------|-----------------|----------------|
| 启动速度 | 300-800ms | **50-200ms** |
| 内存占用 | 30-50MB | **10-20MB** |
| 大数据渲染 | 卡顿 | **流畅** |
| IPC 开销 | 有 | **无** |

#### Redis 客户端场景优势
1. **10万+ keys 虚拟滚动** - 零拷贝渲染
2. **树形结构展开** - 高效虚拟化
3. **实时数据更新** - 无序列化开销
4. **极致启动速度** - 原生渲染

### Redis 客户端层
- **redis 0.27+** - 异步 Redis 客户端
  - 支持 Cluster
  - 支持 Sentinel
  - 异步 I/O (tokio)

### 异步运行时
- **tokio** - Rust 异步运行时标准

### 数据处理
- **serde / serde_json** - 序列化/反序列化

---

## 架构设计

```
┌─────────────────────────────────────────────┐
│         UI Layer (Dioxus + Freya)           │
│    - React-like 组件化开发                   │
│    - 声明式 UI                               │
│    - Skia GPU 渲染                          │
└─────────────────────────────────────────────┘
                    ↓ 零拷贝
┌─────────────────────────────────────────────┐
│         State Management Layer              │
│    - Ferret (Dioxus 内置)                   │
│    - 响应式状态管理                          │
│    - 全局状态 + 局部状态                     │
└─────────────────────────────────────────────┘
                    ↓ 异步
┌─────────────────────────────────────────────┐
│         Redis Client Layer                   │
│    - 连接池管理                              │
│    - 命令执行器                              │
│    - 数据解析器                              │
│    - Cluster/Sentinel 支持                   │
└─────────────────────────────────────────────┘
                    ↓ 异步 I/O
┌─────────────────────────────────────────────┐
│         Network Layer (Tokio)               │
│    - TCP 连接                                │
│    - SSL/TLS                                │
│    - SSH Tunnel                             │
└─────────────────────────────────────────────┘
```

---

## 模块划分

### 1. Connection Module
- 连接配置管理
- 连接池
- SSH/SSL/TLS 支持
- Cluster/Sentinel 模式

### 2. Data Module
- Key-Value 操作
- 数据类型处理器
- SCAN 迭代器
- 批量操作

### 3. UI Module
- 主窗口布局
- 连接面板
- Key 浏览器
- 数据编辑器
- CLI 终端
- 设置面板

### 4. State Module
- 全局状态
- 连接状态
- UI 状态持久化

### 5. Utils Module
- 日志系统
- 配置管理
- 主题系统
- 国际化 (i18n)

---

## Cargo.toml 依赖

```toml
[package]
name = "rust-redis-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
# GUI
dioxus = "0.6"
freya = "0.2"

# Async
tokio = { version = "1", features = ["full"] }

# Redis
redis = { version = "0.27", features = ["tokio-comp", "cluster", "cluster-async"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Utils
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio-test = "0.4"
```

---

## 开发路线

### Phase 1: MVP (Week 1-2)
- [x] 项目结构搭建
- [ ] 连接管理（直连模式）
- [ ] Key 树形浏览
- [ ] 基础数据类型 CRUD (String, Hash, List, Set, ZSet)
- [ ] CLI 命令执行

### Phase 2: Core Features (Week 3-4)
- [ ] SSH/SSL 连接
- [ ] Cluster/Sentinel 支持
- [ ] 10万+ keys 虚拟滚动
- [ ] 数据导入/导出

### Phase 3: Advanced Features (Week 5-6)
- [ ] Stream/JSON 支持
- [ ] 慢查询日志
- [ ] 实时监控
- [ ] AI 辅助功能

### Phase 4: Polish (Week 7-8)
- [ ] 主题系统
- [ ] 多语言支持
- [ ] 性能优化
- [ ] 文档完善

---

## 性能目标

| 指标 | 目标值 |
|------|--------|
| 冷启动时间 | < 100ms |
| 热启动时间 | < 50ms |
| 内存占用 | < 20MB |
| 10万 keys 渲染 | < 500ms |
| 页面切换 | < 16ms (60fps) |

---

## 参考资料

- [Dioxus 官方文档](https://dioxuslabs.com/)
- [Freya 官方文档](https://freyaui.dev/)
- [Redis 命令参考](https://redis.io/commands/)
- [Another Redis Desktop Manager](https://github.com/qishibo/AnotherRedisDesktopManager)
- [Redis Insight](https://redis.com/redis-enterprise/redis-insight/)