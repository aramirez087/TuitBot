# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.902 | 0.882 | 1.145 | 0.736 | 1.145 |
| health_check | 0.337 | 0.237 | 0.709 | 0.212 | 0.709 |
| get_stats | 1.517 | 1.145 | 2.723 | 1.050 | 2.723 |
| list_pending | 0.403 | 0.176 | 1.445 | 0.074 | 1.445 |
| list_unreplied_tweets_with_limit | 0.154 | 0.097 | 0.359 | 0.059 | 0.359 |

**Aggregate** — P50: 0.359 ms, P95: 1.582 ms, Min: 0.059 ms, Max: 2.723 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
