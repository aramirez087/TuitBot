# Session 09 — Latency Report

**Generated:** 2026-03-01 20:26 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.021 | 0.015 | 0.035 | 0.011 | 0.035 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.013 | 0.020 | 0.012 | 0.020 |
| get_config | 0.090 | 0.088 | 0.099 | 0.087 | 0.099 |
| validate_config | 0.124 | 0.014 | 0.566 | 0.010 | 0.566 |
| get_mcp_tool_metrics | 1.140 | 0.664 | 3.138 | 0.528 | 3.138 |
| get_mcp_error_breakdown | 0.239 | 0.172 | 0.627 | 0.092 | 0.627 |
| get_capabilities | 0.906 | 0.766 | 1.485 | 0.604 | 1.485 |
| health_check | 0.315 | 0.381 | 0.500 | 0.094 | 0.500 |
| get_stats | 1.546 | 1.170 | 3.157 | 1.083 | 3.157 |
| list_pending | 0.397 | 0.165 | 1.355 | 0.141 | 1.355 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.035 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.566 |
| Telemetry | 2 | 3.138 |

## Aggregate

**P50:** 0.020 ms | **P95:** 1.208 ms | **Min:** 0.003 ms | **Max:** 3.157 ms

## P95 Gate

**Global P95:** 1.208 ms
**Threshold:** 50.0 ms
**Status:** PASS
