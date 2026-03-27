#!/bin/bash
# test-ssh-tunnel.sh - 测试 SSH 隧道连接到 Redis

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS=0
FAIL=0

echo -e "${BLUE}=========================================="
echo "  SSH 隧道连接测试"
echo "==========================================${NC}"

# 检查 SSH 密钥
if [ ! -f "ssh-keys/id_rsa" ]; then
    echo -e "${RED}错误: SSH 密钥不存在${NC}"
    echo "请先运行: ./setup-ssh-test.sh"
    exit 1
fi

# 检查 SSH 服务器
echo -e "\n${YELLOW}检查 SSH 服务器...${NC}"
if docker ps --filter "name=ssh-server" --format "{{.Names}}" | grep -q "ssh-server"; then
    echo -e "${GREEN}✓ SSH 服务器运行中${NC}"
    ((PASS++))
else
    echo -e "${RED}✗ SSH 服务器未运行${NC}"
    ((FAIL++))
    exit 1
fi

# 检查内部 Redis
echo -e "\n${YELLOW}检查内部 Redis 服务...${NC}"
if docker ps --filter "name=redis-ssh-internal" --format "{{.Names}}" | grep -q "redis-ssh-internal"; then
    echo -e "${GREEN}✓ 内部 Redis 运行中${NC}"
    ((PASS++))
else
    echo -e "${RED}✗ 内部 Redis 未运行${NC}"
    ((FAIL++))
    exit 1
fi

# 测试 SSH 连接
echo -e "\n${YELLOW}测试 SSH 连接...${NC}"
SSH_TEST=$(ssh -i ssh-keys/id_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o PasswordAuthentication=no -p 2222 redis-test@localhost 'echo OK' 2>&1)
if echo "$SSH_TEST" | grep -q "OK"; then
    echo -e "${GREEN}✓ SSH 连接成功${NC}"
    ((PASS++))
else
    echo -e "${RED}✗ SSH 连接失败${NC}"
    echo "$SSH_TEST"
    ((FAIL++))
fi

# 创建 SSH 隧道
echo -e "\n${YELLOW}创建 SSH 隧道...${NC}"
LOCAL_PORT=16379

# 检查端口是否已被占用
if lsof -i :$LOCAL_PORT > /dev/null 2>&1; then
    echo -e "${YELLOW}端口 $LOCAL_PORT 已被占用，尝试使用 16380${NC}"
    LOCAL_PORT=16380
fi

# 启动 SSH 隧道（后台运行）
ssh -i ssh-keys/id_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o PasswordAuthentication=no -p 2222 -N -L $LOCAL_PORT:redis-ssh-internal:6379 redis-test@localhost &
SSH_PID=$!

# 等待隧道建立
sleep 2

# 检查隧道是否建立
if ps -p $SSH_PID > /dev/null 2>&1; then
    echo -e "${GREEN}✓ SSH 隧道已建立 (本地端口: $LOCAL_PORT)${NC}"
    ((PASS++))
else
    echo -e "${RED}✗ SSH 隧道建立失败${NC}"
    ((FAIL++))
fi

# 通过隧道测试 Redis 连接
echo -e "\n${YELLOW}通过 SSH 隧道测试 Redis 连接...${NC}"
if redis-cli -h localhost -p $LOCAL_PORT PING 2>/dev/null | grep -q PONG; then
    echo -e "${GREEN}✓ Redis PING 成功${NC}"
    ((PASS++))
    
    # 测试写入
    redis-cli -h localhost -p $LOCAL_PORT SET "test:ssh:tunnel" "ssh-tunnel-test" > /dev/null 2>&1
    VALUE=$(redis-cli -h localhost -p $LOCAL_PORT GET "test:ssh:tunnel" 2>/dev/null)
    if [ "$VALUE" = "ssh-tunnel-test" ]; then
        echo -e "${GREEN}✓ Redis SET/GET 成功${NC}"
        ((PASS++))
    else
        echo -e "${RED}✗ Redis SET/GET 失败${NC}"
        ((FAIL++))
    fi
    
    # 清理
    redis-cli -h localhost -p $LOCAL_PORT DEL "test:ssh:tunnel" > /dev/null 2>&1
else
    echo -e "${RED}✗ Redis PING 失败${NC}"
    ((FAIL++))
fi

# 关闭 SSH 隧道
kill $SSH_PID 2>/dev/null
echo -e "\n${YELLOW}SSH 隧道已关闭${NC}"

# 结果汇总
echo -e "\n${BLUE}=========================================="
echo "  测试结果汇总"
echo "==========================================${NC}"
echo -e "${GREEN}通过: $PASS${NC}"
echo -e "${RED}失败: $FAIL${NC}"

if [ $FAIL -gt 0 ]; then
    echo -e "\n${RED}存在失败的测试项${NC}"
    exit 1
else
    echo -e "\n${GREEN}所有测试通过!${NC}"
    exit 0
fi