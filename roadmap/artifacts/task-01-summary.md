# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.107 | 0.998 | 1.709 | 0.700 | 1.709 |
| health_check | 0.234 | 0.255 | 0.363 | 0.111 | 0.363 |
| get_stats | 1.704 | 1.258 | 3.323 | 1.205 | 3.323 |
| list_pending | 0.388 | 0.210 | 1.203 | 0.118 | 1.203 |
| list_unreplied_tweets_with_limit | 0.380 | 0.214 | 1.056 | 0.176 | 1.056 |

**Aggregate** — P50: 0.363 ms, P95: 1.709 ms, Min: 0.111 ms, Max: 3.323 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
