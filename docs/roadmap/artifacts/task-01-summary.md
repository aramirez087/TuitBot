# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.808 | 0.725 | 1.313 | 0.487 | 1.313 |
| health_check | 0.183 | 0.145 | 0.410 | 0.098 | 0.410 |
| get_stats | 1.275 | 1.014 | 2.653 | 0.809 | 2.653 |
| list_pending | 0.237 | 0.129 | 0.707 | 0.091 | 0.707 |
| list_unreplied_tweets_with_limit | 0.230 | 0.117 | 0.709 | 0.086 | 0.709 |

**Aggregate** — P50: 0.410 ms, P95: 1.313 ms, Min: 0.086 ms, Max: 2.653 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
