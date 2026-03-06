# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.109 | 1.094 | 1.528 | 0.808 | 1.528 |
| health_check | 0.327 | 0.254 | 0.667 | 0.169 | 0.667 |
| get_stats | 1.628 | 1.549 | 2.530 | 1.063 | 2.530 |
| list_pending | 0.466 | 0.212 | 1.410 | 0.188 | 1.410 |
| list_unreplied_tweets_with_limit | 0.236 | 0.104 | 0.804 | 0.067 | 0.804 |

**Aggregate** — P50: 0.667 ms, P95: 1.636 ms, Min: 0.067 ms, Max: 2.530 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
