# Session 09 — Latency Report

**Generated:** 2026-03-05 00:01 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.012 | 0.020 | 0.011 | 0.020 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.022 | 0.012 | 0.022 |
| get_config | 0.091 | 0.089 | 0.100 | 0.088 | 0.100 |
| validate_config | 0.015 | 0.013 | 0.028 | 0.012 | 0.028 |
| get_mcp_tool_metrics | 1.121 | 0.588 | 3.393 | 0.453 | 3.393 |
| get_mcp_error_breakdown | 0.268 | 0.189 | 0.709 | 0.102 | 0.709 |
| get_capabilities | 1.022 | 0.826 | 2.077 | 0.623 | 2.077 |
| health_check | 0.392 | 0.333 | 0.561 | 0.253 | 0.561 |
| get_stats | 1.983 | 1.440 | 4.160 | 1.223 | 4.160 |
| list_pending | 0.406 | 0.139 | 1.431 | 0.121 | 1.431 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.100 |
| Telemetry | 2 | 3.393 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.440 ms | **Min:** 0.003 ms | **Max:** 4.160 ms

## P95 Gate

**Global P95:** 1.440 ms
**Threshold:** 50.0 ms
**Status:** PASS
