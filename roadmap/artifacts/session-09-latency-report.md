# Session 09 — Latency Report

**Generated:** 2026-02-26 23:47 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.022 | 0.022 | 0.033 | 0.014 | 0.033 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.007 | 0.006 | 0.007 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.014 | 0.020 | 0.012 | 0.020 |
| get_config | 0.085 | 0.082 | 0.094 | 0.080 | 0.094 |
| validate_config | 0.020 | 0.010 | 0.060 | 0.010 | 0.060 |
| get_mcp_tool_metrics | 1.124 | 0.889 | 2.653 | 0.548 | 2.653 |
| get_mcp_error_breakdown | 0.299 | 0.159 | 0.938 | 0.088 | 0.938 |
| get_capabilities | 0.789 | 0.681 | 1.351 | 0.536 | 1.351 |
| health_check | 0.261 | 0.188 | 0.544 | 0.113 | 0.544 |
| get_stats | 1.646 | 1.317 | 3.511 | 0.983 | 3.511 |
| list_pending | 0.423 | 0.128 | 1.641 | 0.068 | 1.641 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.023 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.094 |
| Telemetry | 2 | 2.653 |

## Aggregate

**P50:** 0.021 ms | **P95:** 1.351 ms | **Min:** 0.003 ms | **Max:** 3.511 ms

## P95 Gate

**Global P95:** 1.351 ms
**Threshold:** 50.0 ms
**Status:** PASS
