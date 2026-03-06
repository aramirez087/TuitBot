# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.944 | 0.868 | 1.328 | 0.630 | 1.328 |
| health_check | 0.260 | 0.231 | 0.517 | 0.108 | 0.517 |
| get_stats | 1.620 | 1.199 | 2.816 | 1.047 | 2.816 |
| list_pending | 0.438 | 0.286 | 1.147 | 0.154 | 1.147 |
| list_unreplied_tweets_with_limit | 0.418 | 0.275 | 1.210 | 0.109 | 1.210 |

**Aggregate** — P50: 0.517 ms, P95: 1.971 ms, Min: 0.108 ms, Max: 2.816 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
