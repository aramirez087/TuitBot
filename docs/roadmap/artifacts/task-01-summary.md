# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.066 | 1.076 | 1.425 | 0.723 | 1.425 |
| health_check | 0.271 | 0.218 | 0.618 | 0.131 | 0.618 |
| get_stats | 1.530 | 1.253 | 2.677 | 1.128 | 2.677 |
| list_pending | 0.259 | 0.105 | 0.792 | 0.090 | 0.792 |
| list_unreplied_tweets_with_limit | 0.215 | 0.167 | 0.525 | 0.089 | 0.525 |

**Aggregate** — P50: 0.525 ms, P95: 1.425 ms, Min: 0.089 ms, Max: 2.677 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
