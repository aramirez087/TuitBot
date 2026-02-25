# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.809 | 0.648 | 1.140 | 0.561 | 1.140 |
| health_check | 0.167 | 0.100 | 0.437 | 0.070 | 0.437 |
| get_stats | 1.268 | 1.170 | 1.923 | 0.979 | 1.923 |
| list_pending | 0.242 | 0.092 | 0.861 | 0.070 | 0.861 |
| list_unreplied_tweets_with_limit | 0.274 | 0.125 | 0.869 | 0.113 | 0.869 |

**Aggregate** — P50: 0.437 ms, P95: 1.275 ms, Min: 0.070 ms, Max: 1.923 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
