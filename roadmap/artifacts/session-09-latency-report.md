# Session 09 — Latency Report

**Generated:** 2026-03-06 01:50 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.033 | 0.023 | 0.073 | 0.022 | 0.073 |
| kernel::search_tweets | 0.030 | 0.021 | 0.059 | 0.015 | 0.059 |
| kernel::get_followers | 0.020 | 0.015 | 0.038 | 0.014 | 0.038 |
| kernel::get_user_by_id | 0.010 | 0.009 | 0.016 | 0.008 | 0.016 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.007 | 0.004 | 0.007 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.018 | 0.013 | 0.041 | 0.012 | 0.041 |
| get_config | 0.090 | 0.087 | 0.104 | 0.084 | 0.104 |
| validate_config | 0.452 | 0.012 | 2.211 | 0.011 | 2.211 |
| get_mcp_tool_metrics | 1.195 | 0.742 | 3.084 | 0.660 | 3.084 |
| get_mcp_error_breakdown | 0.312 | 0.204 | 0.786 | 0.134 | 0.786 |
| get_capabilities | 1.005 | 0.845 | 1.735 | 0.667 | 1.735 |
| health_check | 0.290 | 0.190 | 0.724 | 0.105 | 0.724 |
| get_stats | 1.521 | 1.390 | 2.548 | 1.023 | 2.548 |
| list_pending | 0.328 | 0.194 | 0.930 | 0.133 | 0.930 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.059 |
| Kernel write | 2 | 0.007 |
| Config | 3 | 2.211 |
| Telemetry | 2 | 3.084 |

## Aggregate

**P50:** 0.038 ms | **P95:** 1.398 ms | **Min:** 0.003 ms | **Max:** 3.084 ms

## P95 Gate

**Global P95:** 1.398 ms
**Threshold:** 50.0 ms
**Status:** PASS
