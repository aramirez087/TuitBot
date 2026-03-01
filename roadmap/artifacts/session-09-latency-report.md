# Session 09 — Latency Report

**Generated:** 2026-03-01 04:31 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.032 | 0.031 | 0.044 | 0.022 | 0.044 |
| kernel::search_tweets | 0.008 | 0.007 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.012 | 0.008 | 0.026 | 0.007 | 0.026 |
| kernel::get_me | 0.015 | 0.014 | 0.017 | 0.013 | 0.017 |
| kernel::post_tweet | 0.009 | 0.008 | 0.010 | 0.007 | 0.010 |
| kernel::reply_to_tweet | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| score_tweet | 0.030 | 0.024 | 0.057 | 0.020 | 0.057 |
| get_config | 0.093 | 0.089 | 0.110 | 0.086 | 0.110 |
| validate_config | 0.074 | 0.012 | 0.318 | 0.012 | 0.318 |
| get_mcp_tool_metrics | 1.045 | 0.597 | 2.718 | 0.459 | 2.718 |
| get_mcp_error_breakdown | 0.279 | 0.154 | 0.647 | 0.132 | 0.647 |
| get_capabilities | 1.051 | 1.103 | 1.378 | 0.697 | 1.378 |
| health_check | 0.414 | 0.340 | 0.848 | 0.194 | 0.848 |
| get_stats | 1.527 | 1.229 | 2.974 | 0.970 | 2.974 |
| list_pending | 0.379 | 0.114 | 1.448 | 0.096 | 1.448 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.038 |
| Kernel write | 2 | 0.010 |
| Config | 3 | 0.318 |
| Telemetry | 2 | 2.718 |

## Aggregate

**P50:** 0.031 ms | **P95:** 1.274 ms | **Min:** 0.006 ms | **Max:** 2.974 ms

## P95 Gate

**Global P95:** 1.274 ms
**Threshold:** 50.0 ms
**Status:** PASS
