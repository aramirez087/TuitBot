# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.052 | 1.146 | 1.232 | 0.754 | 1.232 |
| health_check | 0.279 | 0.221 | 0.583 | 0.138 | 0.583 |
| get_stats | 1.780 | 1.285 | 3.295 | 1.083 | 3.295 |
| list_pending | 0.389 | 0.168 | 1.264 | 0.150 | 1.264 |
| list_unreplied_tweets_with_limit | 0.323 | 0.133 | 1.030 | 0.118 | 1.030 |

**Aggregate** — P50: 0.583 ms, P95: 1.978 ms, Min: 0.118 ms, Max: 3.295 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
