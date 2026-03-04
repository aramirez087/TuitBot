# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.997 | 0.838 | 1.678 | 0.802 | 1.678 |
| health_check | 0.309 | 0.276 | 0.642 | 0.118 | 0.642 |
| get_stats | 1.999 | 1.718 | 3.581 | 1.409 | 3.581 |
| list_pending | 0.441 | 0.156 | 1.555 | 0.091 | 1.555 |
| list_unreplied_tweets_with_limit | 0.415 | 0.189 | 1.307 | 0.174 | 1.307 |

**Aggregate** — P50: 0.642 ms, P95: 1.759 ms, Min: 0.091 ms, Max: 3.581 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
