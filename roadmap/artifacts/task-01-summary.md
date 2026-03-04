# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.768 | 0.731 | 1.083 | 0.573 | 1.083 |
| health_check | 0.301 | 0.219 | 0.697 | 0.173 | 0.697 |
| get_stats | 1.757 | 1.530 | 2.905 | 1.102 | 2.905 |
| list_pending | 0.284 | 0.147 | 0.913 | 0.073 | 0.913 |
| list_unreplied_tweets_with_limit | 0.297 | 0.119 | 1.002 | 0.110 | 1.002 |

**Aggregate** — P50: 0.573 ms, P95: 1.972 ms, Min: 0.073 ms, Max: 2.905 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
