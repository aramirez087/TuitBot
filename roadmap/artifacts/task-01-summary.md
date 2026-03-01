# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.973 | 0.813 | 1.568 | 0.661 | 1.568 |
| health_check | 0.319 | 0.212 | 0.861 | 0.110 | 0.861 |
| get_stats | 1.377 | 1.335 | 2.217 | 0.973 | 2.217 |
| list_pending | 0.406 | 0.188 | 1.077 | 0.115 | 1.077 |
| list_unreplied_tweets_with_limit | 0.235 | 0.117 | 0.762 | 0.053 | 0.762 |

**Aggregate** — P50: 0.661 ms, P95: 1.568 ms, Min: 0.053 ms, Max: 2.217 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
