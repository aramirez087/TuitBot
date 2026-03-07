# Session 09 — Latency Report

**Generated:** 2026-03-07 03:42 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.028 | 0.011 | 0.028 |
| kernel::search_tweets | 0.009 | 0.008 | 0.014 | 0.007 | 0.014 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.009 | 0.009 | 0.009 | 0.009 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.018 | 0.013 | 0.039 | 0.012 | 0.039 |
| get_config | 0.091 | 0.089 | 0.104 | 0.087 | 0.104 |
| validate_config | 0.021 | 0.012 | 0.058 | 0.011 | 0.058 |
| get_mcp_tool_metrics | 1.148 | 0.689 | 3.088 | 0.553 | 3.088 |
| get_mcp_error_breakdown | 0.253 | 0.196 | 0.578 | 0.135 | 0.578 |
| get_capabilities | 0.991 | 0.953 | 1.283 | 0.675 | 1.283 |
| health_check | 0.176 | 0.110 | 0.471 | 0.081 | 0.471 |
| get_stats | 1.398 | 1.224 | 2.452 | 0.939 | 2.452 |
| list_pending | 0.462 | 0.211 | 1.430 | 0.173 | 1.430 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.014 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 3.088 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.234 ms | **Min:** 0.003 ms | **Max:** 3.088 ms

## P95 Gate

**Global P95:** 1.234 ms
**Threshold:** 50.0 ms
**Status:** PASS
