# Session 09 — Latency Report

**Generated:** 2026-03-06 02:56 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.013 | 0.019 | 0.012 | 0.019 |
| kernel::search_tweets | 0.010 | 0.009 | 0.013 | 0.009 | 0.013 |
| kernel::get_followers | 0.007 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.009 | 0.009 | 0.011 | 0.008 | 0.011 |
| kernel::get_me | 0.009 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.019 | 0.013 | 0.045 | 0.012 | 0.045 |
| get_config | 0.097 | 0.098 | 0.098 | 0.092 | 0.098 |
| validate_config | 0.103 | 0.013 | 0.460 | 0.012 | 0.460 |
| get_mcp_tool_metrics | 1.003 | 0.756 | 2.428 | 0.465 | 2.428 |
| get_mcp_error_breakdown | 0.323 | 0.137 | 1.021 | 0.107 | 1.021 |
| get_capabilities | 0.877 | 0.781 | 1.284 | 0.619 | 1.284 |
| health_check | 0.258 | 0.186 | 0.557 | 0.165 | 0.557 |
| get_stats | 1.741 | 1.055 | 4.282 | 0.939 | 4.282 |
| list_pending | 0.371 | 0.145 | 1.330 | 0.091 | 1.330 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.460 |
| Telemetry | 2 | 2.428 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.284 ms | **Min:** 0.004 ms | **Max:** 4.282 ms

## P95 Gate

**Global P95:** 1.284 ms
**Threshold:** 50.0 ms
**Status:** PASS
