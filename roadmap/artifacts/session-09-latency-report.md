# Session 09 — Latency Report

**Generated:** 2026-02-26 04:03 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.031 | 0.022 | 0.065 | 0.021 | 0.065 |
| kernel::search_tweets | 0.018 | 0.014 | 0.032 | 0.014 | 0.032 |
| kernel::get_followers | 0.012 | 0.011 | 0.013 | 0.011 | 0.013 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.010 | 0.007 | 0.010 |
| kernel::get_me | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::post_tweet | 0.004 | 0.003 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.019 | 0.013 | 0.045 | 0.012 | 0.045 |
| get_config | 0.083 | 0.080 | 0.094 | 0.079 | 0.094 |
| validate_config | 0.020 | 0.011 | 0.056 | 0.010 | 0.056 |
| get_mcp_tool_metrics | 1.438 | 0.722 | 4.363 | 0.671 | 4.363 |
| get_mcp_error_breakdown | 0.285 | 0.210 | 0.560 | 0.121 | 0.560 |
| get_capabilities | 0.949 | 0.904 | 1.220 | 0.733 | 1.220 |
| health_check | 0.369 | 0.202 | 0.974 | 0.173 | 0.974 |
| get_stats | 1.855 | 1.651 | 3.292 | 1.285 | 3.292 |
| list_pending | 0.481 | 0.251 | 1.502 | 0.163 | 1.502 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.032 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.094 |
| Telemetry | 2 | 4.363 |

## Aggregate

**P50:** 0.023 ms | **P95:** 1.502 ms | **Min:** 0.003 ms | **Max:** 4.363 ms

## P95 Gate

**Global P95:** 1.502 ms
**Threshold:** 50.0 ms
**Status:** PASS
