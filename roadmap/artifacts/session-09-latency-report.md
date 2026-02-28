# Session 09 — Latency Report

**Generated:** 2026-02-28 04:26 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.011 | 0.021 | 0.011 | 0.021 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.021 | 0.012 | 0.054 | 0.012 | 0.054 |
| get_config | 0.091 | 0.088 | 0.099 | 0.087 | 0.099 |
| validate_config | 0.054 | 0.010 | 0.230 | 0.010 | 0.230 |
| get_mcp_tool_metrics | 1.231 | 0.805 | 3.220 | 0.599 | 3.220 |
| get_mcp_error_breakdown | 0.315 | 0.206 | 0.828 | 0.117 | 0.828 |
| get_capabilities | 1.095 | 0.934 | 1.464 | 0.766 | 1.464 |
| health_check | 0.198 | 0.129 | 0.453 | 0.115 | 0.453 |
| get_stats | 1.759 | 1.380 | 3.231 | 1.180 | 3.231 |
| list_pending | 0.371 | 0.150 | 1.184 | 0.115 | 1.184 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.230 |
| Telemetry | 2 | 3.220 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.402 ms | **Min:** 0.003 ms | **Max:** 3.231 ms

## P95 Gate

**Global P95:** 1.402 ms
**Threshold:** 50.0 ms
**Status:** PASS
