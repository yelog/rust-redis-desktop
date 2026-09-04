# 性能优化报告

> 本文中的性能数字只有在 `docs/performance/key-browser-baseline.md` 中记录了具体环境和可复现实测结果后才能作为产品承诺。当前实现仍会把扫描结果和树模型保存在内存中，不应宣传为恒定内存或千万级 Key 支持。

## 优化目标

当前目标是让 **10万+ keys** 在可控模式下保持可用，并通过逐步扫描降低一次性加载风险。

## 已实现的优化

### 1. 虚拟滚动 (Virtual Scrolling) ✅

**问题**：
- 传统列表渲染所有 DOM 节点
- 10万 keys = 10万个 DOM 节点
- 内存占用高，渲染慢，滚动卡顿

**解决方案**：
```rust
// 只渲染可见区域 + 缓冲区
let start_index = (scroll_offset / item_height - overscan).max(0);
let end_index = (start_index + visible_count + overscan * 2).min(total);

// 渲染数量固定为 ~40 项
```

**效果**：
- **内存**: O(40) 而非 O(100,000)
- **渲染时间**: ~16ms (常数时间)
- **滚动**: 60fps 流畅

### 2. 增量扫描 (Incremental Scanning) ✅

**问题**：
- SCAN 一次性加载所有键到内存
- 大量键导致：
  - 内存峰值高
  - 加载时间长
  - UI 阻塞

**解决方案**：
```rust
// 分批加载，每批 500 keys
pub async fn scan_keys_with_progress<F>(
    &self,
    pattern: &str,
    batch_size: usize,
    on_batch: F,
) -> Result<usize>
```

**当前边界**：
- Redis 请求按批执行，避免单个请求返回全部 Key。
- 当前 UI 最终仍会构建完整的内存 Key 树，内存并非恒定。
- 大库确认、渐进上限和完整磁盘索引正在逐步实施。

### 3. 性能测试工具 ✅

创建了测试脚本：
```bash
scripts/generate_test_keys.sh
```

可生成 10万 测试键用于性能验证。

## 性能对比

以下历史数字没有统一基准环境和当前版本的可复测证据，暂不作为承诺：

| 指标 | 优化前 | 优化后 | 改进 |
|------|--------|--------|------|
| **10万 keys 内存** | ~500MB | ~50MB | **90% ↓** |
| **首次渲染** | 2-5秒 | <100ms | **50x ⚡** |
| **滚动帧率** | 5-15 fps | 60 fps | **10x ⚡** |
| **启动时间** | ~500ms | ~100ms | **5x ⚡** |

## 测试方法

推荐使用 `scripts/generate-benchmark-data.sh` 和 `scripts/measure-key-browser.sh` 生成并记录结果。

### 1. 生成测试数据

```bash
# 启动 Redis
redis-server

# 生成 10万测试键
./scripts/generate_test_keys.sh

# 或手动生成
redis-cli --eval generate_keys.lua 0 100000
```

### 2. 运行应用测试

```bash
# 编译发布版本
cargo build --release

# 运行
./target/release/rust-redis-desktop
```

### 3. 验证性能

1. 连接到本地 Redis
2. 观察键加载速度
3. 测试滚动流畅度
4. 检查内存占用

```bash
# 监控内存
top | grep rust-redis-desktop

# 或使用 Instruments (macOS)
instruments -t "Allocations" ./target/release/rust-redis-desktop
```

## 待优化项目

### 优先级：高

- [ ] 大数据库扫描前确认和可恢复的渐进扫描
- [ ] 使用临时磁盘索引代替完整内存 Key 树
- [ ] 对 Hash/List/Set/ZSet/Stream 表格执行虚拟行渲染

- [ ] **树节点懒加载**
  - 只在展开时加载子节点
  - 减少初始树构建时间
  - 适合深层嵌套结构

- [ ] **键值缓存**
  - 缓存最近访问的键值
  - 减少网络请求
  - 提升二次访问速度

### 优先级：低

- [ ] **后台预加载**
  - 预测用户可能访问的键
  - 后台异步加载
  - 进一步提升响应速度

- [ ] **虚拟化树结构**
  - 对树形结构也实现虚拟滚动
  - 支持超大规模树 (100万+ 节点)

## 性能最佳实践

### Redis 配置

```redis
# 建议 Redis 配置
maxmemory 2gb
maxmemory-policy allkeys-lru
```

### 应用使用建议

1. **大量键场景 (>10万)**
   - 使用搜索过滤
   - 设置合理的键分隔符 (如 `:`)
   - 避免一次性展开所有节点

2. **网络延迟高**
   - 减小批量大小 (默认 500)
   - 使用连接池
   - 考虑使用 SSH 隧道

3. **内存受限**
   - 启用虚拟滚动 (已默认)
   - 定期刷新键列表
   - 关闭不用的连接

## 监控指标

应用内置性能指标 (待实现):

```rust
struct PerformanceMetrics {
    keys_loaded: usize,
    render_time_ms: u64,
    memory_usage_mb: f64,
    scroll_fps: f64,
}
```

可通过快捷键 `Ctrl+Shift+P` (计划中) 显示性能面板。

---

**性能优化是一个持续过程**。在完成实测和磁盘索引之前，请将当前版本描述为支持分批扫描和虚拟 Key 列表，而不是恒定内存或千万级场景。
