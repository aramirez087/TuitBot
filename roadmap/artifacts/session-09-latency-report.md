# Session 09 — Latency Report

**Generated:** 2026-03-03 04:32 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.028 | 0.011 | 0.028 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.030 | 0.012 | 0.030 |
| get_config | 0.090 | 0.088 | 0.102 | 0.086 | 0.102 |
| validate_config | 0.015 | 0.012 | 0.027 | 0.012 | 0.027 |
| get_mcp_tool_metrics | 1.028 | 0.641 | 2.760 | 0.484 | 2.760 |
| get_mcp_error_breakdown | 0.325 | 0.175 | 0.966 | 0.125 | 0.966 |
| get_capabilities | 1.005 | 0.797 | 1.742 | 0.751 | 1.742 |
| health_check | 0.279 | 0.256 | 0.459 | 0.172 | 0.459 |
| get_stats | 1.639 | 1.333 | 2.670 | 0.954 | 2.670 |
| list_pending | 0.419 | 0.145 | 1.495 | 0.131 | 1.495 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.102 |
| Telemetry | 2 | 2.760 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.495 ms | **Min:** 0.003 ms | **Max:** 2.760 ms

## P95 Gate

**Global P95:** 1.495 ms
**Threshold:** 50.0 ms
**Status:** PASS
