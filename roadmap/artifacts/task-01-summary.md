# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.066 | 0.885 | 1.789 | 0.684 | 1.789 |
| health_check | 0.240 | 0.192 | 0.466 | 0.114 | 0.466 |
| get_stats | 1.974 | 1.944 | 3.073 | 1.058 | 3.073 |
| list_pending | 0.412 | 0.128 | 1.559 | 0.101 | 1.559 |
| list_unreplied_tweets_with_limit | 0.251 | 0.161 | 0.434 | 0.129 | 0.434 |

**Aggregate** — P50: 0.434 ms, P95: 2.197 ms, Min: 0.101 ms, Max: 3.073 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
