# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.914 | 0.914 | 1.248 | 0.688 | 1.248 |
| health_check | 0.297 | 0.257 | 0.531 | 0.185 | 0.531 |
| get_stats | 1.466 | 1.123 | 2.713 | 1.050 | 2.713 |
| list_pending | 0.362 | 0.124 | 1.409 | 0.052 | 1.409 |
| list_unreplied_tweets_with_limit | 0.266 | 0.120 | 0.925 | 0.051 | 0.925 |

**Aggregate** — P50: 0.531 ms, P95: 1.409 ms, Min: 0.051 ms, Max: 2.713 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
