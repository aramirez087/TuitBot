# Session 09 — Latency Report

**Generated:** 2026-03-02 01:19 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.011 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.028 | 0.012 | 0.028 |
| get_config | 0.090 | 0.087 | 0.103 | 0.086 | 0.103 |
| validate_config | 0.013 | 0.010 | 0.025 | 0.010 | 0.025 |
| get_mcp_tool_metrics | 0.877 | 0.470 | 2.528 | 0.377 | 2.528 |
| get_mcp_error_breakdown | 0.249 | 0.189 | 0.574 | 0.109 | 0.574 |
| get_capabilities | 0.816 | 0.681 | 1.343 | 0.652 | 1.343 |
| health_check | 0.258 | 0.146 | 0.589 | 0.128 | 0.589 |
| get_stats | 1.402 | 1.172 | 2.502 | 0.986 | 2.502 |
| list_pending | 0.391 | 0.169 | 1.335 | 0.120 | 1.335 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.103 |
| Telemetry | 2 | 2.528 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.261 ms | **Min:** 0.003 ms | **Max:** 2.528 ms

## P95 Gate

**Global P95:** 1.261 ms
**Threshold:** 50.0 ms
**Status:** PASS
