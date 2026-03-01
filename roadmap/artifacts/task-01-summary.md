# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.941 | 0.750 | 1.491 | 0.723 | 1.491 |
| health_check | 0.241 | 0.138 | 0.576 | 0.095 | 0.576 |
| get_stats | 1.401 | 1.031 | 2.969 | 0.952 | 2.969 |
| list_pending | 0.461 | 0.148 | 1.791 | 0.064 | 1.791 |
| list_unreplied_tweets_with_limit | 0.273 | 0.103 | 0.947 | 0.063 | 0.947 |

**Aggregate** — P50: 0.576 ms, P95: 1.791 ms, Min: 0.063 ms, Max: 2.969 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
