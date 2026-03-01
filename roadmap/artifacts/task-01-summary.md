# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.984 | 0.859 | 1.609 | 0.660 | 1.609 |
| health_check | 0.328 | 0.285 | 0.569 | 0.198 | 0.569 |
| get_stats | 1.458 | 1.143 | 2.854 | 1.011 | 2.854 |
| list_pending | 0.452 | 0.190 | 1.542 | 0.164 | 1.542 |
| list_unreplied_tweets_with_limit | 0.236 | 0.148 | 0.595 | 0.128 | 0.595 |

**Aggregate** — P50: 0.569 ms, P95: 1.609 ms, Min: 0.128 ms, Max: 2.854 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
