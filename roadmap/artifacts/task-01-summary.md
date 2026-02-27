# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.225 | 1.188 | 1.455 | 1.104 | 1.455 |
| health_check | 0.389 | 0.303 | 0.953 | 0.163 | 0.953 |
| get_stats | 2.155 | 2.001 | 3.502 | 1.450 | 3.502 |
| list_pending | 0.474 | 0.104 | 1.912 | 0.068 | 1.912 |
| list_unreplied_tweets_with_limit | 0.284 | 0.101 | 1.030 | 0.085 | 1.030 |

**Aggregate** — P50: 0.953 ms, P95: 2.064 ms, Min: 0.068 ms, Max: 3.502 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
