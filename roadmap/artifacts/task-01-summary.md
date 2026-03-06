# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.021 | 0.947 | 1.370 | 0.680 | 1.370 |
| health_check | 0.290 | 0.278 | 0.532 | 0.175 | 0.532 |
| get_stats | 1.730 | 1.439 | 3.337 | 1.109 | 3.337 |
| list_pending | 0.433 | 0.156 | 1.538 | 0.112 | 1.538 |
| list_unreplied_tweets_with_limit | 0.360 | 0.166 | 0.943 | 0.104 | 0.943 |

**Aggregate** — P50: 0.532 ms, P95: 1.601 ms, Min: 0.104 ms, Max: 3.337 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
