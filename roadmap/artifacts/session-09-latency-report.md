# Session 09 — Latency Report

**Generated:** 2026-03-02 01:28 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.026 | 0.024 | 0.049 | 0.012 | 0.049 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.013 | 0.020 | 0.012 | 0.020 |
| get_config | 0.089 | 0.086 | 0.098 | 0.085 | 0.098 |
| validate_config | 0.013 | 0.010 | 0.023 | 0.010 | 0.023 |
| get_mcp_tool_metrics | 1.015 | 0.545 | 2.838 | 0.513 | 2.838 |
| get_mcp_error_breakdown | 0.256 | 0.179 | 0.717 | 0.090 | 0.717 |
| get_capabilities | 1.085 | 1.000 | 1.357 | 0.789 | 1.357 |
| health_check | 0.370 | 0.345 | 0.527 | 0.241 | 0.527 |
| get_stats | 1.670 | 1.220 | 3.161 | 1.102 | 3.161 |
| list_pending | 0.487 | 0.458 | 1.083 | 0.155 | 1.083 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.033 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.098 |
| Telemetry | 2 | 2.838 |

## Aggregate

**P50:** 0.020 ms | **P95:** 1.336 ms | **Min:** 0.003 ms | **Max:** 3.161 ms

## P95 Gate

**Global P95:** 1.336 ms
**Threshold:** 50.0 ms
**Status:** PASS
