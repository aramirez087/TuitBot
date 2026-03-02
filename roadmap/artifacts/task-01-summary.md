# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.946 | 0.834 | 1.666 | 0.526 | 1.666 |
| health_check | 0.261 | 0.160 | 0.641 | 0.145 | 0.641 |
| get_stats | 1.769 | 1.701 | 2.708 | 1.236 | 2.708 |
| list_pending | 0.487 | 0.248 | 1.271 | 0.126 | 1.271 |
| list_unreplied_tweets_with_limit | 0.365 | 0.180 | 1.110 | 0.134 | 1.110 |

**Aggregate** — P50: 0.625 ms, P95: 1.716 ms, Min: 0.126 ms, Max: 2.708 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
