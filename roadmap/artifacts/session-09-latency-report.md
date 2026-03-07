# Session 09 — Latency Report

**Generated:** 2026-03-07 00:44 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.011 | 0.018 | 0.011 | 0.018 |
| kernel::search_tweets | 0.009 | 0.009 | 0.011 | 0.008 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.007 | 0.006 | 0.007 |
| kernel::get_user_by_id | 0.009 | 0.009 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.009 | 0.009 | 0.009 | 0.009 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.015 | 0.013 | 0.025 | 0.012 | 0.025 |
| get_config | 0.090 | 0.089 | 0.097 | 0.087 | 0.097 |
| validate_config | 0.014 | 0.011 | 0.026 | 0.011 | 0.026 |
| get_mcp_tool_metrics | 1.004 | 0.608 | 2.690 | 0.487 | 2.690 |
| get_mcp_error_breakdown | 0.362 | 0.237 | 1.049 | 0.117 | 1.049 |
| get_capabilities | 1.012 | 0.846 | 1.568 | 0.774 | 1.568 |
| health_check | 0.439 | 0.381 | 0.684 | 0.192 | 0.684 |
| get_stats | 1.662 | 1.268 | 3.007 | 1.119 | 3.007 |
| list_pending | 0.538 | 0.187 | 1.867 | 0.158 | 1.867 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.097 |
| Telemetry | 2 | 2.690 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.568 ms | **Min:** 0.004 ms | **Max:** 3.007 ms

## P95 Gate

**Global P95:** 1.568 ms
**Threshold:** 50.0 ms
**Status:** PASS
