# Session 09 — Latency Report

**Generated:** 2026-02-28 03:09 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.019 | 0.012 | 0.041 | 0.012 | 0.041 |
| kernel::search_tweets | 0.009 | 0.007 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.005 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.027 | 0.012 | 0.027 |
| get_config | 0.091 | 0.089 | 0.101 | 0.085 | 0.101 |
| validate_config | 0.013 | 0.011 | 0.024 | 0.010 | 0.024 |
| get_mcp_tool_metrics | 0.894 | 0.536 | 2.453 | 0.419 | 2.453 |
| get_mcp_error_breakdown | 0.192 | 0.165 | 0.431 | 0.079 | 0.431 |
| get_capabilities | 0.924 | 0.767 | 1.226 | 0.684 | 1.226 |
| health_check | 0.260 | 0.213 | 0.531 | 0.098 | 0.531 |
| get_stats | 1.533 | 1.243 | 2.765 | 0.924 | 2.765 |
| list_pending | 0.501 | 0.191 | 1.815 | 0.120 | 1.815 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.018 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.101 |
| Telemetry | 2 | 2.453 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.243 ms | **Min:** 0.003 ms | **Max:** 2.765 ms

## P95 Gate

**Global P95:** 1.243 ms
**Threshold:** 50.0 ms
**Status:** PASS
