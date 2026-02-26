# Session 09 — Latency Report

**Generated:** 2026-02-26 17:32 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.012 | 0.033 | 0.011 | 0.033 |
| kernel::search_tweets | 0.008 | 0.007 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.012 | 0.022 | 0.012 | 0.022 |
| get_config | 0.088 | 0.084 | 0.097 | 0.082 | 0.097 |
| validate_config | 0.013 | 0.010 | 0.022 | 0.010 | 0.022 |
| get_mcp_tool_metrics | 1.291 | 0.813 | 3.201 | 0.701 | 3.201 |
| get_mcp_error_breakdown | 0.304 | 0.239 | 0.704 | 0.125 | 0.704 |
| get_capabilities | 0.868 | 0.888 | 1.205 | 0.636 | 1.205 |
| health_check | 0.283 | 0.197 | 0.617 | 0.162 | 0.617 |
| get_stats | 1.827 | 1.563 | 2.854 | 1.326 | 2.854 |
| list_pending | 0.422 | 0.180 | 1.448 | 0.113 | 1.448 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.015 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.097 |
| Telemetry | 2 | 3.201 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.448 ms | **Min:** 0.003 ms | **Max:** 3.201 ms

## P95 Gate

**Global P95:** 1.448 ms
**Threshold:** 50.0 ms
**Status:** PASS
