# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.916 | 0.909 | 1.302 | 0.592 | 1.302 |
| health_check | 0.272 | 0.225 | 0.652 | 0.081 | 0.652 |
| get_stats | 1.789 | 1.336 | 3.378 | 1.234 | 3.378 |
| list_pending | 0.298 | 0.125 | 1.007 | 0.112 | 1.007 |
| list_unreplied_tweets_with_limit | 0.232 | 0.097 | 0.729 | 0.077 | 0.729 |

**Aggregate** — P50: 0.592 ms, P95: 1.713 ms, Min: 0.077 ms, Max: 3.378 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
