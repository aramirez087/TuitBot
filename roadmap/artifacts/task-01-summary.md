# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.402 | 1.153 | 2.580 | 0.851 | 2.580 |
| health_check | 0.430 | 0.348 | 0.995 | 0.170 | 0.995 |
| get_stats | 2.314 | 2.068 | 4.130 | 1.599 | 4.130 |
| list_pending | 0.472 | 0.134 | 1.859 | 0.085 | 1.859 |
| list_unreplied_tweets_with_limit | 0.355 | 0.084 | 1.328 | 0.070 | 1.328 |

**Aggregate** — P50: 0.851 ms, P95: 2.580 ms, Min: 0.070 ms, Max: 4.130 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
