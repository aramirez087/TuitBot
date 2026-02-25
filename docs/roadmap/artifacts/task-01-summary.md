# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.031 | 1.099 | 1.384 | 0.666 | 1.384 |
| health_check | 0.401 | 0.317 | 0.827 | 0.204 | 0.827 |
| get_stats | 1.785 | 1.365 | 3.782 | 1.100 | 3.782 |
| list_pending | 0.508 | 0.270 | 1.649 | 0.114 | 1.649 |
| list_unreplied_tweets_with_limit | 0.366 | 0.254 | 0.929 | 0.172 | 0.929 |

**Aggregate** — P50: 0.666 ms, P95: 1.649 ms, Min: 0.114 ms, Max: 3.782 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
