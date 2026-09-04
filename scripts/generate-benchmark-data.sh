#!/usr/bin/env bash
set -euo pipefail

count=100000
prefix_depth=2
key_bytes=32
prefix="rrd-benchmark"
redis_url="redis://127.0.0.1:6379/15"
confirm=false

usage() {
    printf 'Usage: %s [--count N] [--prefix-depth N] [--key-bytes N] [--prefix TEXT] [--redis-url URL] [--yes]\n' "$0"
}

while (($# > 0)); do
    case "$1" in
        --count)
            (($# >= 2)) || { usage >&2; exit 2; }
            count="$2"
            shift 2
            ;;
        --prefix-depth)
            (($# >= 2)) || { usage >&2; exit 2; }
            prefix_depth="$2"
            shift 2
            ;;
        --key-bytes)
            (($# >= 2)) || { usage >&2; exit 2; }
            key_bytes="$2"
            shift 2
            ;;
        --prefix)
            (($# >= 2)) || { usage >&2; exit 2; }
            prefix="$2"
            shift 2
            ;;
        --redis-url)
            (($# >= 2)) || { usage >&2; exit 2; }
            redis_url="$2"
            shift 2
            ;;
        --yes) confirm=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; exit 2 ;;
    esac
done

if ! [[ "$count" =~ ^[1-9][0-9]*$ && "$prefix_depth" =~ ^[0-9]+$ && "$key_bytes" =~ ^[1-9][0-9]*$ ]]; then
    printf 'count, prefix-depth, and key-bytes must be positive integers (prefix-depth may be zero).\n' >&2
    exit 2
fi

if ! [[ "$prefix" =~ ^[A-Za-z0-9_.:-]+$ ]]; then
    printf 'prefix may contain only letters, numbers, underscore, dot, colon, and hyphen.\n' >&2
    exit 2
fi

if ! command -v redis-cli >/dev/null 2>&1; then
    printf 'redis-cli is required to generate benchmark data\n' >&2
    exit 1
fi

if ! redis-cli -u "$redis_url" ping >/dev/null; then
    printf 'Redis is not reachable at the configured URL\n' >&2
    exit 1
fi

printf 'This writes %s benchmark keys with prefix %s to the selected Redis database.\n' "$count" "$prefix"
if [[ "$confirm" != true ]]; then
    read -r -p 'Continue? Type benchmark: ' confirmation
    [[ "$confirmation" == benchmark ]]
fi

for ((i = 0; i < count; i++)); do
    key="$prefix"
    for ((level = 0; level < prefix_depth; level++)); do
        key="$key:$((i % 1000))"
    done
    suffix=$(printf '%0*d' "$key_bytes" "$i")
    printf 'SET %s:%s %s\n' "$key" "$suffix" "$i"
done | redis-cli -u "$redis_url" --pipe >/dev/null

printf 'Generated %s keys. Cleanup is deliberately manual:\n' "$count"
printf 'Set REDIS_URL to the same confirmed database, then run:\n'
printf 'redis-cli -u "$REDIS_URL" --scan --pattern %q | while IFS= read -r key; do [ -z "$key" ] || printf "DEL %%s\\n" "$key"; done | redis-cli -u "$REDIS_URL" --pipe\n' "$prefix:*"
