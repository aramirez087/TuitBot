# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.944 | 0.718 | 1.529 | 0.696 | 1.529 |
| health_check | 0.274 | 0.224 | 0.526 | 0.140 | 0.526 |
| get_stats | 1.381 | 1.106 | 2.603 | 0.997 | 2.603 |
| list_pending | 0.443 | 0.099 | 1.851 | 0.068 | 1.851 |
| list_unreplied_tweets_with_limit | 0.232 | 0.131 | 0.708 | 0.080 | 0.708 |

**Aggregate** — P50: 0.526 ms, P95: 1.851 ms, Min: 0.068 ms, Max: 2.603 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
