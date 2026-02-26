# Session 09 — Latency Report

**Generated:** 2026-02-26 03:09 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.017 | 0.011 | 0.017 |
| kernel::search_tweets | 0.009 | 0.009 | 0.012 | 0.008 | 0.012 |
| kernel::get_followers | 0.007 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_me | 0.009 | 0.009 | 0.009 | 0.009 | 0.009 |
| kernel::post_tweet | 0.005 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.020 | 0.014 | 0.044 | 0.013 | 0.044 |
| get_config | 0.085 | 0.083 | 0.096 | 0.081 | 0.096 |
| validate_config | 0.019 | 0.010 | 0.055 | 0.010 | 0.055 |
| get_mcp_tool_metrics | 1.499 | 1.152 | 3.214 | 0.894 | 3.214 |
| get_mcp_error_breakdown | 0.317 | 0.223 | 0.755 | 0.183 | 0.755 |
| get_capabilities | 0.979 | 0.896 | 1.693 | 0.635 | 1.693 |
| health_check | 0.473 | 0.398 | 1.000 | 0.193 | 1.000 |
| get_stats | 2.071 | 1.849 | 3.137 | 1.429 | 3.137 |
| list_pending | 0.471 | 0.264 | 1.412 | 0.200 | 1.412 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.096 |
| Telemetry | 2 | 3.214 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.693 ms | **Min:** 0.004 ms | **Max:** 3.214 ms

## P95 Gate

**Global P95:** 1.693 ms
**Threshold:** 50.0 ms
**Status:** PASS
