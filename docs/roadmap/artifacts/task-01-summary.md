# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.883 | 0.817 | 1.313 | 0.717 | 1.313 |
| health_check | 0.397 | 0.361 | 0.860 | 0.168 | 0.860 |
| get_stats | 1.752 | 1.722 | 2.854 | 1.111 | 2.854 |
| list_pending | 0.485 | 0.221 | 1.625 | 0.152 | 1.625 |
| list_unreplied_tweets_with_limit | 0.314 | 0.172 | 0.931 | 0.121 | 0.931 |

**Aggregate** — P50: 0.717 ms, P95: 1.923 ms, Min: 0.121 ms, Max: 2.854 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
