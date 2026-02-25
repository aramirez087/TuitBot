# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.867 | 0.940 | 1.065 | 0.601 | 1.065 |
| health_check | 0.181 | 0.109 | 0.436 | 0.101 | 0.436 |
| get_stats | 1.380 | 1.192 | 2.547 | 0.889 | 2.547 |
| list_pending | 0.262 | 0.125 | 0.840 | 0.067 | 0.840 |
| list_unreplied_tweets_with_limit | 0.191 | 0.117 | 0.525 | 0.057 | 0.525 |

**Aggregate** — P50: 0.436 ms, P95: 1.231 ms, Min: 0.057 ms, Max: 2.547 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
