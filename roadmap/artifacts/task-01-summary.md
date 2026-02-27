# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.188 | 0.995 | 2.439 | 0.669 | 2.439 |
| health_check | 0.328 | 0.214 | 0.684 | 0.150 | 0.684 |
| get_stats | 1.678 | 1.250 | 3.382 | 1.146 | 3.382 |
| list_pending | 0.312 | 0.110 | 1.146 | 0.076 | 1.146 |
| list_unreplied_tweets_with_limit | 0.248 | 0.157 | 0.710 | 0.087 | 0.710 |

**Aggregate** — P50: 0.669 ms, P95: 2.439 ms, Min: 0.076 ms, Max: 3.382 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
