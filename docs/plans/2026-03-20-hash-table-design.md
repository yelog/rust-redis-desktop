# Hash Table Design

**背景**

当前 hash 类型 key 在详情区按卡片列表展示，只支持查看，不支持搜索、新增、复制、编辑或删除 field。

**目标**

将 hash 内容改为表格展示，列为 `ID`、`key`、`value`、`action`，并支持以下能力：
- 按关键字过滤 field key 和 value
- 新增一行 hash field
- 复制 value
- 编辑 key 和 value
- 删除 field，保留二次确认

**设计**

- 在 `ValueViewer` 的 `KeyType::Hash` 分支内直接实现表格视图，不新增独立页面。
- 维持现有 Redis 操作入口，复用 `hash_set_field` 与 `hash_delete_field`。
- 搜索为前端本地过滤，不额外请求 Redis。
- 编辑为行级内联编辑：
  - 已有行允许同时修改 field 与 value
  - 若 field 名变化，保存时先删除旧 field，再写入新 field
- 新增行为表格顶部插入一条临时编辑行，保存后写入 Redis 并刷新表格
- 删除操作显示确认弹窗，确认后执行 `hdel`
- 复制操作优先使用桌面端可用的剪贴板能力，将 value 写入系统剪贴板

**状态与反馈**

- Hash 数据加载后转为稳定排序的行列表，保证表格顺序可预期
- 每行维护独立的编辑态和操作态，避免全表锁死
- 顶部显示搜索框、新增按钮与字段总数
- 对保存、删除、复制结果显示轻量状态文案

**验证**

- `cargo check`
- 手动验证搜索、新增、复制、编辑、删除确认与刷新结果
