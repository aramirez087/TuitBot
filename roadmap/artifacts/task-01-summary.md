# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.009 | 0.924 | 1.796 | 0.665 | 1.796 |
| health_check | 0.311 | 0.234 | 0.755 | 0.125 | 0.755 |
| get_stats | 1.539 | 1.182 | 3.009 | 1.082 | 3.009 |
| list_pending | 0.277 | 0.155 | 0.818 | 0.096 | 0.818 |
| list_unreplied_tweets_with_limit | 0.197 | 0.118 | 0.523 | 0.105 | 0.523 |

**Aggregate** — P50: 0.523 ms, P95: 1.796 ms, Min: 0.096 ms, Max: 3.009 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
