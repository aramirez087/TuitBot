# Session 09 — Latency Report

**Generated:** 2026-03-01 18:28 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.021 | 0.014 | 0.049 | 0.013 | 0.049 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.010 | 0.007 | 0.010 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.014 | 0.022 | 0.012 | 0.022 |
| get_config | 0.092 | 0.094 | 0.100 | 0.086 | 0.100 |
| validate_config | 0.021 | 0.011 | 0.063 | 0.010 | 0.063 |
| get_mcp_tool_metrics | 1.002 | 0.583 | 2.916 | 0.461 | 2.916 |
| get_mcp_error_breakdown | 0.265 | 0.147 | 0.766 | 0.093 | 0.766 |
| get_capabilities | 0.989 | 0.891 | 1.474 | 0.699 | 1.474 |
| health_check | 0.402 | 0.295 | 0.655 | 0.248 | 0.655 |
| get_stats | 2.313 | 1.756 | 4.071 | 1.617 | 4.071 |
| list_pending | 0.483 | 0.187 | 1.676 | 0.150 | 1.676 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.017 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.100 |
| Telemetry | 2 | 2.916 |

## Aggregate

**P50:** 0.015 ms | **P95:** 1.701 ms | **Min:** 0.003 ms | **Max:** 4.071 ms

## P95 Gate

**Global P95:** 1.701 ms
**Threshold:** 50.0 ms
**Status:** PASS
