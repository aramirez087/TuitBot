# Session 09 — Latency Report

**Generated:** 2026-03-01 20:42 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.012 | 0.012 | 0.016 | 0.011 | 0.016 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.005 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.014 | 0.013 | 0.020 | 0.013 | 0.020 |
| get_config | 0.092 | 0.088 | 0.109 | 0.087 | 0.109 |
| validate_config | 0.014 | 0.011 | 0.024 | 0.010 | 0.024 |
| get_mcp_tool_metrics | 0.885 | 0.545 | 2.275 | 0.512 | 2.275 |
| get_mcp_error_breakdown | 0.242 | 0.145 | 0.697 | 0.093 | 0.697 |
| get_capabilities | 0.893 | 0.826 | 1.372 | 0.649 | 1.372 |
| health_check | 0.330 | 0.332 | 0.373 | 0.285 | 0.373 |
| get_stats | 1.562 | 1.077 | 3.602 | 0.811 | 3.602 |
| list_pending | 0.330 | 0.146 | 1.156 | 0.096 | 1.156 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.109 |
| Telemetry | 2 | 2.275 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.156 ms | **Min:** 0.003 ms | **Max:** 3.602 ms

## P95 Gate

**Global P95:** 1.156 ms
**Threshold:** 50.0 ms
**Status:** PASS
