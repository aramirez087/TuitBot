# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.072 | 0.903 | 1.731 | 0.871 | 1.731 |
| health_check | 0.243 | 0.247 | 0.364 | 0.126 | 0.364 |
| get_stats | 1.680 | 1.801 | 2.401 | 1.013 | 2.401 |
| list_pending | 0.366 | 0.124 | 1.349 | 0.099 | 1.349 |
| list_unreplied_tweets_with_limit | 0.201 | 0.121 | 0.586 | 0.081 | 0.586 |

**Aggregate** — P50: 0.364 ms, P95: 1.938 ms, Min: 0.081 ms, Max: 2.401 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
