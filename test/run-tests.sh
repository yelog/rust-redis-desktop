#!/bin/bash
set -e

# run-tests.sh - Rust Redis Desktop 完整测试流程入口
# 用法: ./run-tests.sh [--skip-data] [--skip-unit] [--skip-integration]

REDIS_HOST="${REDIS_HOST:-localhost}"
REDIS_PORT="${REDIS_PORT:-6379}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMEOUT="${TIMEOUT:-30}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 解析参数
SKIP_DATA=false
SKIP_UNIT=false
SKIP_INTEGRATION=false

for arg in "$@"; do
    case $arg in
        --skip-data) SKIP_DATA=true ;;
        --skip-unit) SKIP_UNIT=true ;;
        --skip-integration) SKIP_INTEGRATION=true ;;
        --help)
            echo "用法: $0 [选项]"
            echo "选项:"
            echo "  --skip-data         跳过测试数据初始化"
            echo "  --skip-unit         跳过 Rust 单元测试"
            echo "  --skip-integration  跳过集成测试"
            echo "  --help              显示帮助信息"
            exit 0
            ;;
    esac
done

echo -e "${BLUE}=========================================="
echo "  Rust Redis Desktop - Value 解析测试"
echo "==========================================${NC}"

# 1. 检查 Redis 连接
echo -e "\n${YELLOW}[1/5] 检查 Redis 连接...${NC}"
if ! redis-cli -h $REDIS_HOST -p $REDIS_PORT ping 2>/dev/null | grep -q PONG; then
    echo -e "${RED}错误: 无法连接到 Redis at $REDIS_HOST:$REDIS_PORT${NC}"
    echo -e "${YELLOW}提示: 请先启动 Redis 容器:${NC}"
    echo "  cd test && docker-compose up -d redis-standalone"
    exit 1
fi
echo -e "${GREEN}✓ Redis 连接正常${NC}"

# 2. 初始化测试数据
if [ "$SKIP_DATA" = false ]; then
    echo -e "\n${YELLOW}[2/5] 初始化测试数据...${NC}"
    if [ -f "$SCRIPT_DIR/init-test-data.sh" ]; then
        REDIS_HOST=$REDIS_HOST REDIS_PORT=$REDIS_PORT bash "$SCRIPT_DIR/init-test-data.sh"
        echo -e "${GREEN}✓ 测试数据初始化完成${NC}"
    else
        echo -e "${RED}错误: 找不到 init-test-data.sh${NC}"
        exit 1
    fi
else
    echo -e "\n${YELLOW}[2/5] 跳过测试数据初始化${NC}"
fi

# 3. 验证测试数据
echo -e "\n${YELLOW}[3/5] 验证测试数据完整性...${NC}"
PASS=0
FAIL=0

verify_key_type() {
    local key=$1
    local expected=$2
    local actual=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT TYPE "$key" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        echo -e "  ${GREEN}✓${NC} $key: type=$expected"
        ((PASS++))
    else
        echo -e "  ${RED}✗${NC} $key: expected=$expected, got=$actual"
        ((FAIL++))
    fi
}

verify_key_exists() {
    local key=$1
    local exists=$(redis-cli -h $REDIS_HOST -p $REDIS_PORT EXISTS "$key" 2>/dev/null)
    if [ "$exists" = "1" ]; then
        echo -e "  ${GREEN}✓${NC} $key exists"
        ((PASS++))
    else
        echo -e "  ${RED}✗${NC} $key not found"
        ((FAIL++))
    fi
}

echo "Redis 基本类型:"
verify_key_type "test:string:simple" "string"
verify_key_type "test:hash:user" "hash"
verify_key_type "test:list:queue" "list"
verify_key_type "test:set:tags" "set"
verify_key_type "test:zset:scores" "zset"
verify_key_type "test:stream:events" "stream"

echo -e "\n序列化数据:"
verify_key_exists "test:php:array"
verify_key_exists "test:php:object"
verify_key_exists "test:msgpack:map"
verify_key_exists "test:pickle:dict"
verify_key_exists "test:kryo:string"
verify_key_exists "test:fst:string"
verify_key_exists "test:protobuf:simple"

echo -e "\n图片数据:"
verify_key_exists "test:image:png"
verify_key_exists "test:image:jpeg"
verify_key_exists "test:image:gif"

echo -e "\n${BLUE}数据验证汇总: ${GREEN}通过 $PASS${NC}, ${RED}失败 $FAIL${NC}"

if [ $FAIL -gt 0 ]; then
    echo -e "${RED}警告: 部分测试数据验证失败${NC}"
fi

# 4. 运行 Rust 单元测试
if [ "$SKIP_UNIT" = false ]; then
    echo -e "\n${YELLOW}[4/5] 运行单元测试...${NC}"
    cd "$SCRIPT_DIR/.."
    
    echo "  运行序列化格式检测测试..."
    cargo test serialization:: --no-fail-fast 2>&1 | tail -5
    
    echo "  运行连接配置测试..."
    cargo test tests::connection_tests --no-fail-fast 2>&1 | tail -5
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ 单元测试完成${NC}"
    else
        echo -e "${RED}✗ 单元测试失败${NC}"
    fi
else
    echo -e "\n${YELLOW}[4/5] 跳过单元测试${NC}"
fi

# 5. 集成测试
if [ "$SKIP_INTEGRATION" = false ]; then
    echo -e "\n${YELLOW}[5/5] 运行集成测试...${NC}"
    if [ -f "$SCRIPT_DIR/verify-results.sh" ]; then
        REDIS_HOST=$REDIS_HOST REDIS_PORT=$REDIS_PORT bash "$SCRIPT_DIR/verify-results.sh"
    else
        echo -e "${YELLOW}跳过集成测试 (verify-results.sh 不存在)${NC}"
    fi
    
    # 连接模式测试
    if [ -f "$SCRIPT_DIR/test-connections.sh" ]; then
        echo -e "\n${BLUE}运行连接模式测试...${NC}"
        REDIS_HOST=$REDIS_HOST bash "$SCRIPT_DIR/test-connections.sh" direct
    fi
else
    echo -e "\n${YELLOW}[5/5] 跳过集成测试${NC}"
fi

echo -e "\n${GREEN}=========================================="
echo "  所有测试完成!"
echo "==========================================${NC}"
echo ""
echo "测试用例文件位于: $SCRIPT_DIR/test-cases/"
echo "运行以下命令查看详细测试结果:"
echo "  cargo test --lib -- --nocapture"