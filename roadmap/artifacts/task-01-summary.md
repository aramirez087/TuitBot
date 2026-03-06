# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.034 | 0.805 | 2.020 | 0.712 | 2.020 |
| health_check | 0.269 | 0.167 | 0.674 | 0.127 | 0.674 |
| get_stats | 1.499 | 1.273 | 2.529 | 1.082 | 2.529 |
| list_pending | 0.360 | 0.124 | 1.334 | 0.095 | 1.334 |
| list_unreplied_tweets_with_limit | 0.265 | 0.145 | 0.765 | 0.110 | 0.765 |

**Aggregate** — P50: 0.674 ms, P95: 2.020 ms, Min: 0.095 ms, Max: 2.529 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
