# Session 09 — Latency Report

**Generated:** 2026-02-28 23:37 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.017 | 0.011 | 0.017 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.012 | 0.030 | 0.012 | 0.030 |
| get_config | 0.089 | 0.087 | 0.099 | 0.083 | 0.099 |
| validate_config | 0.019 | 0.010 | 0.053 | 0.010 | 0.053 |
| get_mcp_tool_metrics | 1.097 | 0.687 | 2.744 | 0.463 | 2.744 |
| get_mcp_error_breakdown | 0.240 | 0.135 | 0.723 | 0.082 | 0.723 |
| get_capabilities | 1.024 | 0.973 | 1.605 | 0.782 | 1.605 |
| health_check | 0.398 | 0.327 | 0.873 | 0.198 | 0.873 |
| get_stats | 1.669 | 1.256 | 3.333 | 0.998 | 3.333 |
| list_pending | 0.449 | 0.148 | 1.635 | 0.120 | 1.635 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.099 |
| Telemetry | 2 | 2.744 |

## Aggregate

**P50:** 0.012 ms | **P95:** 1.566 ms | **Min:** 0.003 ms | **Max:** 3.333 ms

## P95 Gate

**Global P95:** 1.566 ms
**Threshold:** 50.0 ms
**Status:** PASS
