# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.955 | 0.818 | 1.308 | 0.783 | 1.308 |
| health_check | 0.326 | 0.267 | 0.570 | 0.180 | 0.570 |
| get_stats | 1.515 | 1.282 | 2.105 | 1.159 | 2.105 |
| list_pending | 0.270 | 0.111 | 0.885 | 0.100 | 0.885 |
| list_unreplied_tweets_with_limit | 0.236 | 0.126 | 0.713 | 0.088 | 0.713 |

**Aggregate** — P50: 0.570 ms, P95: 1.802 ms, Min: 0.088 ms, Max: 2.105 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
