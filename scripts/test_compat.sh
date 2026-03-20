#!/bin/bash

CONFIG_FILE=~/Library/Application\ Support/rust-redis-desktop/config.json

echo "=== 测试配置兼容性 ==="
echo ""

echo "1. 当前配置文件:"
cat "$CONFIG_FILE" | python3 -m json.tool 2>/dev/null || echo "文件不存在或格式错误"
echo ""

echo "2. 备份配置文件"
cp "$CONFIG_FILE" "$CONFIG_FILE.bak"
echo "备份到: $CONFIG_FILE.bak"
echo ""

echo "3. 启动应用..."
echo "   - 左侧应显示旧连接 'Lssc-uat'"
echo "   - 尝试创建新连接 'Test'"
echo "   - 保存后应显示 2 个连接"
echo ""
echo "按 Ctrl+C 退出测试"

cargo run --release 2>&1 | grep -E "(INFO|ERROR|connections)"