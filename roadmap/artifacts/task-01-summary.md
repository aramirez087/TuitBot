# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.974 | 1.002 | 1.251 | 0.724 | 1.251 |
| health_check | 0.226 | 0.231 | 0.275 | 0.138 | 0.275 |
| get_stats | 1.568 | 1.200 | 3.306 | 1.027 | 3.306 |
| list_pending | 0.366 | 0.117 | 1.414 | 0.053 | 1.414 |
| list_unreplied_tweets_with_limit | 0.288 | 0.132 | 0.947 | 0.102 | 0.947 |

**Aggregate** — P50: 0.275 ms, P95: 1.414 ms, Min: 0.053 ms, Max: 3.306 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
