# Session 09 — Latency Report

**Generated:** 2026-02-27 02:21 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.013 | 0.036 | 0.012 | 0.036 |
| kernel::search_tweets | 0.011 | 0.010 | 0.017 | 0.008 | 0.017 |
| kernel::get_followers | 0.008 | 0.008 | 0.014 | 0.006 | 0.014 |
| kernel::get_user_by_id | 0.014 | 0.009 | 0.022 | 0.008 | 0.022 |
| kernel::get_me | 0.014 | 0.014 | 0.015 | 0.014 | 0.015 |
| kernel::post_tweet | 0.008 | 0.007 | 0.009 | 0.006 | 0.009 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.018 | 0.013 | 0.040 | 0.012 | 0.040 |
| get_config | 0.091 | 0.084 | 0.110 | 0.082 | 0.110 |
| validate_config | 0.022 | 0.013 | 0.063 | 0.011 | 0.063 |
| get_mcp_tool_metrics | 1.210 | 0.727 | 3.285 | 0.569 | 3.285 |
| get_mcp_error_breakdown | 0.394 | 0.211 | 1.009 | 0.112 | 1.009 |
| get_capabilities | 1.370 | 1.279 | 1.970 | 0.944 | 1.970 |
| health_check | 0.431 | 0.506 | 0.655 | 0.144 | 0.655 |
| get_stats | 2.318 | 1.649 | 4.843 | 1.538 | 4.843 |
| list_pending | 0.665 | 0.175 | 2.619 | 0.153 | 2.619 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.022 |
| Kernel write | 2 | 0.009 |
| Config | 3 | 0.110 |
| Telemetry | 2 | 3.285 |

## Aggregate

**P50:** 0.022 ms | **P95:** 1.970 ms | **Min:** 0.004 ms | **Max:** 4.843 ms

## P95 Gate

**Global P95:** 1.970 ms
**Threshold:** 50.0 ms
**Status:** PASS
