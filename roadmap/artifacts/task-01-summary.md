# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.032 | 0.901 | 1.725 | 0.616 | 1.725 |
| health_check | 0.309 | 0.186 | 0.734 | 0.154 | 0.734 |
| get_stats | 1.638 | 1.190 | 3.330 | 1.123 | 3.330 |
| list_pending | 0.356 | 0.152 | 1.171 | 0.116 | 1.171 |
| list_unreplied_tweets_with_limit | 0.302 | 0.094 | 1.146 | 0.073 | 1.146 |

**Aggregate** — P50: 0.616 ms, P95: 1.725 ms, Min: 0.073 ms, Max: 3.330 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
