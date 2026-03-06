# Session 09 — Latency Report

**Generated:** 2026-03-06 05:22 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.029 | 0.011 | 0.029 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.031 | 0.012 | 0.031 |
| get_config | 0.092 | 0.090 | 0.101 | 0.088 | 0.101 |
| validate_config | 0.014 | 0.012 | 0.026 | 0.011 | 0.026 |
| get_mcp_tool_metrics | 1.061 | 0.601 | 3.077 | 0.470 | 3.077 |
| get_mcp_error_breakdown | 0.229 | 0.095 | 0.715 | 0.086 | 0.715 |
| get_capabilities | 1.097 | 1.039 | 1.470 | 0.779 | 1.470 |
| health_check | 0.415 | 0.389 | 0.716 | 0.232 | 0.716 |
| get_stats | 1.978 | 1.681 | 2.677 | 1.405 | 2.677 |
| list_pending | 0.383 | 0.128 | 1.407 | 0.120 | 1.407 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.101 |
| Telemetry | 2 | 3.077 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.643 ms | **Min:** 0.003 ms | **Max:** 3.077 ms

## P95 Gate

**Global P95:** 1.643 ms
**Threshold:** 50.0 ms
**Status:** PASS
