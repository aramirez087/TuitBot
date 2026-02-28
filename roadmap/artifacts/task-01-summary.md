# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.838 | 0.708 | 1.355 | 0.652 | 1.355 |
| health_check | 0.280 | 0.217 | 0.655 | 0.129 | 0.655 |
| get_stats | 1.386 | 1.233 | 2.302 | 0.914 | 2.302 |
| list_pending | 0.391 | 0.157 | 1.376 | 0.095 | 1.376 |
| list_unreplied_tweets_with_limit | 0.278 | 0.114 | 0.923 | 0.095 | 0.923 |

**Aggregate** — P50: 0.652 ms, P95: 1.376 ms, Min: 0.095 ms, Max: 2.302 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
