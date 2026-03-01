# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.056 | 1.005 | 1.372 | 0.942 | 1.372 |
| health_check | 0.346 | 0.278 | 0.504 | 0.196 | 0.504 |
| get_stats | 1.684 | 1.440 | 2.978 | 1.061 | 2.978 |
| list_pending | 0.390 | 0.127 | 1.447 | 0.121 | 1.447 |
| list_unreplied_tweets_with_limit | 0.220 | 0.099 | 0.692 | 0.081 | 0.692 |

**Aggregate** — P50: 0.504 ms, P95: 1.635 ms, Min: 0.081 ms, Max: 2.978 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
