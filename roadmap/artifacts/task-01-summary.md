# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.146 | 0.957 | 1.877 | 0.761 | 1.877 |
| health_check | 0.297 | 0.145 | 0.757 | 0.115 | 0.757 |
| get_stats | 2.186 | 1.960 | 3.748 | 1.431 | 3.748 |
| list_pending | 0.620 | 0.192 | 2.243 | 0.168 | 2.243 |
| list_unreplied_tweets_with_limit | 0.266 | 0.088 | 0.904 | 0.087 | 0.904 |

**Aggregate** — P50: 0.757 ms, P95: 2.243 ms, Min: 0.087 ms, Max: 3.748 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
