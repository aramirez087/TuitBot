# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.252 | 1.246 | 2.135 | 0.667 | 2.135 |
| health_check | 0.384 | 0.338 | 0.613 | 0.254 | 0.613 |
| get_stats | 1.640 | 1.258 | 2.943 | 1.079 | 2.943 |
| list_pending | 0.386 | 0.125 | 1.433 | 0.079 | 1.433 |
| list_unreplied_tweets_with_limit | 0.221 | 0.197 | 0.473 | 0.113 | 0.473 |

**Aggregate** — P50: 0.473 ms, P95: 2.135 ms, Min: 0.079 ms, Max: 2.943 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
