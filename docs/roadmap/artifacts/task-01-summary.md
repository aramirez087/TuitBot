# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.920 | 0.885 | 1.387 | 0.643 | 1.387 |
| health_check | 0.283 | 0.202 | 0.622 | 0.154 | 0.622 |
| get_stats | 1.888 | 1.613 | 2.986 | 1.453 | 2.986 |
| list_pending | 0.578 | 0.223 | 2.027 | 0.178 | 2.027 |
| list_unreplied_tweets_with_limit | 0.253 | 0.158 | 0.562 | 0.135 | 0.562 |

**Aggregate** — P50: 0.562 ms, P95: 2.027 ms, Min: 0.135 ms, Max: 2.986 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
