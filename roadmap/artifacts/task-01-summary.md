# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.944 | 0.864 | 1.526 | 0.603 | 1.526 |
| health_check | 0.296 | 0.202 | 0.700 | 0.114 | 0.700 |
| get_stats | 1.425 | 1.291 | 2.798 | 0.856 | 2.798 |
| list_pending | 0.436 | 0.131 | 1.676 | 0.095 | 1.676 |
| list_unreplied_tweets_with_limit | 0.269 | 0.100 | 0.955 | 0.085 | 0.955 |

**Aggregate** — P50: 0.603 ms, P95: 1.676 ms, Min: 0.085 ms, Max: 2.798 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
