# Session 09 — Latency Report

**Generated:** 2026-03-04 04:25 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.028 | 0.011 | 0.028 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.007 | 0.003 | 0.007 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.017 | 0.012 | 0.038 | 0.011 | 0.038 |
| get_config | 0.090 | 0.086 | 0.106 | 0.084 | 0.106 |
| validate_config | 0.065 | 0.012 | 0.278 | 0.011 | 0.278 |
| get_mcp_tool_metrics | 1.183 | 0.761 | 2.998 | 0.668 | 2.998 |
| get_mcp_error_breakdown | 0.251 | 0.153 | 0.582 | 0.121 | 0.582 |
| get_capabilities | 1.058 | 0.871 | 1.708 | 0.683 | 1.708 |
| health_check | 0.274 | 0.191 | 0.525 | 0.167 | 0.525 |
| get_stats | 2.010 | 2.027 | 3.154 | 1.150 | 3.154 |
| list_pending | 0.371 | 0.165 | 1.238 | 0.128 | 1.238 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.007 |
| Config | 3 | 0.278 |
| Telemetry | 2 | 2.998 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.708 ms | **Min:** 0.003 ms | **Max:** 3.154 ms

## P95 Gate

**Global P95:** 1.708 ms
**Threshold:** 50.0 ms
**Status:** PASS
