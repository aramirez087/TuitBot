# Session 09 — Latency Report

**Generated:** 2026-03-03 04:21 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.008 | 0.007 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.018 | 0.013 | 0.036 | 0.012 | 0.036 |
| get_config | 0.092 | 0.089 | 0.105 | 0.087 | 0.105 |
| validate_config | 0.023 | 0.012 | 0.067 | 0.011 | 0.067 |
| get_mcp_tool_metrics | 1.070 | 0.591 | 2.995 | 0.439 | 2.995 |
| get_mcp_error_breakdown | 0.327 | 0.195 | 0.999 | 0.118 | 0.999 |
| get_capabilities | 0.841 | 0.783 | 1.330 | 0.539 | 1.330 |
| health_check | 0.267 | 0.259 | 0.507 | 0.125 | 0.507 |
| get_stats | 1.830 | 1.624 | 3.121 | 1.237 | 3.121 |
| list_pending | 0.369 | 0.134 | 1.396 | 0.083 | 1.396 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.105 |
| Telemetry | 2 | 2.995 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.434 ms | **Min:** 0.003 ms | **Max:** 3.121 ms

## P95 Gate

**Global P95:** 1.434 ms
**Threshold:** 50.0 ms
**Status:** PASS
