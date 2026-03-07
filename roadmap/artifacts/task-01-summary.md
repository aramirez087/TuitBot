# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.193 | 1.007 | 1.920 | 0.895 | 1.920 |
| health_check | 0.309 | 0.265 | 0.558 | 0.147 | 0.558 |
| get_stats | 1.798 | 1.361 | 3.552 | 1.185 | 3.552 |
| list_pending | 0.433 | 0.128 | 1.645 | 0.078 | 1.645 |
| list_unreplied_tweets_with_limit | 0.283 | 0.114 | 0.951 | 0.077 | 0.951 |

**Aggregate** — P50: 0.558 ms, P95: 1.920 ms, Min: 0.077 ms, Max: 3.552 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
