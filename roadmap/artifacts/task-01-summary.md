# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.936 | 0.822 | 1.559 | 0.632 | 1.559 |
| health_check | 0.221 | 0.208 | 0.290 | 0.180 | 0.290 |
| get_stats | 1.591 | 1.364 | 2.771 | 1.038 | 2.771 |
| list_pending | 0.302 | 0.153 | 0.933 | 0.129 | 0.933 |
| list_unreplied_tweets_with_limit | 0.229 | 0.095 | 0.761 | 0.072 | 0.761 |

**Aggregate** — P50: 0.290 ms, P95: 1.559 ms, Min: 0.072 ms, Max: 2.771 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
