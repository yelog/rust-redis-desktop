#!/bin/bash
# setup-ssh-test.sh - 设置 SSH 隧道测试环境

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== 设置 SSH 隧道测试环境 ==="

# 生成测试密钥对（如果不存在）
if [ ! -f "$SCRIPT_DIR/ssh-keys/id_rsa" ]; then
    echo "生成 SSH 测试密钥对..."
    mkdir -p "$SCRIPT_DIR/ssh-keys"
    ssh-keygen -t rsa -b 2048 -f "$SCRIPT_DIR/ssh-keys/id_rsa" -N "" -C "redis-test"
    chmod 600 "$SCRIPT_DIR/ssh-keys/id_rsa"
    
    # 创建 authorized_keys
    cp "$SCRIPT_DIR/ssh-keys/id_rsa.pub" "$SCRIPT_DIR/ssh-keys/authorized_keys"
    chmod 644 "$SCRIPT_DIR/ssh-keys/authorized_keys"
    echo "✓ SSH 密钥对已生成"
else
    echo "✓ SSH 密钥对已存在"
fi

# 创建 SSH 测试连接配置
cat > "$SCRIPT_DIR/ssh-test-config.env" << EOF
# SSH 隧道测试配置
SSH_HOST=localhost
SSH_PORT=2222
SSH_USER=redis-test
SSH_PASSWORD=redis-test
SSH_KEY_PATH=$SCRIPT_DIR/ssh-keys/id_rsa

# Redis 配置（通过 SSH 隧道访问）
REDIS_HOST=redis-ssh-internal
REDIS_PORT=6379
EOF

echo ""
echo "=== SSH 测试环境配置 ==="
echo "SSH 服务器: localhost:2222"
echo "用户名: redis-test"
echo "密码: redis-test"
echo "私钥: $SCRIPT_DIR/ssh-keys/id_rsa"
echo ""
echo "要启动 SSH 测试环境，运行："
echo "  cd test && docker-compose up -d ssh-server redis-ssh-internal"
echo ""
echo "测试 SSH 连接："
echo "  ssh -i $SCRIPT_DIR/ssh-keys/id_rsa -p 2222 redis-test@localhost"