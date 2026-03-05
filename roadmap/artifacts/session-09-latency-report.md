# Session 09 — Latency Report

**Generated:** 2026-03-05 21:40 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.019 | 0.012 | 0.050 | 0.011 | 0.050 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.009 | 0.009 | 0.012 | 0.008 | 0.012 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.018 | 0.013 | 0.039 | 0.012 | 0.039 |
| get_config | 0.091 | 0.089 | 0.104 | 0.086 | 0.104 |
| validate_config | 0.021 | 0.012 | 0.059 | 0.011 | 0.059 |
| get_mcp_tool_metrics | 1.130 | 0.650 | 3.061 | 0.633 | 3.061 |
| get_mcp_error_breakdown | 0.302 | 0.156 | 0.892 | 0.137 | 0.892 |
| get_capabilities | 1.008 | 0.777 | 1.850 | 0.714 | 1.850 |
| health_check | 0.428 | 0.313 | 0.762 | 0.146 | 0.762 |
| get_stats | 2.016 | 1.657 | 3.241 | 1.300 | 3.241 |
| list_pending | 0.346 | 0.142 | 1.141 | 0.120 | 1.141 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 3.061 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.657 ms | **Min:** 0.003 ms | **Max:** 3.241 ms

## P95 Gate

**Global P95:** 1.657 ms
**Threshold:** 50.0 ms
**Status:** PASS
