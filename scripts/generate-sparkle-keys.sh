#!/bin/bash

# Sparkle EdDSA 密钥生成脚本
# 用于生成更新签名所需的公钥和私钥

set -e

echo "=== Sparkle EdDSA 密钥生成 ==="
echo ""
echo "此脚本将生成用于 Sparkle 更新签名的 EdDSA 密钥对"
echo ""

# 检查是否已安装 Sparkle 工具
# 可以从 https://github.com/sparkle-project/Sparkle/releases 下载

if command -v generate_keys &> /dev/null; then
    KEY_TOOL="generate_keys"
elif [ -f "/usr/local/bin/generate_keys" ]; then
    KEY_TOOL="/usr/local/bin/generate_keys"
elif [ -f "./Sparkle/bin/generate_keys" ]; then
    KEY_TOOL="./Sparkle/bin/generate_keys"
else
    echo "未找到 generate_keys 工具"
    echo "请从 https://github.com/sparkle-project/Sparkle/releases 下载 Sparkle"
    echo "并将 generate_keys 放到 /usr/local/bin/ 或 ./Sparkle/bin/"
    echo ""
    echo "或者使用以下方法手动生成密钥:"
    echo ""
    echo "方法 1: 使用 Python (需要安装 pynacl)"
    echo "  pip install pynacl"
    echo "  python scripts/generate_keys_python.py"
    echo ""
    echo "方法 2: 使用 OpenSSL (生成 Ed25519 密钥)"
    echo "  openssl genpkey -algorithm ED25519 -out sparkle_private_key.pem"
    echo "  openssl pkey -in sparkle_private_key.pem -pubout -out sparkle_public_key.pem"
    echo ""
    exit 1
fi

echo "使用工具: $KEY_TOOL"
echo ""

# 生成密钥
echo "正在生成密钥..."
$KEY_TOOL

echo ""
echo "=== 密钥生成完成 ==="
echo ""
echo "请将以下信息保存:"
echo ""
echo "1. 私钥 (SPARKLE_PRIVATE_KEY): 存储到 GitHub Secrets"
echo "   - 用于 CI 中签名更新包"
echo "   - 请妥善保管，不要泄露"
echo ""
echo "2. 公钥 (SPARKLE_PUBLIC_KEY): 嵌入到应用中"
echo "   - 在 Info.plist 中配置 SUPublicEDKey"
echo "   - 用户验证更新包的签名"
echo ""
echo "GitHub Secrets 配置:"
echo "   Repository -> Settings -> Secrets and variables -> Actions"
echo "   添加 SPARKLE_PRIVATE_KEY"
echo ""