# Session 09 — Latency Report

**Generated:** 2026-03-05 02:00 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.016 | 0.011 | 0.035 | 0.011 | 0.035 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.009 | 0.009 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.029 | 0.012 | 0.029 |
| get_config | 0.089 | 0.088 | 0.098 | 0.086 | 0.098 |
| validate_config | 0.021 | 0.012 | 0.057 | 0.012 | 0.057 |
| get_mcp_tool_metrics | 0.993 | 0.651 | 2.518 | 0.527 | 2.518 |
| get_mcp_error_breakdown | 0.223 | 0.147 | 0.584 | 0.106 | 0.584 |
| get_capabilities | 0.923 | 0.999 | 1.116 | 0.696 | 1.116 |
| health_check | 0.345 | 0.309 | 0.629 | 0.182 | 0.629 |
| get_stats | 1.891 | 1.251 | 4.486 | 0.995 | 4.486 |
| list_pending | 0.363 | 0.169 | 1.212 | 0.111 | 1.212 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.098 |
| Telemetry | 2 | 2.518 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.212 ms | **Min:** 0.004 ms | **Max:** 4.486 ms

## P95 Gate

**Global P95:** 1.212 ms
**Threshold:** 50.0 ms
**Status:** PASS
