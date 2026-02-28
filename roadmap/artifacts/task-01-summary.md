# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.391 | 1.379 | 1.716 | 1.181 | 1.716 |
| health_check | 0.282 | 0.201 | 0.565 | 0.162 | 0.565 |
| get_stats | 2.563 | 2.135 | 4.900 | 1.670 | 4.900 |
| list_pending | 0.668 | 0.312 | 2.180 | 0.200 | 2.180 |
| list_unreplied_tweets_with_limit | 0.372 | 0.213 | 1.047 | 0.150 | 1.047 |

**Aggregate** — P50: 0.565 ms, P95: 2.287 ms, Min: 0.150 ms, Max: 4.900 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
