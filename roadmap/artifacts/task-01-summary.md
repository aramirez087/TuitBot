# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.884 | 0.776 | 1.383 | 0.589 | 1.383 |
| health_check | 0.282 | 0.263 | 0.535 | 0.129 | 0.535 |
| get_stats | 1.452 | 1.191 | 2.487 | 0.932 | 2.487 |
| list_pending | 0.371 | 0.142 | 1.359 | 0.094 | 1.359 |
| list_unreplied_tweets_with_limit | 0.168 | 0.109 | 0.405 | 0.068 | 0.405 |

**Aggregate** — P50: 0.405 ms, P95: 1.459 ms, Min: 0.068 ms, Max: 2.487 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
