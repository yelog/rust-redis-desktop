# 测试连接保存功能

## 当前状态

配置文件位置：
```
~/Library/Application Support/rust-redis-desktop/config.json
```

当前配置：
```json
{
  "connections": [
    {
      "id": "8e8eb7ad-ccff-45e4-8979-b2302fbc14c3",
      "name": "Lssc-uat",
      "host": "10.188.132.29",
      "port": 30379
    }
  ]
}
```

## 测试步骤

### 1. 启动应用

```bash
cargo run --release
```

### 2. 检查初始状态

在左侧侧边栏顶部，应该显示：
- "1 connection(s)" 或
- "No connections"

### 3. 创建新连接

1. 点击 "+ New Connection" 按钮
2. 填写表单：
   - Name: "Test Local"
   - Host: "127.0.0.1"
   - Port: 6379
3. 点击 "💾 Save" 按钮

### 4. 验证结果

**期望结果：**
- 表单关闭
- 左侧列表显示 "2 connection(s)"
- 列表中出现 "Test Local" 连接
- 如果鼠标悬停，显示 Edit/Delete 按钮

**如果列表没有更新：**
1. 重启应用
2. 查看是否显示 "2 connection(s)"
3. 如果显示了，说明保存成功但 UI 更新有问题
4. 如果没有，说明配置文件写入失败

### 5. 检查配置文件

```bash
cat ~/Library/Application\ Support/rust-redis-desktop/config.json | python3 -m json.tool
```

应该看到两个连接配置。

## 调试方法

### 查看日志

```bash
RUST_LOG=info cargo run --release 2>&1 | tee debug.log
```

保存连接时应该看到：
```
Saving connection: Test Local (uuid...)
Config saved
Updating connections list: [(uuid, "Test Local"), ...]
```

### 手动检查配置文件

```bash
# 查看配置文件
cat ~/Library/Application\ Support/rust-redis-desktop/config.json

# 查看文件更新时间
ls -la ~/Library/Application\ Support/rust-redis-desktop/
```

## 已知问题

如果列表不显示，请告诉我：
1. 侧边栏顶部显示的连接数量
2. 配置文件的内容
3. 是否有任何错误日志

这样我可以快速定位问题。