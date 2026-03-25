#!/bin/sh

REDIS_HOST="${REDIS_HOST:-localhost}"
REDIS_PORT="${REDIS_PORT:-6379}"

# Wait for Redis to be ready
echo "Waiting for Redis at ${REDIS_HOST}:${REDIS_PORT} to be ready..."
until redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} ping 2>/dev/null | grep -q PONG; do
  sleep 1
done

echo "Redis is ready. Initializing test data..."

# ============================================
# PHP Serialization Test Data
# Format: a:2:{s:3:"foo";s:3:"bar";s:3:"baz";i:42;}
# Meaning: associative array with 2 key-value pairs
# ============================================
echo "Setting PHP serialization test data..."
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SET "test:php:array" 'a:2:{s:3:"foo";s:3:"bar";s:3:"baz";i:42;}'

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SET "test:php:object" 'O:8:"stdClass":2:{s:4:"name";s:4:"test";s:5:"value";i:123;}'

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SET "test:php:nested" 'a:2:{s:4:"user";a:3:{s:2:"id";i:1;s:4:"name";s:4:"John";s:5:"email";s:13:"john@test.com";}s:6:"status";s:6:"active";}'

# ============================================
# MessagePack Test Data
# Format: {"name":"test","count":100}
# ============================================
echo "Setting MessagePack test data..."
printf '\x82\xa4name\xa4test\xa5count\x64' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:msgpack:map"

printf '\x93\xa3one\xa3two\xa3three' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:msgpack:array"

printf '\x82\xa4code\x64\xa7message\xa7success' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:msgpack:response"

# ============================================
# Python Pickle Test Data
# Format: {"items":[1,2,3],"active":True}
# ============================================
echo "Setting Pickle test data..."
printf '\x80\x04\x95\x1c\x00\x00\x00\x00\x00\x00\x00}\x94(\x8c\x05items\x94]\x94(K\x01K\x02K\x03e\x8c\x06active\x94\x88u.' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:pickle:dict"

printf '\x80\x04\x95\x0b\x00\x00\x00\x00\x00\x00\x00}\x94(\x8c\x01a\x94K\x01\x8c\x01b\x94K\x02u.' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:pickle:simple"

printf '\x80\x04\x95\x12\x00\x00\x00\x00\x00\x00\x00]\x94(K\x01K\x02K\x03K\x04K\x05e.' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:pickle:list"

# ============================================
# Kryo Test Data
# Format: Simple types
# ============================================
echo "Setting Kryo test data..."
printf '\x0a\x05Hello' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:kryo:string"

printf '\x04\x7f' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:kryo:int"

printf '\x08' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:kryo:bool-true"

printf '\x09' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:kryo:bool-false"

printf '\x0b\x03\x04\x01\x04\x02\x04\x03' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:kryo:list"

# ============================================
# FST Test Data
# Format: FST serialized data
# ============================================
echo "Setting FST test data..."
printf '\xf0\x00\x00\x00\x00\x00\x00\x04test' | redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} -x SET "test:fst:string"

# ============================================
# Bitmap Test Data
# ============================================
echo "Setting Bitmap test data..."
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SETBIT "test:bitmap:flags" 0 1
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SETBIT "test:bitmap:flags" 3 1
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SETBIT "test:bitmap:flags" 7 1
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SETBIT "test:bitmap:flags" 10 1
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SETBIT "test:bitmap:flags" 15 1

# ============================================
# Regular Redis Data Types for comparison
# ============================================
echo "Setting regular Redis data types..."

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SET "test:string:simple" "Hello World"
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SET "test:string:json" '{"name":"test","value":123}'

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} HSET "test:hash:user" "id" "1" "name" "John" "email" "john@example.com"
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} HSET "test:hash:config" "theme" "dark" "language" "en" "timeout" "30"

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} LPUSH "test:list:queue" "task3" "task2" "task1"
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} RPUSH "test:list:logs" "log1" "log2" "log3"

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SADD "test:set:tags" "redis" "database" "cache" "nosql"
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} SADD "test:set:users" "user1" "user2" "user3"

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} ZADD "test:zset:scores" 100 "player1" 200 "player2" 150 "player3"
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} ZADD "test:zset:priority" 1 "low" 5 "medium" 10 "high"

redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} XADD "test:stream:events" "*" "type" "click" "x" "100" "y" "200"
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} XADD "test:stream:events" "*" "type" "scroll" "position" "500"
redis-cli -h ${REDIS_HOST} -p ${REDIS_PORT} XADD "test:stream:events" "*" "type" "keypress" "key" "Enter"

echo "Test data initialization complete!"
echo ""
echo "Summary of test keys:"
echo "  PHP:           test:php:array, test:php:object, test:php:nested"
echo "  MessagePack:   test:msgpack:map, test:msgpack:array, test:msgpack:response"
echo "  Pickle:        test:pickle:dict, test:pickle:simple, test:pickle:list"
echo "  Kryo:          test:kryo:string, test:kryo:int, test:kryo:bool-true, test:kryo:bool-false, test:kryo:list"
echo "  FST:           test:fst:string"
echo "  Bitmap:        test:bitmap:flags"
echo "  String:        test:string:simple, test:string:json"
echo "  Hash:          test:hash:user, test:hash:config"
echo "  List:          test:list:queue, test:list:logs"
echo "  Set:           test:set:tags, test:set:users"
echo "  ZSet:          test:zset:scores, test:zset:priority"
echo "  Stream:        test:stream:events"