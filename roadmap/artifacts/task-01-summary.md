# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.901 | 0.881 | 1.391 | 0.571 | 1.391 |
| health_check | 0.302 | 0.171 | 0.868 | 0.095 | 0.868 |
| get_stats | 1.551 | 1.143 | 3.141 | 0.973 | 3.141 |
| list_pending | 0.385 | 0.164 | 1.403 | 0.081 | 1.403 |
| list_unreplied_tweets_with_limit | 0.196 | 0.112 | 0.610 | 0.057 | 0.610 |

**Aggregate** — P50: 0.571 ms, P95: 1.403 ms, Min: 0.057 ms, Max: 3.141 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
