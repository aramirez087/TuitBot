# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.601 | 0.531 | 0.904 | 0.517 | 0.904 |
| health_check | 0.089 | 0.076 | 0.141 | 0.075 | 0.141 |
| get_stats | 0.473 | 0.425 | 0.667 | 0.414 | 0.667 |
| list_pending | 0.082 | 0.058 | 0.170 | 0.058 | 0.170 |
| list_unreplied_tweets_with_limit | 0.066 | 0.056 | 0.102 | 0.056 | 0.102 |

**Aggregate** — P50: 0.102 ms, P95: 0.667 ms, Min: 0.056 ms, Max: 0.904 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
