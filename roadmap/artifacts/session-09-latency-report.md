# Session 09 — Latency Report

**Generated:** 2026-03-02 01:50 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.011 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.008 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.005 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.028 | 0.011 | 0.028 |
| get_config | 0.090 | 0.088 | 0.102 | 0.084 | 0.102 |
| validate_config | 0.013 | 0.010 | 0.025 | 0.010 | 0.025 |
| get_mcp_tool_metrics | 0.925 | 0.626 | 2.201 | 0.490 | 2.201 |
| get_mcp_error_breakdown | 0.287 | 0.146 | 0.974 | 0.079 | 0.974 |
| get_capabilities | 0.864 | 0.787 | 1.505 | 0.564 | 1.505 |
| health_check | 0.306 | 0.249 | 0.598 | 0.166 | 0.598 |
| get_stats | 1.729 | 1.349 | 3.170 | 1.138 | 3.170 |
| list_pending | 0.437 | 0.179 | 1.314 | 0.158 | 1.314 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.102 |
| Telemetry | 2 | 2.201 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.349 ms | **Min:** 0.003 ms | **Max:** 3.170 ms

## P95 Gate

**Global P95:** 1.349 ms
**Threshold:** 50.0 ms
**Status:** PASS
