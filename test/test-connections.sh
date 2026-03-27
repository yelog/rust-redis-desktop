#!/bin/bash
# test-connections.sh - 测试各种连接模式
# 用法: ./test-connections.sh [connection_type]
# connection_type: all, direct, cluster, sentinel, ssl

set -e

REDIS_HOST="${REDIS_HOST:-localhost}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS=0
FAIL=0

# 测试连接类型
TEST_TYPE="${1:-all}"

echo -e "${BLUE}=========================================="
echo "  Redis 连接模式测试"
echo "==========================================${NC}"

# 检查 redis-cli 是否可用
if ! command -v redis-cli &> /dev/null; then
    echo -e "${RED}错误: redis-cli 未安装${NC}"
    exit 1
fi

test_connection() {
    local name=$1
    local host=$2
    local port=$3
    local extra_args=$4
    
    echo -e "\n${YELLOW}测试: $name${NC}"
    echo "  地址: $host:$port"
    
    # 测试 PING
    if redis-cli -h $host -p $port $extra_args PING 2>/dev/null | grep -q PONG; then
        echo -e "  ${GREEN}✓ PING 成功${NC}"
        ((PASS++))
    else
        echo -e "  ${RED}✗ PING 失败${NC}"
        ((FAIL++))
        return 1
    fi
    
    # 测试 INFO
    if redis-cli -h $host -p $port $extra_args INFO server 2>/dev/null | grep -q "redis_version"; then
        local version=$(redis-cli -h $host -p $port $extra_args INFO server 2>/dev/null | grep "redis_version:" | cut -d: -f2 | tr -d '\r')
        echo -e "  ${GREEN}✓ INFO 成功 (Redis $version)${NC}"
        ((PASS++))
    else
        echo -e "  ${RED}✗ INFO 失败${NC}"
        ((FAIL++))
    fi
    
    # 测试 SET/GET
    local test_key="test:connection:$name"
    local test_value="connection-test-$(date +%s)"
    redis-cli -h $host -p $port $extra_args SET "$test_key" "$test_value" > /dev/null 2>&1
    local get_result=$(redis-cli -h $host -p $port $extra_args GET "$test_key" 2>/dev/null)
    if [ "$get_result" = "$test_value" ]; then
        echo -e "  ${GREEN}✓ SET/GET 成功${NC}"
        ((PASS++))
        redis-cli -h $host -p $port $extra_args DEL "$test_key" > /dev/null 2>&1
    else
        echo -e "  ${RED}✗ SET/GET 失败${NC}"
        ((FAIL++))
    fi
    
    return 0
}

test_cluster() {
    echo -e "\n${YELLOW}测试: Cluster 连接${NC}"
    
    # 检查集群节点是否运行
    local nodes_running=0
    for port in 7000 7001 7002 7003 7004 7005; do
        if redis-cli -h $REDIS_HOST -p $port PING 2>/dev/null | grep -q PONG; then
            ((nodes_running++))
        fi
    done
    
    if [ $nodes_running -lt 3 ]; then
        echo -e "  ${YELLOW}⚠ 跳过: 集群节点未启动 ($nodes_running/6 运行中)${NC}"
        echo "  提示: 运行 'cd test && docker-compose up -d' 启动集群"
        return 0
    fi
    
    echo "  集群节点: $nodes_running/6 运行中"
    
    # 测试集群连接
    if redis-cli -h $REDIS_HOST -p 7000 CLUSTER INFO 2>/dev/null | grep -q "cluster_state:ok"; then
        echo -e "  ${GREEN}✓ 集群状态正常${NC}"
        ((PASS++))
    else
        echo -e "  ${RED}✗ 集群状态异常${NC}"
        ((FAIL++))
    fi
    
    # 测试集群节点数
    local node_count=$(redis-cli -h $REDIS_HOST -p 7000 CLUSTER NODES 2>/dev/null | wc -l | tr -d ' ')
    if [ "$node_count" = "6" ]; then
        echo -e "  ${GREEN}✓ 集群节点数正确 ($node_count)${NC}"
        ((PASS++))
    else
        echo -e "  ${YELLOW}⚠ 集群节点数: $node_count (预期 6)${NC}"
    fi
    
    # 测试集群写入
    local cluster_key="test:cluster:{hash}:key"
    local cluster_value="cluster-test-$(date +%s)"
    redis-cli -h $REDIS_HOST -p 7000 SET "$cluster_key" "$cluster_value" > /dev/null 2>&1
    local get_result=$(redis-cli -h $REDIS_HOST -p 7000 GET "$cluster_key" 2>/dev/null)
    if [ "$get_result" = "$cluster_value" ]; then
        echo -e "  ${GREEN}✓ 集群写入/读取成功${NC}"
        ((PASS++))
        redis-cli -h $REDIS_HOST -p 7000 DEL "$cluster_key" > /dev/null 2>&1
    else
        echo -e "  ${RED}✗ 集群写入/读取失败${NC}"
        ((FAIL++))
    fi
}

test_sentinel() {
    echo -e "\n${YELLOW}测试: Sentinel 连接${NC}"
    
    # 检查 Sentinel 是否运行
    if ! redis-cli -h $REDIS_HOST -p 26379 PING 2>/dev/null | grep -q PONG; then
        echo -e "  ${YELLOW}⚠ 跳过: Sentinel 未运行${NC}"
        echo "  提示: 运行 'cd test && docker-compose up -d redis-sentinel-1' 启动 Sentinel"
        return 0
    fi
    
    # 测试 Sentinel 连接
    echo -e "  ${GREEN}✓ Sentinel PING 成功${NC}"
    ((PASS++))
    
    # 获取 master 信息
    local master_info=$(redis-cli -h $REDIS_HOST -p 26379 SENTINEL master mymaster 2>/dev/null)
    if [ -n "$master_info" ]; then
        local master_ip=$(echo "$master_info" | grep -A1 "^ip$" | tail -1)
        local master_port=$(echo "$master_info" | grep -A1 "^port$" | tail -1)
        echo -e "  ${GREEN}✓ Master 地址: $master_ip:$master_port${NC}"
        ((PASS++))
    else
        echo -e "  ${RED}✗ 获取 Master 信息失败${NC}"
        ((FAIL++))
    fi
    
    # 获取 slaves 信息
    local slave_count=$(redis-cli -h $REDIS_HOST -p 26379 SENTINEL slaves mymaster 2>/dev/null | grep -c "name")
    if [ "$slave_count" -ge 1 ]; then
        echo -e "  ${GREEN}✓ Slave 数量: $slave_count${NC}"
        ((PASS++))
    else
        echo -e "  ${YELLOW}⚠ 未检测到 Slave${NC}"
    fi
}

test_ssl() {
    echo -e "\n${YELLOW}测试: SSL/TLS 连接${NC}"
    
    # 检查 SSL 证书
    if [ ! -d "$SCRIPT_DIR/redis-ssl-certs" ]; then
        echo -e "  ${YELLOW}⚠ 跳过: SSL 证书目录不存在${NC}"
        echo "  提示: 需要 $SCRIPT_DIR/redis-ssl-certs 目录"
        return 0
    fi
    
    # 检查 Redis SSL 服务是否运行
    if ! redis-cli -h $REDIS_HOST -p 6382 PING 2>/dev/null | grep -q PONG; then
        # 可能需要 TLS 连接
        if redis-cli -h $REDIS_HOST -p 6382 --tls --cacert "$SCRIPT_DIR/redis-ssl-certs/ca.crt" PING 2>/dev/null | grep -q PONG; then
            echo -e "  ${GREEN}✓ SSL/TLS PING 成功${NC}"
            ((PASS++))
            
            # 测试 SSL 写入
            local ssl_key="test:ssl:key"
            local ssl_value="ssl-test-$(date +%s)"
            redis-cli -h $REDIS_HOST -p 6382 --tls --cacert "$SCRIPT_DIR/redis-ssl-certs/ca.crt" SET "$ssl_key" "$ssl_value" > /dev/null 2>&1
            local get_result=$(redis-cli -h $REDIS_HOST -p 6382 --tls --cacert "$SCRIPT_DIR/redis-ssl-certs/ca.crt" GET "$ssl_key" 2>/dev/null)
            if [ "$get_result" = "$ssl_value" ]; then
                echo -e "  ${GREEN}✓ SSL/TLS 写入/读取成功${NC}"
                ((PASS++))
            else
                echo -e "  ${RED}✗ SSL/TLS 写入/读取失败${NC}"
                ((FAIL++))
            fi
        else
            echo -e "  ${YELLOW}⚠ 跳过: Redis SSL 服务未运行${NC}"
            echo "  提示: 运行 'cd test && docker-compose up -d redis-ssl' 启动 SSL 服务"
        fi
    else
        # 非 TLS 模式运行
        test_connection "SSL (非TLS模式)" $REDIS_HOST 6382
    fi
}

test_readonly() {
    echo -e "\n${YELLOW}测试: 只读模式${NC}"
    echo "  注意: 此测试需要应用程序支持只读模式验证"
    
    # 模拟只读模式测试 - 检查从库是否只读
    if redis-cli -h $REDIS_HOST -p 6381 PING 2>/dev/null | grep -q PONG; then
        echo -e "  ${GREEN}✓ Slave (6381) 连接成功${NC}"
        ((PASS++))
        
        # 从库应该是只读的
        local readonly=$(redis-cli -h $REDIS_HOST -p 6381 CONFIG GET slave-read-only 2>/dev/null | tail -1)
        if [ "$readonly" = "yes" ]; then
            echo -e "  ${GREEN}✓ Slave 只读模式已启用${NC}"
            ((PASS++))
        else
            echo -e "  ${YELLOW}⚠ Slave 只读模式状态: $readonly${NC}"
        fi
    else
        echo -e "  ${YELLOW}⚠ 跳过: Slave 未运行${NC}"
    fi
}

test_redis_stack() {
    echo -e "\n${YELLOW}测试: Redis Stack${NC}"
    
    if redis-cli -h $REDIS_HOST -p 6383 PING 2>/dev/null | grep -q PONG; then
        echo -e "  ${GREEN}✓ Redis Stack PING 成功${NC}"
        ((PASS++))
        
        # 检查模块
        local modules=$(redis-cli -h $REDIS_HOST -p 6383 MODULE LIST 2>/dev/null)
        if [ -n "$modules" ]; then
            local module_count=$(echo "$modules" | grep -c "name")
            echo -e "  ${GREEN}✓ 已加载 $module_count 个模块${NC}"
            ((PASS++))
            
            # 测试 JSON 模块
            if echo "$modules" | grep -q "ReJSON"; then
                redis-cli -h $REDIS_HOST -p 6383 JSON.SET test:stack:json $ '{"name":"test","value":123}' > /dev/null 2>&1
                local json_result=$(redis-cli -h $REDIS_HOST -p 6383 JSON.GET test:stack:json 2>/dev/null)
                if [ -n "$json_result" ]; then
                    echo -e "  ${GREEN}✓ JSON 模块工作正常${NC}"
                    ((PASS++))
                    redis-cli -h $REDIS_HOST -p 6383 DEL test:stack:json > /dev/null 2>&1
                fi
            fi
        else
            echo -e "  ${YELLOW}⚠ 未检测到模块${NC}"
        fi
    else
        echo -e "  ${YELLOW}⚠ 跳过: Redis Stack 未运行${NC}"
        echo "  提示: 运行 'cd test && docker-compose up -d redis-stack' 启动 Redis Stack"
    fi
}

# 执行测试
echo -e "\n${BLUE}开始连接测试...${NC}"

if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "direct" ]; then
    test_connection "Direct 连接" $REDIS_HOST 6379
fi

if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "cluster" ]; then
    test_cluster
fi

if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "sentinel" ]; then
    test_sentinel
fi

if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "ssl" ]; then
    test_ssl
fi

if [ "$TEST_TYPE" = "all" ]; then
    test_readonly
    test_redis_stack
fi

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