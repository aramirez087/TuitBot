# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.487 | 1.144 | 2.299 | 0.779 | 2.299 |
| health_check | 0.544 | 0.589 | 0.645 | 0.377 | 0.645 |
| get_stats | 2.322 | 1.886 | 4.683 | 1.191 | 4.683 |
| list_pending | 0.502 | 0.122 | 1.992 | 0.068 | 1.992 |
| list_unreplied_tweets_with_limit | 0.499 | 0.173 | 1.802 | 0.156 | 1.802 |

**Aggregate** — P50: 0.645 ms, P95: 2.299 ms, Min: 0.068 ms, Max: 4.683 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
