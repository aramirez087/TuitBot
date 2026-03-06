# Session 09 — Latency Report

**Generated:** 2026-03-06 05:12 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.028 | 0.012 | 0.028 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.013 | 0.008 | 0.034 | 0.006 | 0.034 |
| kernel::get_user_by_id | 0.017 | 0.011 | 0.040 | 0.008 | 0.040 |
| kernel::get_me | 0.009 | 0.009 | 0.012 | 0.009 | 0.012 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.015 | 0.014 | 0.020 | 0.013 | 0.020 |
| get_config | 0.091 | 0.090 | 0.099 | 0.086 | 0.099 |
| validate_config | 0.074 | 0.012 | 0.319 | 0.012 | 0.319 |
| get_mcp_tool_metrics | 1.175 | 0.773 | 2.771 | 0.616 | 2.771 |
| get_mcp_error_breakdown | 0.251 | 0.137 | 0.724 | 0.111 | 0.724 |
| get_capabilities | 0.910 | 0.858 | 1.208 | 0.720 | 1.208 |
| health_check | 0.302 | 0.236 | 0.556 | 0.204 | 0.556 |
| get_stats | 1.410 | 1.108 | 2.704 | 0.982 | 2.704 |
| list_pending | 0.369 | 0.179 | 1.278 | 0.060 | 1.278 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.034 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.319 |
| Telemetry | 2 | 2.771 |

## Aggregate

**P50:** 0.020 ms | **P95:** 1.208 ms | **Min:** 0.004 ms | **Max:** 2.771 ms

## P95 Gate

**Global P95:** 1.208 ms
**Threshold:** 50.0 ms
**Status:** PASS
