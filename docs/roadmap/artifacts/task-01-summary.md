# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.120 | 1.038 | 1.590 | 0.878 | 1.590 |
| health_check | 0.507 | 0.396 | 1.067 | 0.268 | 1.067 |
| get_stats | 1.991 | 1.654 | 3.560 | 1.529 | 3.560 |
| list_pending | 0.508 | 0.205 | 1.613 | 0.112 | 1.613 |
| list_unreplied_tweets_with_limit | 0.336 | 0.193 | 0.796 | 0.126 | 0.796 |

**Aggregate** — P50: 0.796 ms, P95: 1.666 ms, Min: 0.112 ms, Max: 3.560 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
