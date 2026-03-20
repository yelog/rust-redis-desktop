# Hash Table Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 hash key 详情区改为可搜索、可新增、可复制、可编辑、可删除的表格视图。

**Architecture:** 在 `ValueViewer` 中直接实现 hash 表格与行级交互状态。继续复用现有 Redis 命令，保存或删除后重新拉取 hash 数据，保证界面与服务端一致。

**Tech Stack:** Rust, Dioxus Desktop, redis async client

---

### Task 1: 重构 hash 数据展示为表格

**Files:**
- Modify: `src/ui/value_viewer.rs`

**Step 1: 定义 hash 行结构与排序逻辑**

- 将 `HashMap<String, String>` 转为稳定排序的行列表用于渲染
- 保留原始 `HashMap` 作为加载结果缓存

**Step 2: 添加顶部工具栏**

- 添加搜索输入框
- 添加“新增行”按钮
- 添加字段数与操作状态提示

**Step 3: 渲染表头与表体**

- 固定列：`ID`、`key`、`value`、`action`
- 空结果时显示空状态

### Task 2: 实现复制、编辑与新增

**Files:**
- Modify: `src/ui/value_viewer.rs`

**Step 1: 复制 value**

- 接入桌面端剪贴板能力
- 点击后更新状态提示

**Step 2: 行级编辑**

- 普通行支持进入编辑态
- 允许同时修改 field 和 value
- 保存时：
  - field 未变化：直接 `hset`
  - field 已变化：先 `hdel(old)` 再 `hset(new, value)`

**Step 3: 新增行**

- 顶部插入临时新行
- 校验 field 非空且不与现有 field 冲突
- 保存成功后退出新增态并刷新

### Task 3: 实现删除确认与刷新

**Files:**
- Modify: `src/ui/value_viewer.rs`

**Step 1: 删除确认弹窗**

- 点击删除图标后弹窗确认
- 确认后执行 `hash_delete_field`

**Step 2: 刷新当前 hash 数据**

- 将 hash 加载逻辑提取为可复用闭包/辅助函数
- 保存、新增、删除成功后调用刷新

### Task 4: 验证

**Files:**
- Modify: `src/ui/value_viewer.rs`

**Step 1: 运行检查**

Run: `cargo check`
Expected: PASS

**Step 2: 检查交互边界**

- 搜索过滤是否覆盖 key/value
- 新增与重命名 field 是否正确处理冲突
- 删除确认与复制反馈是否生效
