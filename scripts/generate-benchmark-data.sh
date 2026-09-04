#!/usr/bin/env bash
set -euo pipefail

count=100000
prefix_depth=2
key_bytes=32
prefix="rrd-benchmark"
redis_url="redis://127.0.0.1:6379/15"

usage() {
    printf 'Usage: %s [--count N] [--prefix-depth N] [--key-bytes N] [--prefix TEXT] [--redis-url URL]\n' "$0"
}

while (($# > 0)); do
    case "$1" in
        --count) count="$2"; shift 2 ;;
        --prefix-depth) prefix_depth="$2"; shift 2 ;;
        --key-bytes) key_bytes="$2"; shift 2 ;;
        --prefix) prefix="$2"; shift 2 ;;
        --redis-url) redis_url="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; exit 2 ;;
    esac
done

if ! [[ "$count" =~ ^[1-9][0-9]*$ && "$prefix_depth" =~ ^[0-9]+$ && "$key_bytes" =~ ^[1-9][0-9]*$ ]]; then
    printf 'count, prefix-depth, and key-bytes must be positive integers (prefix-depth may be zero).\n' >&2
    exit 2
fi

if ! redis-cli -u "$redis_url" ping >/dev/null; then
    printf 'Redis is not reachable at %s\n' "$redis_url" >&2
    exit 1
fi

printf 'This writes %s benchmark keys with prefix %s to %s.\n' "$count" "$prefix" "$redis_url"
read -r -p 'Continue? Type benchmark: ' confirmation
[[ "$confirmation" == benchmark ]]

for ((i = 0; i < count; i++)); do
    key="$prefix"
    for ((level = 0; level < prefix_depth; level++)); do
        key="$key:$((i % 1000))"
    done
    suffix=$(printf '%0*d' "$key_bytes" "$i")
    printf 'SET %s:%s %s\n' "$key" "$suffix" "$i"
done | redis-cli -u "$redis_url" --pipe >/dev/null

printf 'Generated %s keys. Cleanup is deliberately manual:\n' "$count"
printf 'redis-cli -u %q --scan --pattern %q | xargs -r -n 100 redis-cli -u %q del\n' "$redis_url" "$prefix:*" "$redis_url"
