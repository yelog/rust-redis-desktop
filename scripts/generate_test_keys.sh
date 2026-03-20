#!/bin/bash

# Redis 性能测试脚本 - 生成大量键用于测试

REDIS_CLI=${REDIS_CLI:-redis-cli}
HOST=${1:-127.0.0.1}
PORT=${2:-6379}

echo "🚀 Generating test keys on $HOST:$PORT"

# 生成 10万 个键
echo "Generating 100,000 keys..."

for i in {1..100000}; do
    key="user:$((i / 1000)):item:$i"
    value="value_$i"
    $REDIS_CLI -h $HOST -p $PORT SET "$key" "$value" > /dev/null
    
    if [ $((i % 10000)) -eq 0 ]; then
        echo "  ✓ Generated $i keys"
    fi
done

echo "✅ Done! Generated 100,000 keys"
echo ""
echo "Sample keys:"
$REDIS_CLI -h $HOST -p $PORT KEYS "user:0:*" | head -5
echo ""
echo "Total keys in DB:"
$REDIS_CLI -h $HOST -p $PORT DBSIZE