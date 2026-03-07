# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.183 | 0.966 | 1.641 | 0.833 | 1.641 |
| health_check | 0.350 | 0.247 | 0.783 | 0.185 | 0.783 |
| get_stats | 1.713 | 1.383 | 2.960 | 1.104 | 2.960 |
| list_pending | 1.161 | 0.462 | 3.709 | 0.152 | 3.709 |
| list_unreplied_tweets_with_limit | 0.259 | 0.176 | 0.772 | 0.063 | 0.772 |

**Aggregate** — P50: 0.783 ms, P95: 2.960 ms, Min: 0.063 ms, Max: 3.709 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
