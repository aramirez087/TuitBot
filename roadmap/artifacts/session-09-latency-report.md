# Session 09 — Latency Report

**Generated:** 2026-02-28 18:36 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.011 | 0.028 | 0.011 | 0.028 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.012 | 0.031 | 0.012 | 0.031 |
| get_config | 0.088 | 0.086 | 0.099 | 0.083 | 0.099 |
| validate_config | 0.013 | 0.011 | 0.023 | 0.010 | 0.023 |
| get_mcp_tool_metrics | 0.986 | 0.571 | 2.637 | 0.536 | 2.637 |
| get_mcp_error_breakdown | 0.296 | 0.154 | 0.885 | 0.091 | 0.885 |
| get_capabilities | 0.965 | 0.985 | 1.351 | 0.731 | 1.351 |
| health_check | 0.283 | 0.254 | 0.566 | 0.161 | 0.566 |
| get_stats | 2.003 | 1.950 | 3.067 | 1.138 | 3.067 |
| list_pending | 0.514 | 0.178 | 1.713 | 0.156 | 1.713 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.099 |
| Telemetry | 2 | 2.637 |

## Aggregate

**P50:** 0.012 ms | **P95:** 1.713 ms | **Min:** 0.003 ms | **Max:** 3.067 ms

## P95 Gate

**Global P95:** 1.713 ms
**Threshold:** 50.0 ms
**Status:** PASS
