#!/bin/bash
# start-all-services.sh - 启动所有测试服务

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=========================================="
echo "  启动 Redis 测试服务"
echo "==========================================${NC}"

# 检查 Docker 是否运行
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}错误: Docker 未运行${NC}"
    exit 1
fi

# 创建 SSL 证书目录（如果不存在）
if [ ! -d "redis-ssl-certs" ]; then
    echo -e "${YELLOW}创建 SSL 证书目录...${NC}"
    mkdir -p redis-ssl-certs
    
    # 生成自签名证书
    openssl req -x509 -newkey rsa:2048 -keyout redis-ssl-certs/redis.key -out redis-ssl-certs/redis.crt -days 365 -nodes -subj "/CN=localhost" 2>/dev/null
    cp redis-ssl-certs/redis.crt redis-ssl-certs/ca.crt
    echo -e "${GREEN}✓ SSL 证书已生成${NC}"
fi

# 创建 SSH 密钥（如果需要）
if [ ! -f "ssh-keys/id_rsa" ]; then
    echo -e "${YELLOW}生成 SSH 密钥...${NC}"
    mkdir -p ssh-keys
    ssh-keygen -t rsa -b 2048 -f ssh-keys/id_rsa -N "" -C "redis-test" 2>/dev/null
    cp ssh-keys/id_rsa.pub ssh-keys/authorized_keys
    chmod 600 ssh-keys/id_rsa
    chmod 644 ssh-keys/authorized_keys
    echo -e "${GREEN}✓ SSH 密钥已生成${NC}"
fi

# 选择启动的服务
SERVICE_TYPE="${1:-basic}"

case "$SERVICE_TYPE" in
    basic)
        SERVICES="redis-standalone redis-test-data"
        echo -e "${YELLOW}启动基本服务: redis-standalone${NC}"
        ;;
    all)
        SERVICES="redis-standalone redis-stack redis-cluster-node-1 redis-cluster-node-2 redis-cluster-node-3 redis-cluster-node-4 redis-cluster-node-5 redis-cluster-node-6 redis-cluster-init redis-master redis-slave redis-sentinel-1 redis-ssl redis-test-data"
        echo -e "${YELLOW}启动所有服务...${NC}"
        ;;
    cluster)
        SERVICES="redis-cluster-node-1 redis-cluster-node-2 redis-cluster-node-3 redis-cluster-node-4 redis-cluster-node-5 redis-cluster-node-6 redis-cluster-init"
        echo -e "${YELLOW}启动 Cluster 服务...${NC}"
        ;;
    sentinel)
        SERVICES="redis-master redis-slave redis-sentinel-1"
        echo -e "${YELLOW}启动 Sentinel 服务...${NC}"
        ;;
    ssl)
        SERVICES="redis-ssl"
        echo -e "${YELLOW}启动 SSL 服务...${NC}"
        ;;
    ssh)
        SERVICES="redis-ssh-internal ssh-server"
        echo -e "${YELLOW}启动 SSH 隧道服务...${NC}"
        ;;
    *)
        echo "用法: $0 [basic|all|cluster|sentinel|ssl|ssh]"
        exit 1
        ;;
esac

# 启动服务
docker-compose up -d $SERVICES

echo ""
echo -e "${GREEN}等待服务启动...${NC}"
sleep 3

# 显示服务状态
echo ""
echo -e "${BLUE}=== 服务状态 ===${NC}"
docker-compose ps

# 验证基本连接
echo ""
echo -e "${BLUE}=== 连接测试 ===${NC}"

if echo "$SERVICES" | grep -q "redis-standalone"; then
    if redis-cli -h localhost -p 6379 ping 2>/dev/null | grep -q PONG; then
        echo -e "${GREEN}✓ Direct 连接 (6379): 正常${NC}"
    else
        echo -e "${YELLOW}⚠ Direct 连接 (6379): 等待中...${NC}"
    fi
fi

if echo "$SERVICES" | grep -q "redis-stack"; then
    if redis-cli -h localhost -p 6383 ping 2>/dev/null | grep -q PONG; then
        echo -e "${GREEN}✓ Redis Stack (6383): 正常${NC}"
    else
        echo -e "${YELLOW}⚠ Redis Stack (6383): 等待中...${NC}"
    fi
fi

if echo "$SERVICES" | grep -q "redis-sentinel-1"; then
    if redis-cli -h localhost -p 26379 ping 2>/dev/null | grep -q PONG; then
        echo -e "${GREEN}✓ Sentinel (26379): 正常${NC}"
    else
        echo -e "${YELLOW}⚠ Sentinel (26379): 等待中...${NC}"
    fi
fi

if echo "$SERVICES" | grep -q "redis-cluster-node-1"; then
    if redis-cli -h localhost -p 7000 ping 2>/dev/null | grep -q PONG; then
        echo -e "${GREEN}✓ Cluster 节点 (7000): 正常${NC}"
    else
        echo -e "${YELLOW}⚠ Cluster 节点 (7000): 等待中...${NC}"
    fi
fi

if echo "$SERVICES" | grep -q "redis-ssl"; then
    if redis-cli -h localhost -p 6382 ping 2>/dev/null | grep -q PONG; then
        echo -e "${GREEN}✓ SSL (6382): 正常${NC}"
    else
        echo -e "${YELLOW}⚠ SSL (6382): 等待中...${NC}"
    fi
fi

echo ""
echo -e "${GREEN}=========================================="
echo "  服务启动完成!"
echo "==========================================${NC}"
echo ""
echo "运行测试:"
echo "  ./run-tests.sh          # 运行完整测试"
echo "  ./test-connections.sh   # 测试连接模式"
echo ""
echo "停止服务:"
echo "  docker-compose down"