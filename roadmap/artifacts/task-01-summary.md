# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.964 | 0.836 | 1.564 | 0.714 | 1.564 |
| health_check | 0.223 | 0.185 | 0.433 | 0.111 | 0.433 |
| get_stats | 1.421 | 1.208 | 2.488 | 1.073 | 2.488 |
| list_pending | 0.372 | 0.162 | 1.257 | 0.081 | 1.257 |
| list_unreplied_tweets_with_limit | 0.200 | 0.101 | 0.607 | 0.088 | 0.607 |

**Aggregate** — P50: 0.433 ms, P95: 1.564 ms, Min: 0.081 ms, Max: 2.488 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
