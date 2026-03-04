# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.133 | 1.079 | 1.719 | 0.835 | 1.719 |
| health_check | 0.278 | 0.223 | 0.571 | 0.183 | 0.571 |
| get_stats | 1.584 | 1.263 | 3.072 | 1.067 | 3.072 |
| list_pending | 0.428 | 0.125 | 1.669 | 0.094 | 1.669 |
| list_unreplied_tweets_with_limit | 0.275 | 0.103 | 0.924 | 0.096 | 0.924 |

**Aggregate** — P50: 0.571 ms, P95: 1.719 ms, Min: 0.094 ms, Max: 3.072 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
