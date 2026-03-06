# Session 09 — Latency Report

**Generated:** 2026-03-06 04:10 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.016 | 0.013 | 0.027 | 0.012 | 0.027 |
| kernel::search_tweets | 0.009 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.007 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.009 | 0.009 | 0.011 | 0.009 | 0.011 |
| kernel::get_me | 0.009 | 0.009 | 0.011 | 0.008 | 0.011 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.021 | 0.012 | 0.021 |
| get_config | 0.093 | 0.092 | 0.102 | 0.088 | 0.102 |
| validate_config | 0.099 | 0.013 | 0.442 | 0.013 | 0.442 |
| get_mcp_tool_metrics | 1.152 | 0.893 | 2.583 | 0.615 | 2.583 |
| get_mcp_error_breakdown | 0.267 | 0.161 | 0.736 | 0.120 | 0.736 |
| get_capabilities | 0.966 | 0.829 | 1.621 | 0.705 | 1.621 |
| health_check | 0.315 | 0.240 | 0.658 | 0.191 | 0.658 |
| get_stats | 1.452 | 1.180 | 2.852 | 0.959 | 2.852 |
| list_pending | 0.278 | 0.153 | 0.831 | 0.091 | 0.831 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.014 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.442 |
| Telemetry | 2 | 2.583 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.180 ms | **Min:** 0.003 ms | **Max:** 2.852 ms

## P95 Gate

**Global P95:** 1.180 ms
**Threshold:** 50.0 ms
**Status:** PASS
