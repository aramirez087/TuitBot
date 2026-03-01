# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.170 | 1.019 | 1.972 | 0.814 | 1.972 |
| health_check | 0.322 | 0.264 | 0.507 | 0.202 | 0.507 |
| get_stats | 1.682 | 1.409 | 2.989 | 1.259 | 2.989 |
| list_pending | 0.357 | 0.127 | 1.252 | 0.088 | 1.252 |
| list_unreplied_tweets_with_limit | 0.226 | 0.096 | 0.765 | 0.064 | 0.765 |

**Aggregate** — P50: 0.507 ms, P95: 1.972 ms, Min: 0.064 ms, Max: 2.989 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
