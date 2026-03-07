# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.984 | 0.784 | 1.740 | 0.668 | 1.740 |
| health_check | 0.345 | 0.256 | 0.634 | 0.244 | 0.634 |
| get_stats | 1.565 | 1.140 | 3.419 | 0.925 | 3.419 |
| list_pending | 0.353 | 0.140 | 1.259 | 0.095 | 1.259 |
| list_unreplied_tweets_with_limit | 0.217 | 0.131 | 0.591 | 0.100 | 0.591 |

**Aggregate** — P50: 0.591 ms, P95: 1.740 ms, Min: 0.095 ms, Max: 3.419 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
