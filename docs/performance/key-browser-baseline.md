# Key Browser Performance Baseline

This file records measurements, not product guarantees. Complete the manual UI fields after running the release binary.

- Timestamp (UTC): 2026-09-04T04:39:27Z
- Commit: 53dd9346fb6378afeb2c2ab04fb2c96729389abf
- Redis: 7.4.11
- Platform: Darwin LZHANG7-F42L24M.lenovo.com 25.6.0 Darwin Kernel Version 25.6.0: Fri Jul 31 19:18:48 PDT 2026; root:xnu-12377.161.14~5/RELEASE_ARM64_T6020 arm64
- Redis URL: redacted; database selected by the command arguments
- Dataset count: 100000
- Dataset prefix: rrd-benchmark-20260904
- Observed matching keys: 100000
- Redis-side SCAN wall time (seconds): 1.53

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


scripts/generate-benchmark-data.sh --count 100000 --prefix rrd-benchmark-20260904 --yes

Do not delete keys without confirming the Redis URL and database. The generator prints a cleanup command and never performs cleanup automatically.

## Interpretation

Progressive mode still keeps matching results in memory. Complete indexed mode uses a temporary disk-backed index, but this baseline must not be interpreted as constant-memory or 10-million-key support without recording the manual UI measurements above.
