#!/bin/bash

# 下载并设置 Sparkle Framework
# 用于 macOS 应用的自动更新

set -e

SPARKLE_VERSION="2.6.4"
SPARKLE_URL="https://github.com/sparkle-project/Sparkle/releases/download/${SPARKLE_VERSION}/Sparkle-${SPARKLE_VERSION}.tar.xz"

FRAMEWORKS_DIR="Frameworks"
SPARKLE_FRAMEWORK="Sparkle.framework"

echo "=== 设置 Sparkle Framework ==="
echo "版本: $SPARKLE_VERSION"

# 创建 Frameworks 目录
mkdir -p "$FRAMEWORKS_DIR"

# 下载 Sparkle
if [ ! -d "$FRAMEWORKS_DIR/$SPARKLE_FRAMEWORK" ]; then
    echo "下载 Sparkle..."
    
    TEMP_FILE="/tmp/sparkle-${SPARKLE_VERSION}.tar.xz"
    TEMP_DIR="/tmp/sparkle-${SPARKLE_VERSION}"
    
    # 下载
    curl -L -o "$TEMP_FILE" "$SPARKLE_URL"
    
    # 解压
    mkdir -p "$TEMP_DIR"
    tar -xf "$TEMP_FILE" -C "$TEMP_DIR"
    
    # 复制 Framework
    cp -R "$TEMP_DIR/Sparkle.framework" "$FRAMEWORKS_DIR/"
    
    # 清理
    rm -f "$TEMP_FILE"
    rm -rf "$TEMP_DIR"
    
    echo "Sparkle.framework 已下载"
else
    echo "Sparkle.framework 已存在"
fi

# 输出签名工具路径
echo ""
echo "签名工具路径: $FRAMEWORKS_DIR/$SPARKLE_FRAMEWORK/Versions/A/Resources/sign_update"
echo ""
echo "设置完成!"