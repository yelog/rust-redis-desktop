#!/usr/bin/env bash
set -euo pipefail

redis_url="${REDIS_URL:-redis://127.0.0.1:6379/15}"
prefix="rrd-integration-$$-${RANDOM}"
count="${LARGE_DB_TEST_COUNT:-1200}"
collection_count="${LARGE_DB_TEST_COLLECTION_COUNT:-1200}"

usage() {
    printf 'Usage: %s [--redis-url URL] [--count N] [--collection-count N]\n' "$0"
}

while (($# > 0)); do
    case "$1" in
        --redis-url)
            (($# >= 2)) || { usage >&2; exit 2; }
            redis_url="$2"
            shift 2
            ;;
        --count)
            (($# >= 2)) || { usage >&2; exit 2; }
            count="$2"
            shift 2
            ;;
        --collection-count)
            (($# >= 2)) || { usage >&2; exit 2; }
            collection_count="$2"
            shift 2
            ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; exit 2 ;;
    esac
done

if ! [[ "$count" =~ ^[1-9][0-9]*$ && "$collection_count" =~ ^[1-9][0-9]*$ ]]; then
    printf 'count and collection-count must be positive integers\n' >&2
    exit 2
fi

if ! command -v redis-cli >/dev/null 2>&1; then
    printf 'redis-cli is required for the large database workflow test\n' >&2
    exit 1
fi

redis() { redis-cli -u "$redis_url" "$@"; }
cleanup() {
    while IFS= read -r key; do
        [[ -z "$key" ]] || redis DEL "$key" >/dev/null
    done < <(redis --scan --pattern "$prefix:*") || true
}
trap cleanup EXIT

[[ "$(redis PING)" == PONG ]]
before_db_size=$(redis DBSIZE)
for ((i = 0; i < count; i++)); do
    printf 'SET %s:key:%06d %d\n' "$prefix" "$i" "$i"
done | redis --pipe >/dev/null
redis LPUSH "$prefix:list" value >/dev/null
redis XADD "$prefix:stream" '*' field value >/dev/null
for ((i = 0; i < collection_count; i++)); do
    printf 'HSET %s:hash field:%06d value:%d\n' "$prefix" "$i" "$i"
    printf 'SADD %s:set member:%06d\n' "$prefix" "$i"
    printf 'ZADD %s:zset %d member:%06d\n' "$prefix" "$i" "$i"
done | redis --pipe >/dev/null

scan_collection_count() {
    local command="$1"
    local key="$2"
    local expected="$3"
    local item_width="$4"
    local cursor=0
    local item_count=0
    local rounds=0

    while :; do
        local scan_reply next_cursor batch_items batch_lines
        scan_reply=$(redis --raw "$command" "$key" "$cursor" COUNT 100)
        if [[ "$scan_reply" == *$'\n'* ]]; then
            next_cursor=${scan_reply%%$'\n'*}
            batch_items=${scan_reply#*$'\n'}
        else
            next_cursor="$scan_reply"
            batch_items=""
        fi
        if [[ -n "$batch_items" ]]; then
            batch_lines=$(printf '%s\n' "$batch_items" | sed '/^$/d' | wc -l | tr -d ' ')
            item_count=$((item_count + batch_lines / item_width))
        fi
        rounds=$((rounds + 1))
        cursor="$next_cursor"
        if [[ "$cursor" == "0" ]]; then
            break
        fi
    done

    [[ "$rounds" -gt 1 ]]
    [[ "$item_count" == "$expected" ]]
}

scan_cursor=0
scan_count=0
scan_rounds=0
checkpoint_cursor=0
checkpoint_count=0
while :; do
    scan_reply=$(redis --raw SCAN "$scan_cursor" MATCH "$prefix:key:*" COUNT 100)
    if [[ "$scan_reply" == *$'\n'* ]]; then
        next_cursor=${scan_reply%%$'\n'*}
        batch_keys=${scan_reply#*$'\n'}
    else
        next_cursor="$scan_reply"
        batch_keys=""
    fi
    if [[ -n "$batch_keys" ]]; then
        batch_count=$(printf '%s\n' "$batch_keys" | sed '/^$/d' | wc -l | tr -d ' ')
        scan_count=$((scan_count + batch_count))
    fi
    scan_rounds=$((scan_rounds + 1))
    scan_cursor="$next_cursor"
    if [[ "$scan_rounds" == "1" && "$scan_cursor" != "0" ]]; then
        checkpoint_cursor="$scan_cursor"
        checkpoint_count="$scan_count"
        break
    fi
    if [[ "$scan_cursor" == "0" ]]; then
        break
    fi
done

[[ "$checkpoint_cursor" != "0" ]]
while :; do
    scan_reply=$(redis --raw SCAN "$scan_cursor" MATCH "$prefix:key:*" COUNT 100)
    if [[ "$scan_reply" == *$'\n'* ]]; then
        next_cursor=${scan_reply%%$'\n'*}
        batch_keys=${scan_reply#*$'\n'}
    else
        next_cursor="$scan_reply"
        batch_keys=""
    fi
    if [[ -n "$batch_keys" ]]; then
        batch_count=$(printf '%s\n' "$batch_keys" | sed '/^$/d' | wc -l | tr -d ' ')
        scan_count=$((scan_count + batch_count))
    fi
    scan_rounds=$((scan_rounds + 1))
    scan_cursor="$next_cursor"
    if [[ "$scan_cursor" == "0" ]]; then
        break
    fi
done
[[ "$scan_count" == "$count" ]]
[[ "$scan_count" -gt "$checkpoint_count" ]]
[[ "$scan_rounds" -gt 2 ]]
[[ "$(redis DBSIZE)" -ge "$((before_db_size + count + 5))" ]]
[[ "$(redis TYPE "$prefix:hash")" == hash ]]
[[ "$(redis TYPE "$prefix:list")" == list ]]
[[ "$(redis TYPE "$prefix:set")" == set ]]
[[ "$(redis TYPE "$prefix:zset")" == zset ]]
[[ "$(redis TYPE "$prefix:stream")" == stream ]]
[[ "$(redis HLEN "$prefix:hash")" == "$collection_count" ]]
[[ "$(redis LLEN "$prefix:list")" == 1 ]]
[[ "$(redis SCARD "$prefix:set")" == "$collection_count" ]]
[[ "$(redis ZCARD "$prefix:zset")" == "$collection_count" ]]
[[ "$(redis XLEN "$prefix:stream")" == 1 ]]
scan_collection_count HSCAN "$prefix:hash" "$collection_count" 2
scan_collection_count SSCAN "$prefix:set" "$collection_count" 1
scan_collection_count ZSCAN "$prefix:zset" "$collection_count" 2

printf 'Large database workflow passed: %s keys, %s collection members, %s SCAN rounds, Redis %s\n' \
    "$count" "$collection_count" "$scan_rounds" "$(redis INFO server | sed -n 's/^redis_version://p' | tr -d '\r')"
