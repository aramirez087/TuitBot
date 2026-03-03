# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.843 | 0.762 | 1.203 | 0.698 | 1.203 |
| health_check | 0.232 | 0.175 | 0.385 | 0.137 | 0.385 |
| get_stats | 1.412 | 1.194 | 2.695 | 0.895 | 2.695 |
| list_pending | 0.409 | 0.130 | 1.521 | 0.113 | 1.521 |
| list_unreplied_tweets_with_limit | 0.319 | 0.186 | 0.848 | 0.155 | 0.848 |

**Aggregate** — P50: 0.385 ms, P95: 1.521 ms, Min: 0.113 ms, Max: 2.695 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
