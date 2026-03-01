# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.370 | 1.274 | 1.990 | 0.787 | 1.990 |
| health_check | 0.309 | 0.273 | 0.473 | 0.230 | 0.473 |
| get_stats | 1.752 | 1.375 | 3.516 | 1.090 | 3.516 |
| list_pending | 0.377 | 0.156 | 1.323 | 0.071 | 1.323 |
| list_unreplied_tweets_with_limit | 0.196 | 0.123 | 0.507 | 0.094 | 0.507 |

**Aggregate** — P50: 0.473 ms, P95: 1.990 ms, Min: 0.071 ms, Max: 3.516 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
