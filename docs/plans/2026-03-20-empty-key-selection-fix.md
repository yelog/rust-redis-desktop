# Empty Key Selection Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复以分隔符结尾的 Redis key 在树视图中与父节点共享选中态的问题，仅高亮 `[空]` 叶子节点。

**Architecture:** 为树节点拆分“UI 唯一身份”和“实际 Redis 路径”两个字段。展开态与组件 key 使用唯一身份，值加载、复制和删除继续使用实际 Redis key 或目录前缀。

**Tech Stack:** Rust, Dioxus, Redis key tree builder

---

### Task 1: 拆分树节点身份字段

**Files:**
- Modify: `src/redis/types.rs`
- Modify: `src/redis/tree.rs`

**Step 1: 添加树节点唯一身份字段**

给 `TreeNode` 增加 `node_id` 和 `path` 字段，保留现有 `name`、`is_leaf`、`children` 等语义。

**Step 2: 调整树构建逻辑**

普通目录节点使用稳定且唯一的 `folder:` 前缀构造 `node_id`，叶子节点使用 `leaf:` 前缀构造 `node_id`。`path` 对叶子节点表示真实 Redis key，对目录节点表示扫描/删除用的前缀。

### Task 2: 切换 UI 到新的字段

**Files:**
- Modify: `src/ui/lazy_tree_node.rs`
- Modify: `src/ui/key_item.rs`
- Modify: `src/ui/virtual_key_list.rs`
- Modify: `src/ui/key_browser.rs`

**Step 1: 选中态仅对叶子节点生效**

树节点高亮判断改为 `node.is_leaf && selected_key == node.path`。

**Step 2: 展开态和组件 key 使用 `node_id`**

避免目录节点与 `[空]` 叶子共享同一个 identity。

**Step 3: 上下文菜单继续使用 `path`**

复制 key、删除 key、目录删除扫描仍然基于真实 Redis key 或前缀。

### Task 3: 添加回归测试

**Files:**
- Modify: `src/redis/tree.rs`

**Step 1: 为尾部分隔符 key 添加单元测试**

验证 `cache:sys_file:` 会生成目录节点和 `[空]` 叶子节点，且二者 `node_id` 不同、`path` 相同。

**Step 2: 运行测试**

执行相关 `cargo test`，确认构建逻辑未回归。
