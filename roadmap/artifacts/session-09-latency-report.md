# Session 09 — Latency Report

**Generated:** 2026-03-01 23:40 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.028 | 0.011 | 0.028 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.027 | 0.012 | 0.027 |
| get_config | 0.093 | 0.091 | 0.104 | 0.087 | 0.104 |
| validate_config | 0.014 | 0.011 | 0.026 | 0.010 | 0.026 |
| get_mcp_tool_metrics | 0.961 | 0.622 | 2.578 | 0.430 | 2.578 |
| get_mcp_error_breakdown | 0.240 | 0.128 | 0.680 | 0.126 | 0.680 |
| get_capabilities | 0.937 | 0.736 | 1.685 | 0.660 | 1.685 |
| health_check | 0.377 | 0.321 | 0.771 | 0.156 | 0.771 |
| get_stats | 1.572 | 1.134 | 3.273 | 1.046 | 3.273 |
| list_pending | 0.450 | 0.204 | 1.575 | 0.083 | 1.575 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 2.578 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.361 ms | **Min:** 0.004 ms | **Max:** 3.273 ms

## P95 Gate

**Global P95:** 1.361 ms
**Threshold:** 50.0 ms
**Status:** PASS
