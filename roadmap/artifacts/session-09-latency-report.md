# Session 09 — Latency Report

**Generated:** 2026-03-01 23:02 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.016 | 0.011 | 0.016 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.008 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.015 | 0.013 | 0.022 | 0.013 | 0.022 |
| get_config | 0.093 | 0.090 | 0.104 | 0.088 | 0.104 |
| validate_config | 0.086 | 0.011 | 0.388 | 0.010 | 0.388 |
| get_mcp_tool_metrics | 1.356 | 1.097 | 2.862 | 0.649 | 2.862 |
| get_mcp_error_breakdown | 0.244 | 0.131 | 0.782 | 0.081 | 0.782 |
| get_capabilities | 0.845 | 0.758 | 1.288 | 0.660 | 1.288 |
| health_check | 0.360 | 0.314 | 0.711 | 0.181 | 0.711 |
| get_stats | 1.632 | 1.269 | 3.518 | 0.926 | 3.518 |
| list_pending | 0.438 | 0.148 | 1.531 | 0.127 | 1.531 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.388 |
| Telemetry | 2 | 2.862 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.288 ms | **Min:** 0.003 ms | **Max:** 3.518 ms

## P95 Gate

**Global P95:** 1.288 ms
**Threshold:** 50.0 ms
**Status:** PASS
