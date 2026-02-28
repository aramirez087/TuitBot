# Session 09 — Latency Report

**Generated:** 2026-02-28 04:01 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.018 | 0.011 | 0.018 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.019 | 0.013 | 0.044 | 0.012 | 0.044 |
| get_config | 0.089 | 0.086 | 0.098 | 0.085 | 0.098 |
| validate_config | 0.018 | 0.010 | 0.046 | 0.010 | 0.046 |
| get_mcp_tool_metrics | 0.952 | 0.592 | 2.569 | 0.443 | 2.569 |
| get_mcp_error_breakdown | 0.270 | 0.189 | 0.659 | 0.110 | 0.659 |
| get_capabilities | 1.295 | 1.371 | 1.678 | 0.823 | 1.678 |
| health_check | 0.296 | 0.210 | 0.650 | 0.137 | 0.650 |
| get_stats | 3.382 | 3.559 | 4.423 | 1.502 | 4.423 |
| list_pending | 1.654 | 0.327 | 7.128 | 0.118 | 7.128 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.098 |
| Telemetry | 2 | 2.569 |

## Aggregate

**P50:** 0.013 ms | **P95:** 3.312 ms | **Min:** 0.003 ms | **Max:** 7.128 ms

## P95 Gate

**Global P95:** 3.312 ms
**Threshold:** 50.0 ms
**Status:** PASS
