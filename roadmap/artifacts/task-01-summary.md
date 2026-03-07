# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 0.949 | 0.899 | 1.688 | 0.599 | 1.688 |
| health_check | 0.285 | 0.207 | 0.502 | 0.155 | 0.502 |
| get_stats | 1.533 | 1.271 | 2.852 | 1.011 | 2.852 |
| list_pending | 0.281 | 0.106 | 0.975 | 0.089 | 0.975 |
| list_unreplied_tweets_with_limit | 0.246 | 0.133 | 0.680 | 0.127 | 0.680 |

**Aggregate** — P50: 0.502 ms, P95: 1.688 ms, Min: 0.089 ms, Max: 2.852 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
