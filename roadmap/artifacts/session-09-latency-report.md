# Session 09 — Latency Report

**Generated:** 2026-03-01 05:19 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.019 | 0.012 | 0.049 | 0.011 | 0.049 |
| kernel::search_tweets | 0.017 | 0.015 | 0.035 | 0.009 | 0.035 |
| kernel::get_followers | 0.016 | 0.014 | 0.031 | 0.011 | 0.031 |
| kernel::get_user_by_id | 0.013 | 0.008 | 0.035 | 0.008 | 0.035 |
| kernel::get_me | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.005 | 0.003 | 0.005 |
| score_tweet | 0.014 | 0.013 | 0.019 | 0.013 | 0.019 |
| get_config | 0.092 | 0.089 | 0.101 | 0.088 | 0.101 |
| validate_config | 0.013 | 0.010 | 0.024 | 0.010 | 0.024 |
| get_mcp_tool_metrics | 1.241 | 0.855 | 2.466 | 0.717 | 2.466 |
| get_mcp_error_breakdown | 0.355 | 0.158 | 1.152 | 0.119 | 1.152 |
| get_capabilities | 0.909 | 0.893 | 1.263 | 0.674 | 1.263 |
| health_check | 0.326 | 0.192 | 0.676 | 0.171 | 0.676 |
| get_stats | 1.530 | 1.103 | 3.466 | 0.835 | 3.466 |
| list_pending | 0.434 | 0.194 | 1.472 | 0.135 | 1.472 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.035 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.101 |
| Telemetry | 2 | 2.466 |

## Aggregate

**P50:** 0.024 ms | **P95:** 1.263 ms | **Min:** 0.003 ms | **Max:** 3.466 ms

## P95 Gate

**Global P95:** 1.263 ms
**Threshold:** 50.0 ms
**Status:** PASS
