# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.940 | 0.794 | 1.715 | 0.614 | 1.715 |
| health_check | 0.341 | 0.256 | 0.669 | 0.178 | 0.669 |
| get_stats | 1.473 | 1.277 | 2.690 | 0.825 | 2.690 |
| list_pending | 0.358 | 0.131 | 1.296 | 0.096 | 1.296 |
| list_unreplied_tweets_with_limit | 0.382 | 0.173 | 1.292 | 0.111 | 1.292 |

**Aggregate** — P50: 0.614 ms, P95: 1.715 ms, Min: 0.096 ms, Max: 2.690 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
