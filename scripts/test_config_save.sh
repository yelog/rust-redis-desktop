#!/bin/bash

CONFIG_FILE=~/Library/Application\ Support/rust-redis-desktop/config.json

echo "=== 测试配置文件写入 ==="
echo ""
echo "1. 当前配置文件内容:"
cat "$CONFIG_FILE" 2>/dev/null | python3 -m json.tool || echo "文件不存在"
echo ""

echo "2. 请在应用中执行以下操作:"
echo "   - 点击 '+ New Connection'"
echo "   - 填写名称: 'Test Connection'"
echo "   - 填写主机: '127.0.0.1'"
echo "   - 填写端口: 6379"
echo "   - 点击 '💾 Save'"
echo ""
echo "按回车键继续..."
read

echo ""
echo "3. 保存后配置文件内容:"
cat "$CONFIG_FILE" 2>/dev/null | python3 -m json.tool || echo "文件不存在"
echo ""

echo "=== 测试完成 ==="