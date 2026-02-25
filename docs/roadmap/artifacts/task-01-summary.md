# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.768 | 0.726 | 1.162 | 0.473 | 1.162 |
| health_check | 0.281 | 0.216 | 0.507 | 0.209 | 0.507 |
| get_stats | 1.349 | 1.077 | 2.787 | 0.843 | 2.787 |
| list_pending | 0.255 | 0.133 | 0.782 | 0.073 | 0.782 |
| list_unreplied_tweets_with_limit | 0.239 | 0.131 | 0.695 | 0.093 | 0.695 |

**Aggregate** — P50: 0.473 ms, P95: 1.162 ms, Min: 0.073 ms, Max: 2.787 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
