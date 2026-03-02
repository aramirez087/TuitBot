# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.942 | 0.895 | 1.525 | 0.661 | 1.525 |
| health_check | 0.308 | 0.240 | 0.616 | 0.169 | 0.616 |
| get_stats | 1.407 | 1.120 | 2.805 | 0.908 | 2.805 |
| list_pending | 0.431 | 0.094 | 1.772 | 0.077 | 1.772 |
| list_unreplied_tweets_with_limit | 0.276 | 0.140 | 0.857 | 0.101 | 0.857 |

**Aggregate** — P50: 0.616 ms, P95: 1.772 ms, Min: 0.077 ms, Max: 2.805 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
