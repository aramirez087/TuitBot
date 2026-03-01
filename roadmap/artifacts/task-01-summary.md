# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.093 | 1.063 | 1.529 | 0.800 | 1.529 |
| health_check | 0.231 | 0.186 | 0.360 | 0.143 | 0.360 |
| get_stats | 1.707 | 1.234 | 3.701 | 1.056 | 3.701 |
| list_pending | 0.355 | 0.117 | 1.355 | 0.076 | 1.355 |
| list_unreplied_tweets_with_limit | 0.198 | 0.090 | 0.601 | 0.080 | 0.601 |

**Aggregate** — P50: 0.360 ms, P95: 1.529 ms, Min: 0.076 ms, Max: 3.701 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
