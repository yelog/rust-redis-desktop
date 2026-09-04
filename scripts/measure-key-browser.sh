#!/usr/bin/env bash
set -euo pipefail

redis_url="redis://127.0.0.1:6379/15"
count=100000
report="docs/performance/key-browser-baseline.md"

while (($# > 0)); do
    case "$1" in
        --redis-url) redis_url="$2"; shift 2 ;;
        --count) count="$2"; shift 2 ;;
        --report) report="$2"; shift 2 ;;
        -h|--help)
            printf 'Usage: %s [--redis-url URL] [--count N] [--report PATH]\n' "$0"
            exit 0
            ;;
        *) printf 'Unknown argument: %s\n' "$1" >&2; exit 2 ;;
    esac
done

if ! redis-cli -u "$redis_url" ping >/dev/null; then
    printf 'Redis is not reachable at %s\n' "$redis_url" >&2
    exit 1
fi

mkdir -p "$(dirname "$report")"
commit=$(git rev-parse HEAD)
timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
redis_version=$(redis-cli -u "$redis_url" info server | awk -F: '/^redis_version:/{gsub(/\r/, "", $2); print $2}')
platform=$(uname -a)

cat > "$report" <<EOF
# Key Browser Performance Baseline

This file records measurements, not product guarantees. Complete the manual UI fields after running the release binary.

- Timestamp (UTC): $timestamp
- Commit: $commit
- Redis: $redis_version
- Platform: $platform
- Redis URL: redacted; database selected by the command arguments
- Dataset count: $count

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


scripts/generate-benchmark-data.sh --count $count

Do not delete keys without confirming the Redis URL and database. The generator prints a cleanup command and never performs cleanup automatically.

## Interpretation

The application currently uses an in-memory result/tree model. Until the disk-backed index milestone is complete, this baseline must not be interpreted as constant-memory or 10-million-key support.
EOF

printf 'Wrote baseline template to %s\n' "$report"
