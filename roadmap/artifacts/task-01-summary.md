# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.873 | 0.834 | 1.171 | 0.620 | 1.171 |
| health_check | 0.340 | 0.237 | 0.560 | 0.233 | 0.560 |
| get_stats | 2.082 | 1.855 | 3.735 | 1.146 | 3.735 |
| list_pending | 0.375 | 0.195 | 1.130 | 0.124 | 1.130 |
| list_unreplied_tweets_with_limit | 0.242 | 0.158 | 0.638 | 0.112 | 0.638 |

**Aggregate** — P50: 0.560 ms, P95: 1.884 ms, Min: 0.112 ms, Max: 3.735 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
