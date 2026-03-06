# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.998 | 0.706 | 2.136 | 0.655 | 2.136 |
| health_check | 0.276 | 0.201 | 0.675 | 0.138 | 0.675 |
| get_stats | 1.759 | 1.387 | 3.475 | 1.089 | 3.475 |
| list_pending | 0.364 | 0.124 | 1.348 | 0.085 | 1.348 |
| list_unreplied_tweets_with_limit | 0.202 | 0.133 | 0.504 | 0.104 | 0.504 |

**Aggregate** — P50: 0.504 ms, P95: 2.136 ms, Min: 0.085 ms, Max: 3.475 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
