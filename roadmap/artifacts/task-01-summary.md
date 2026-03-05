# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.804 | 0.740 | 1.124 | 0.666 | 1.124 |
| health_check | 0.368 | 0.384 | 0.511 | 0.233 | 0.511 |
| get_stats | 1.878 | 1.457 | 2.659 | 1.423 | 2.659 |
| list_pending | 0.364 | 0.159 | 1.236 | 0.089 | 1.236 |
| list_unreplied_tweets_with_limit | 0.323 | 0.245 | 0.769 | 0.157 | 0.769 |

**Aggregate** — P50: 0.511 ms, P95: 2.407 ms, Min: 0.089 ms, Max: 2.659 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
