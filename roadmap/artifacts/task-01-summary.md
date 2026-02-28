# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.874 | 0.915 | 1.103 | 0.684 | 1.103 |
| health_check | 0.262 | 0.164 | 0.674 | 0.112 | 0.674 |
| get_stats | 1.470 | 1.139 | 2.943 | 1.007 | 2.943 |
| list_pending | 0.324 | 0.120 | 1.080 | 0.110 | 1.080 |
| list_unreplied_tweets_with_limit | 0.201 | 0.120 | 0.529 | 0.112 | 0.529 |

**Aggregate** — P50: 0.529 ms, P95: 1.233 ms, Min: 0.110 ms, Max: 2.943 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
