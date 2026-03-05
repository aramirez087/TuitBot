# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.931 | 0.759 | 1.702 | 0.673 | 1.702 |
| health_check | 0.297 | 0.220 | 0.641 | 0.166 | 0.641 |
| get_stats | 1.484 | 1.214 | 2.823 | 0.913 | 2.823 |
| list_pending | 0.314 | 0.098 | 1.178 | 0.092 | 1.178 |
| list_unreplied_tweets_with_limit | 0.292 | 0.133 | 0.921 | 0.115 | 0.921 |

**Aggregate** — P50: 0.641 ms, P95: 1.702 ms, Min: 0.092 ms, Max: 2.823 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
