# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.021 | 0.899 | 1.640 | 0.730 | 1.640 |
| health_check | 0.279 | 0.234 | 0.489 | 0.120 | 0.489 |
| get_stats | 1.810 | 1.348 | 3.564 | 1.065 | 3.564 |
| list_pending | 0.464 | 0.102 | 1.913 | 0.080 | 1.913 |
| list_unreplied_tweets_with_limit | 0.204 | 0.133 | 0.481 | 0.118 | 0.481 |

**Aggregate** — P50: 0.481 ms, P95: 1.913 ms, Min: 0.080 ms, Max: 3.564 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
