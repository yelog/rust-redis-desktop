#!/bin/bash
# verify-results.sh - 验证 Redis 数据和解析结果
# 用法: REDIS_HOST=localhost REDIS_PORT=6379 ./verify-results.sh

REDIS_HOST="${REDIS_HOST:-localhost}"
REDIS_PORT="${REDIS_PORT:-6379}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS=0
FAIL=0

echo -e "${BLUE}=========================================="
echo "  Redis 数据验证"
echo "==========================================${NC}"

check_key_type() {
    local key=$1
    local expected=$2
    local actual=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT TYPE "$key" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        echo -e "${GREEN}✓${NC} $key: type=$expected"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $key: expected=$expected, got=$actual"
        ((FAIL++))
    fi
}

check_key_value() {
    local key=$1
    local expected=$2
    local actual=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT GET "$key" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        echo -e "${GREEN}✓${NC} $key: value match"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $key: value mismatch"
        ((FAIL++))
    fi
}

check_key_exists() {
    local key=$1
    local exists=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT EXISTS "$key" 2>/dev/null)
    if [ "$exists" = "1" ]; then
        echo -e "${GREEN}✓${NC} $key exists"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $key not found"
        ((FAIL++))
    fi
}

check_hash_fields() {
    local key=$1
    shift
    local fields=("$@")
    local missing=0
    for field in "${fields[@]}"; do
        local exists=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT HEXISTS "$key" "$field" 2>/dev/null)
        if [ "$exists" = "1" ]; then
            ((PASS++))
        else
            echo -e "${RED}✗${NC} $key missing field: $field"
            ((FAIL++))
            ((missing++))
        fi
    done
    if [ $missing -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $key: all fields present"
    fi
}

check_list_length() {
    local key=$1
    local expected=$2
    local actual=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT LLEN "$key" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        echo -e "${GREEN}✓${NC} $key: length=$expected"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $key: expected length=$expected, got=$actual"
        ((FAIL++))
    fi
}

check_set_cardinality() {
    local key=$1
    local expected=$2
    local actual=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT SCARD "$key" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        echo -e "${GREEN}✓${NC} $key: cardinality=$expected"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $key: expected cardinality=$expected, got=$actual"
        ((FAIL++))
    fi
}

check_zset_cardinality() {
    local key=$1
    local expected=$2
    local actual=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT ZCARD "$key" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        echo -e "${GREEN}✓${NC} $key: cardinality=$expected"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $key: expected cardinality=$expected, got=$actual"
        ((FAIL++))
    fi
}

check_stream_length() {
    local key=$1
    local expected=$2
    local actual=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT XLEN "$key" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        echo -e "${GREEN}✓${NC} $key: length=$expected"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $key: expected length=$expected, got=$actual"
        ((FAIL++))
    fi
}

# ===== Redis 基本类型测试 =====
echo -e "\n${YELLOW}=== Redis 基本类型测试 ===${NC}"

echo -e "\n${BLUE}String 类型:${NC}"
check_key_type "test:string:simple" "string"
check_key_value "test:string:simple" "Hello World"
check_key_type "test:string:json" "string"

echo -e "\n${BLUE}Hash 类型:${NC}"
check_key_type "test:hash:user" "hash"
check_hash_fields "test:hash:user" "id" "name" "email"
check_key_type "test:hash:config" "hash"
check_hash_fields "test:hash:config" "theme" "language" "timeout"

echo -e "\n${BLUE}List 类型:${NC}"
check_key_type "test:list:queue" "list"
check_list_length "test:list:queue" "3"
check_key_type "test:list:logs" "list"
check_list_length "test:list:logs" "3"

echo -e "\n${BLUE}Set 类型:${NC}"
check_key_type "test:set:tags" "set"
check_set_cardinality "test:set:tags" "4"
check_key_type "test:set:users" "set"
check_set_cardinality "test:set:users" "3"

echo -e "\n${BLUE}ZSet 类型:${NC}"
check_key_type "test:zset:scores" "zset"
check_zset_cardinality "test:zset:scores" "3"
check_key_type "test:zset:priority" "zset"
check_zset_cardinality "test:zset:priority" "3"

echo -e "\n${BLUE}Stream 类型:${NC}"
check_key_type "test:stream:events" "stream"
check_stream_length "test:stream:events" "3"

# ===== 序列化数据测试 =====
echo -e "\n${YELLOW}=== 序列化数据测试 ===${NC}"

echo -e "\n${BLUE}PHP 序列化:${NC}"
check_key_exists "test:php:array"
check_key_exists "test:php:object"
check_key_exists "test:php:nested"

echo -e "\n${BLUE}MessagePack:${NC}"
check_key_exists "test:msgpack:map"
check_key_exists "test:msgpack:array"
check_key_exists "test:msgpack:response"

echo -e "\n${BLUE}Python Pickle:${NC}"
check_key_exists "test:pickle:dict"
check_key_exists "test:pickle:simple"
check_key_exists "test:pickle:list"

echo -e "\n${BLUE}Kryo:${NC}"
check_key_exists "test:kryo:string"
check_key_exists "test:kryo:int"
check_key_exists "test:kryo:bool-true"
check_key_exists "test:kryo:bool-false"
check_key_exists "test:kryo:list"

echo -e "\n${BLUE}FST:${NC}"
check_key_exists "test:fst:string"

echo -e "\n${BLUE}Protobuf:${NC}"
check_key_exists "test:protobuf:simple"
check_key_exists "test:protobuf:user"
check_key_exists "test:protobuf:numbers"

# ===== 图片数据测试 =====
echo -e "\n${YELLOW}=== 图片数据测试 ===${NC}"
check_key_exists "test:image:png"
check_key_exists "test:image:jpeg"
check_key_exists "test:image:gif"

# ===== 边界情况测试 =====
echo -e "\n${YELLOW}=== 边界情况测试 ===${NC}"
check_key_type "test:bitmap:flags" "string"

# ===== 大数据集测试（可选） =====
echo -e "\n${YELLOW}=== 大数据集测试 (可选) ===${NC}"

if [ "$(redis-cli -h $REDIS_HOST -p $REDIS_PORT EXISTS test:hash:large 2>/dev/null)" = "1" ]; then
    echo -e "${BLUE}大数据集已存在，验证中...${NC}"
    check_key_type "test:hash:large" "hash"
    check_key_type "test:list:large" "list"
    check_key_type "test:set:large" "set"
    check_key_type "test:zset:large" "zset"
else
    echo -e "${YELLOW}大数据集不存在，跳过测试${NC}"
    echo "提示: 如需创建大数据集，请运行 init-test-data.sh（需要较长时间）"
fi

# ===== 结果汇总 =====
echo -e "\n${BLUE}=========================================="
echo "  测试结果汇总"
echo "==========================================${NC}"
echo -e "${GREEN}通过: $PASS${NC}"
echo -e "${RED}失败: $FAIL${NC}"
echo ""

if [ $FAIL -gt 0 ]; then
    echo -e "${RED}存在失败的测试项，请检查上述输出${NC}"
    exit 1
else
    echo -e "${GREEN}所有测试通过!${NC}"
    exit 0
fi