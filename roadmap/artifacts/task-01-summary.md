# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.939 | 0.767 | 1.798 | 0.650 | 1.798 |
| health_check | 0.363 | 0.266 | 0.853 | 0.122 | 0.853 |
| get_stats | 1.359 | 0.993 | 2.856 | 0.968 | 2.856 |
| list_pending | 0.345 | 0.138 | 1.256 | 0.082 | 1.256 |
| list_unreplied_tweets_with_limit | 0.293 | 0.126 | 1.051 | 0.057 | 1.051 |

**Aggregate** — P50: 0.650 ms, P95: 1.798 ms, Min: 0.057 ms, Max: 2.856 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
