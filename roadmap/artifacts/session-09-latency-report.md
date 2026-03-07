# Session 09 — Latency Report

**Generated:** 2026-03-07 00:29 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.016 | 0.012 | 0.026 | 0.012 | 0.026 |
| kernel::search_tweets | 0.009 | 0.008 | 0.011 | 0.008 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.029 | 0.013 | 0.029 |
| get_config | 0.092 | 0.096 | 0.099 | 0.085 | 0.099 |
| validate_config | 0.019 | 0.012 | 0.048 | 0.011 | 0.048 |
| get_mcp_tool_metrics | 0.967 | 0.530 | 2.708 | 0.464 | 2.708 |
| get_mcp_error_breakdown | 0.298 | 0.195 | 0.801 | 0.100 | 0.801 |
| get_capabilities | 0.917 | 0.988 | 1.101 | 0.655 | 1.101 |
| health_check | 0.446 | 0.436 | 0.687 | 0.284 | 0.687 |
| get_stats | 1.704 | 1.177 | 3.575 | 1.041 | 3.575 |
| list_pending | 0.418 | 0.175 | 1.457 | 0.094 | 1.457 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.016 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.099 |
| Telemetry | 2 | 2.708 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.177 ms | **Min:** 0.003 ms | **Max:** 3.575 ms

## P95 Gate

**Global P95:** 1.177 ms
**Threshold:** 50.0 ms
**Status:** PASS
