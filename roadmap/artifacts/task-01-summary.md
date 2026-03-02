# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.894 | 0.745 | 1.280 | 0.632 | 1.280 |
| health_check | 0.272 | 0.188 | 0.722 | 0.115 | 0.722 |
| get_stats | 1.360 | 1.069 | 2.676 | 0.911 | 2.676 |
| list_pending | 0.390 | 0.133 | 1.454 | 0.102 | 1.454 |
| list_unreplied_tweets_with_limit | 0.279 | 0.121 | 0.946 | 0.083 | 0.946 |

**Aggregate** — P50: 0.632 ms, P95: 1.454 ms, Min: 0.083 ms, Max: 2.676 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
