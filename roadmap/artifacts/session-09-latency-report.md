# Session 09 — Latency Report

**Generated:** 2026-02-27 03:58 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.017 | 0.011 | 0.017 |
| kernel::search_tweets | 0.008 | 0.007 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.007 | 0.006 | 0.007 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.021 | 0.013 | 0.021 |
| get_config | 0.084 | 0.080 | 0.093 | 0.079 | 0.093 |
| validate_config | 0.094 | 0.011 | 0.425 | 0.010 | 0.425 |
| get_mcp_tool_metrics | 1.259 | 0.813 | 3.062 | 0.609 | 3.062 |
| get_mcp_error_breakdown | 0.265 | 0.184 | 0.629 | 0.128 | 0.629 |
| get_capabilities | 0.944 | 0.789 | 1.590 | 0.667 | 1.590 |
| health_check | 0.279 | 0.270 | 0.540 | 0.103 | 0.540 |
| get_stats | 1.544 | 1.091 | 3.376 | 1.044 | 3.376 |
| list_pending | 0.366 | 0.096 | 1.376 | 0.076 | 1.376 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.425 |
| Telemetry | 2 | 3.062 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.152 ms | **Min:** 0.003 ms | **Max:** 3.376 ms

## P95 Gate

**Global P95:** 1.152 ms
**Threshold:** 50.0 ms
**Status:** PASS
