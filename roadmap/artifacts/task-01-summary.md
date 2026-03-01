# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.946 | 0.899 | 1.332 | 0.752 | 1.332 |
| health_check | 0.253 | 0.195 | 0.518 | 0.142 | 0.518 |
| get_stats | 1.660 | 1.191 | 3.019 | 1.149 | 3.019 |
| list_pending | 0.405 | 0.145 | 1.494 | 0.092 | 1.494 |
| list_unreplied_tweets_with_limit | 0.292 | 0.171 | 0.841 | 0.108 | 0.841 |

**Aggregate** — P50: 0.518 ms, P95: 1.793 ms, Min: 0.092 ms, Max: 3.019 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
