# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.937 | 0.894 | 1.432 | 0.691 | 1.432 |
| health_check | 0.277 | 0.208 | 0.559 | 0.176 | 0.559 |
| get_stats | 1.524 | 1.280 | 2.881 | 1.022 | 2.881 |
| list_pending | 0.337 | 0.142 | 1.154 | 0.109 | 1.154 |
| list_unreplied_tweets_with_limit | 0.235 | 0.069 | 0.897 | 0.054 | 0.897 |

**Aggregate** — P50: 0.559 ms, P95: 1.432 ms, Min: 0.054 ms, Max: 2.881 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
