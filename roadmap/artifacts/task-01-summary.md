# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.900 | 0.835 | 1.377 | 0.531 | 1.377 |
| health_check | 0.460 | 0.364 | 0.962 | 0.135 | 0.962 |
| get_stats | 1.983 | 1.546 | 4.266 | 1.193 | 4.266 |
| list_pending | 0.397 | 0.132 | 1.455 | 0.113 | 1.455 |
| list_unreplied_tweets_with_limit | 0.274 | 0.163 | 0.816 | 0.092 | 0.816 |

**Aggregate** — P50: 0.531 ms, P95: 1.642 ms, Min: 0.092 ms, Max: 4.266 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
