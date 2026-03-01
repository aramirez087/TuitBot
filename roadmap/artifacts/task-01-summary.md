# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.244 | 1.205 | 1.725 | 0.932 | 1.725 |
| health_check | 0.457 | 0.332 | 0.826 | 0.200 | 0.826 |
| get_stats | 2.162 | 1.673 | 4.394 | 1.473 | 4.394 |
| list_pending | 0.621 | 0.367 | 1.539 | 0.162 | 1.539 |
| list_unreplied_tweets_with_limit | 0.261 | 0.107 | 0.852 | 0.103 | 0.852 |

**Aggregate** — P50: 0.826 ms, P95: 1.725 ms, Min: 0.103 ms, Max: 4.394 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
