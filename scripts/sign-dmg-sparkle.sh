#!/bin/bash

# Sparkle DMG 签名脚本
# 使用 EdDSA 私钥对 DMG 文件进行签名

set -e

DMG_FILE="${1:?需要提供 DMG 文件路径}"
PRIVATE_KEY="${SPARKLE_PRIVATE_KEY:-}"

if [ -z "$PRIVATE_KEY" ]; then
    echo "错误: 未设置 SPARKLE_PRIVATE_KEY 环境变量"
    exit 1
fi

echo "正在签名: $DMG_FILE"

# 检查签名工具
if command -v sign_update &> /dev/null; then
    SIGN_TOOL="sign_update"
elif [ -f "/usr/local/bin/sign_update" ]; then
    SIGN_TOOL="/usr/local/bin/sign_update"
elif [ -f "./Sparkle/bin/sign_update" ]; then
    SIGN_TOOL="./Sparkle/bin/sign_update"
else
    echo "未找到 sign_update 工具，使用 Python 脚本签名"
    
    # 使用 Python 签名
    python3 scripts/sign_update_python.py "$DMG_FILE" "$PRIVATE_KEY"
    exit 0
fi

# 使用 Sparkle 工具签名
echo "签名结果:"
$SIGN_TOOL -f "$DMG_FILE" -p "$PRIVATE_KEY"

echo ""
echo "签名完成"