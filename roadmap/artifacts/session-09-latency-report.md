# Session 09 — Latency Report

**Generated:** 2026-03-01 22:46 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.013 | 0.017 | 0.012 | 0.017 |
| kernel::search_tweets | 0.009 | 0.008 | 0.011 | 0.008 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.011 | 0.007 | 0.011 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.013 | 0.022 | 0.012 | 0.022 |
| get_config | 0.095 | 0.094 | 0.105 | 0.089 | 0.105 |
| validate_config | 0.075 | 0.011 | 0.332 | 0.010 | 0.332 |
| get_mcp_tool_metrics | 1.160 | 0.738 | 3.265 | 0.462 | 3.265 |
| get_mcp_error_breakdown | 0.226 | 0.170 | 0.531 | 0.075 | 0.531 |
| get_capabilities | 1.031 | 0.944 | 1.749 | 0.665 | 1.749 |
| health_check | 0.226 | 0.181 | 0.397 | 0.165 | 0.397 |
| get_stats | 1.548 | 1.118 | 3.264 | 0.986 | 3.264 |
| list_pending | 0.359 | 0.111 | 1.297 | 0.092 | 1.297 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.332 |
| Telemetry | 2 | 3.265 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.297 ms | **Min:** 0.003 ms | **Max:** 3.265 ms

## P95 Gate

**Global P95:** 1.297 ms
**Threshold:** 50.0 ms
**Status:** PASS
