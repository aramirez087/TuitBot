# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.981 | 0.808 | 1.601 | 0.733 | 1.601 |
| health_check | 0.267 | 0.189 | 0.568 | 0.148 | 0.568 |
| get_stats | 1.333 | 1.091 | 2.518 | 0.920 | 2.518 |
| list_pending | 0.307 | 0.101 | 1.151 | 0.066 | 1.151 |
| list_unreplied_tweets_with_limit | 0.283 | 0.112 | 0.982 | 0.102 | 0.982 |

**Aggregate** — P50: 0.568 ms, P95: 1.601 ms, Min: 0.066 ms, Max: 2.518 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
