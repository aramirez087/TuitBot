# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.903 | 0.804 | 1.431 | 0.698 | 1.431 |
| health_check | 0.293 | 0.280 | 0.465 | 0.193 | 0.465 |
| get_stats | 1.968 | 1.885 | 3.304 | 1.103 | 3.304 |
| list_pending | 0.602 | 0.211 | 2.095 | 0.163 | 2.095 |
| list_unreplied_tweets_with_limit | 0.251 | 0.128 | 0.745 | 0.086 | 0.745 |

**Aggregate** — P50: 0.465 ms, P95: 2.161 ms, Min: 0.086 ms, Max: 3.304 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
