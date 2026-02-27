# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.022 | 0.911 | 1.636 | 0.749 | 1.636 |
| health_check | 0.223 | 0.175 | 0.433 | 0.146 | 0.433 |
| get_stats | 1.687 | 1.217 | 3.192 | 1.032 | 3.192 |
| list_pending | 0.347 | 0.120 | 1.282 | 0.075 | 1.282 |
| list_unreplied_tweets_with_limit | 0.240 | 0.104 | 0.820 | 0.078 | 0.820 |

**Aggregate** — P50: 0.433 ms, P95: 1.797 ms, Min: 0.075 ms, Max: 3.192 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
