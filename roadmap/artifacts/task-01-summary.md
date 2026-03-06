# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.151 | 1.015 | 1.522 | 0.926 | 1.522 |
| health_check | 0.358 | 0.305 | 0.668 | 0.131 | 0.668 |
| get_stats | 1.736 | 1.469 | 3.053 | 1.109 | 3.053 |
| list_pending | 0.436 | 0.151 | 1.624 | 0.101 | 1.624 |
| list_unreplied_tweets_with_limit | 0.302 | 0.102 | 1.069 | 0.099 | 1.069 |

**Aggregate** — P50: 0.668 ms, P95: 1.823 ms, Min: 0.099 ms, Max: 3.053 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
