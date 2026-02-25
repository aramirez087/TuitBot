# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.858 | 0.774 | 1.232 | 0.471 | 1.232 |
| health_check | 0.190 | 0.149 | 0.368 | 0.138 | 0.368 |
| get_stats | 1.313 | 0.911 | 2.945 | 0.852 | 2.945 |
| list_pending | 0.342 | 0.151 | 1.167 | 0.103 | 1.167 |
| list_unreplied_tweets_with_limit | 0.162 | 0.116 | 0.349 | 0.068 | 0.349 |

**Aggregate** — P50: 0.349 ms, P95: 1.232 ms, Min: 0.068 ms, Max: 2.945 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
