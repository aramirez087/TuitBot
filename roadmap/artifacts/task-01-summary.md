# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.979 | 0.719 | 1.680 | 0.704 | 1.680 |
| health_check | 0.262 | 0.191 | 0.415 | 0.179 | 0.415 |
| get_stats | 1.614 | 1.200 | 3.323 | 1.012 | 3.323 |
| list_pending | 0.368 | 0.132 | 1.395 | 0.086 | 1.395 |
| list_unreplied_tweets_with_limit | 0.265 | 0.118 | 0.891 | 0.090 | 0.891 |

**Aggregate** — P50: 0.415 ms, P95: 1.680 ms, Min: 0.086 ms, Max: 3.323 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
