# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.943 | 0.693 | 1.774 | 0.607 | 1.774 |
| health_check | 0.296 | 0.194 | 0.608 | 0.124 | 0.608 |
| get_stats | 1.562 | 1.330 | 2.483 | 1.219 | 2.483 |
| list_pending | 0.275 | 0.126 | 0.904 | 0.093 | 0.904 |
| list_unreplied_tweets_with_limit | 0.216 | 0.128 | 0.586 | 0.074 | 0.586 |

**Aggregate** — P50: 0.586 ms, P95: 1.774 ms, Min: 0.074 ms, Max: 2.483 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
