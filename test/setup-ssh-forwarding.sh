#!/bin/bash
# setup-ssh-forwarding.sh - 设置允许端口转发的 SSH 服务器

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "创建自定义 SSH 配置..."

# 创建自定义 SSH 配置目录
mkdir -p ssh-config

# 创建 sshd_config 允许端口转发
cat > ssh-config/sshd_config << 'EOF'
Port 2222
PermitRootLogin yes
PasswordAuthentication yes
PubkeyAuthentication yes
AllowTcpForwarding yes
GatewayPorts yes
PermitTunnel yes
Subsystem sftp /usr/lib/ssh/sftp-server
EOF

echo "✓ SSH 配置已创建"
echo ""
echo "需要重新创建 SSH 容器以应用配置..."
echo "运行: docker-compose stop ssh-server && docker-compose rm -f ssh-server"
echo "然后: docker-compose up -d ssh-server"