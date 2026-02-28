# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.970 | 0.894 | 1.541 | 0.709 | 1.541 |
| health_check | 0.294 | 0.211 | 0.638 | 0.195 | 0.638 |
| get_stats | 1.839 | 1.708 | 3.280 | 0.989 | 3.280 |
| list_pending | 0.396 | 0.116 | 1.493 | 0.080 | 1.493 |
| list_unreplied_tweets_with_limit | 0.240 | 0.134 | 0.510 | 0.078 | 0.510 |

**Aggregate** — P50: 0.510 ms, P95: 1.980 ms, Min: 0.078 ms, Max: 3.280 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
