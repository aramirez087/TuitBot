# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.953 | 0.833 | 1.582 | 0.593 | 1.582 |
| health_check | 0.263 | 0.205 | 0.580 | 0.134 | 0.580 |
| get_stats | 1.528 | 1.255 | 2.818 | 0.927 | 2.818 |
| list_pending | 0.356 | 0.121 | 1.336 | 0.079 | 1.336 |
| list_unreplied_tweets_with_limit | 0.335 | 0.306 | 0.719 | 0.137 | 0.719 |

**Aggregate** — P50: 0.580 ms, P95: 1.582 ms, Min: 0.079 ms, Max: 2.818 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
