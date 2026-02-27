# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.921 | 0.798 | 1.503 | 0.705 | 1.503 |
| health_check | 0.281 | 0.248 | 0.476 | 0.159 | 0.476 |
| get_stats | 1.631 | 1.309 | 3.171 | 0.881 | 3.171 |
| list_pending | 0.323 | 0.120 | 1.200 | 0.078 | 1.200 |
| list_unreplied_tweets_with_limit | 0.291 | 0.134 | 0.921 | 0.129 | 0.921 |

**Aggregate** — P50: 0.476 ms, P95: 1.633 ms, Min: 0.078 ms, Max: 3.171 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
