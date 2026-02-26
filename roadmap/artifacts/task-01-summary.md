# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.999 | 0.908 | 1.371 | 0.858 | 1.371 |
| health_check | 0.335 | 0.211 | 0.836 | 0.180 | 0.836 |
| get_stats | 1.721 | 1.534 | 2.776 | 1.361 | 2.776 |
| list_pending | 0.464 | 0.154 | 1.743 | 0.114 | 1.743 |
| list_unreplied_tweets_with_limit | 0.307 | 0.130 | 0.995 | 0.105 | 0.995 |

**Aggregate** — P50: 0.836 ms, P95: 1.743 ms, Min: 0.105 ms, Max: 2.776 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
