# Session 09 — Latency Report

**Generated:** 2026-02-27 03:44 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.012 | 0.025 | 0.011 | 0.025 |
| kernel::search_tweets | 0.008 | 0.007 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.026 | 0.012 | 0.026 |
| get_config | 0.083 | 0.081 | 0.093 | 0.080 | 0.093 |
| validate_config | 0.019 | 0.010 | 0.054 | 0.010 | 0.054 |
| get_mcp_tool_metrics | 1.123 | 0.526 | 3.446 | 0.494 | 3.446 |
| get_mcp_error_breakdown | 0.236 | 0.123 | 0.704 | 0.105 | 0.704 |
| get_capabilities | 1.023 | 1.056 | 1.194 | 0.679 | 1.194 |
| health_check | 0.315 | 0.216 | 0.757 | 0.070 | 0.757 |
| get_stats | 1.870 | 1.714 | 3.026 | 1.340 | 3.026 |
| list_pending | 0.463 | 0.170 | 1.451 | 0.147 | 1.451 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.093 |
| Telemetry | 2 | 3.446 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.451 ms | **Min:** 0.003 ms | **Max:** 3.446 ms

## P95 Gate

**Global P95:** 1.451 ms
**Threshold:** 50.0 ms
**Status:** PASS
