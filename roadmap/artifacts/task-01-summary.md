# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.968 | 0.920 | 1.658 | 0.593 | 1.658 |
| health_check | 0.350 | 0.205 | 0.916 | 0.169 | 0.916 |
| get_stats | 1.400 | 1.139 | 2.770 | 0.819 | 2.770 |
| list_pending | 0.248 | 0.129 | 0.772 | 0.069 | 0.772 |
| list_unreplied_tweets_with_limit | 0.186 | 0.103 | 0.521 | 0.095 | 0.521 |

**Aggregate** — P50: 0.521 ms, P95: 1.658 ms, Min: 0.069 ms, Max: 2.770 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
