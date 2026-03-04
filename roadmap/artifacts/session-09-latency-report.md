# Session 09 — Latency Report

**Generated:** 2026-03-04 04:06 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.011 | 0.027 | 0.011 | 0.027 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.014 | 0.012 | 0.022 | 0.012 | 0.022 |
| get_config | 0.095 | 0.096 | 0.108 | 0.087 | 0.108 |
| validate_config | 0.016 | 0.013 | 0.028 | 0.012 | 0.028 |
| get_mcp_tool_metrics | 1.080 | 0.560 | 3.073 | 0.491 | 3.073 |
| get_mcp_error_breakdown | 0.288 | 0.229 | 0.736 | 0.094 | 0.736 |
| get_capabilities | 1.088 | 0.941 | 1.756 | 0.843 | 1.756 |
| health_check | 0.226 | 0.161 | 0.499 | 0.123 | 0.499 |
| get_stats | 1.634 | 1.309 | 3.145 | 1.163 | 3.145 |
| list_pending | 0.385 | 0.137 | 1.377 | 0.083 | 1.377 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.108 |
| Telemetry | 2 | 3.073 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.339 ms | **Min:** 0.004 ms | **Max:** 3.145 ms

## P95 Gate

**Global P95:** 1.339 ms
**Threshold:** 50.0 ms
**Status:** PASS
