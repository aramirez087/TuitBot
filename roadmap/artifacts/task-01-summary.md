# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.088 | 1.046 | 1.488 | 0.880 | 1.488 |
| health_check | 0.254 | 0.215 | 0.469 | 0.151 | 0.469 |
| get_stats | 1.835 | 1.497 | 3.641 | 1.218 | 3.641 |
| list_pending | 0.397 | 0.154 | 1.334 | 0.107 | 1.334 |
| list_unreplied_tweets_with_limit | 0.264 | 0.129 | 0.890 | 0.078 | 0.890 |

**Aggregate** — P50: 0.469 ms, P95: 1.542 ms, Min: 0.078 ms, Max: 3.641 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
