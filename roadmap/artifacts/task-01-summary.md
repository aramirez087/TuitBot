# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.917 | 0.867 | 1.407 | 0.622 | 1.407 |
| health_check | 0.298 | 0.166 | 0.841 | 0.107 | 0.841 |
| get_stats | 1.441 | 1.151 | 2.635 | 1.059 | 2.635 |
| list_pending | 0.375 | 0.211 | 1.102 | 0.152 | 1.102 |
| list_unreplied_tweets_with_limit | 0.183 | 0.095 | 0.548 | 0.056 | 0.548 |

**Aggregate** — P50: 0.548 ms, P95: 1.407 ms, Min: 0.056 ms, Max: 2.635 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
