# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.257 | 1.152 | 1.613 | 1.118 | 1.613 |
| health_check | 0.409 | 0.322 | 0.905 | 0.138 | 0.905 |
| get_stats | 1.937 | 1.586 | 3.700 | 1.175 | 3.700 |
| list_pending | 0.320 | 0.159 | 0.988 | 0.126 | 0.988 |
| list_unreplied_tweets_with_limit | 0.302 | 0.135 | 0.997 | 0.063 | 0.997 |

**Aggregate** — P50: 0.905 ms, P95: 1.813 ms, Min: 0.063 ms, Max: 3.700 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
