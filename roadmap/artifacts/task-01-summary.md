# Task 01 — Baseline Benchmark

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| get_capabilities | 1.047 | 0.831 | 1.992 | 0.642 | 1.992 |
| health_check | 0.329 | 0.237 | 0.721 | 0.203 | 0.721 |
| get_stats | 1.844 | 1.623 | 3.272 | 1.083 | 3.272 |
| list_pending | 0.406 | 0.163 | 0.964 | 0.113 | 0.964 |
| list_unreplied_tweets_with_limit | 0.342 | 0.192 | 0.943 | 0.171 | 0.943 |

**Aggregate** — P50: 0.661 ms, P95: 1.992 ms, Min: 0.113 ms, Max: 3.272 ms

Migrated: 5 / 27 tools — Schema pass rate: 100%
