#!/usr/bin/env bash
set -euo pipefail

redis_url="redis://127.0.0.1:6379/15"
count=100000
prefix="rrd-benchmark"
report="docs/performance/key-browser-baseline.md"

while (($# > 0)); do
    case "$1" in
        --redis-url)
            (($# >= 2)) || { printf 'Missing value for --redis-url\n' >&2; exit 2; }
            redis_url="$2"
            shift 2
            ;;
        --count)
            (($# >= 2)) || { printf 'Missing value for --count\n' >&2; exit 2; }
            count="$2"
            shift 2
            ;;
        --prefix)
            (($# >= 2)) || { printf 'Missing value for --prefix\n' >&2; exit 2; }
            prefix="$2"
            shift 2
            ;;
        --report)
            (($# >= 2)) || { printf 'Missing value for --report\n' >&2; exit 2; }
            report="$2"
            shift 2
            ;;
        -h|--help)
            printf 'Usage: %s [--redis-url URL] [--count N] [--prefix TEXT] [--report PATH]\n' "$0"
            exit 0
            ;;
        *) printf 'Unknown argument: %s\n' "$1" >&2; exit 2 ;;
    esac
done

if ! [[ "$count" =~ ^[1-9][0-9]*$ ]]; then
    printf 'count must be a positive integer\n' >&2
    exit 2
fi

if ! [[ "$prefix" =~ ^[A-Za-z0-9_.:-]+$ ]]; then
    printf 'prefix may contain only letters, numbers, underscore, dot, colon, and hyphen.\n' >&2
    exit 2
fi

if ! command -v redis-cli >/dev/null 2>&1; then
    printf 'redis-cli is required to measure Redis-side scan timing\n' >&2
    exit 1
fi

if ! redis-cli -u "$redis_url" ping >/dev/null; then
    printf 'Redis is not reachable at the configured URL\n' >&2
    exit 1
fi

mkdir -p "$(dirname "$report")"
commit=$(git rev-parse HEAD)
timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
redis_version=$(redis-cli -u "$redis_url" info server | awk -F: '/^redis_version:/{gsub(/\r/, "", $2); print $2}')
platform=$(uname -a)
scan_timing=$( { /usr/bin/time -p redis-cli -u "$redis_url" --scan --pattern "$prefix:*" >/dev/null; } 2>&1 )
scan_seconds=$(printf '%s\n' "$scan_timing" | awk '$1 == "real" { print $2 }')
observed_count=$(redis-cli -u "$redis_url" --scan --pattern "$prefix:*" | sed '/^$/d' | wc -l | tr -d ' ')
if [[ "$observed_count" != "$count" ]]; then
    printf 'Expected %s matching keys for prefix %s, found %s\n' "$count" "$prefix" "$observed_count" >&2
    exit 1
fi

cat > "$report" <<EOF
# Key Browser Performance Baseline

This file records measurements, not product guarantees. Complete the manual UI fields after running the release binary.

- Timestamp (UTC): $timestamp
- Commit: $commit
- Redis: $redis_version
- Platform: $platform
- Redis URL: redacted; database selected by the command arguments
- Dataset count: $count
- Dataset prefix: $prefix
- Observed matching keys: $observed_count
- Redis-side SCAN wall time (seconds): $scan_seconds

## Measurements

| Metric | Result | Method / notes |
|---|---:|---|
| First visible results | TODO | Stopwatch from refresh to first rows |
| Scan completion | TODO | Stopwatch from refresh to completed state |
| Peak RSS | TODO | Activity Monitor / Instruments / platform equivalent |
| Idle RSS after clear | TODO | Record after 30 seconds |
| Approximate scroll FPS | TODO | Manual observation or trace |
| Rendered rows at rest | TODO | Debug counter or DOM inspection |

## Dataset

Generate data separately with:


scripts/generate-benchmark-data.sh --count $count --prefix $prefix --yes

Do not delete keys without confirming the Redis URL and database. The generator prints a cleanup command and never performs cleanup automatically.

## Interpretation

Progressive mode still keeps matching results in memory. Complete indexed mode uses a temporary disk-backed index, but this baseline must not be interpreted as constant-memory or 10-million-key support without recording the manual UI measurements above.
EOF

printf 'Wrote baseline template to %s\n' "$report"
