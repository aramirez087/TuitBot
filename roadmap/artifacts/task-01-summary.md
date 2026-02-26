# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.741 | 0.592 | 1.286 | 0.555 | 1.286 |
| health_check | 0.373 | 0.267 | 0.842 | 0.185 | 0.842 |
| get_stats | 1.928 | 1.626 | 3.230 | 1.547 | 3.230 |
| list_pending | 0.381 | 0.127 | 1.318 | 0.108 | 1.318 |
| list_unreplied_tweets_with_limit | 0.282 | 0.125 | 0.906 | 0.105 | 0.906 |

**Aggregate** — P50: 0.555 ms, P95: 1.669 ms, Min: 0.105 ms, Max: 3.230 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
