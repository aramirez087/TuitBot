# Session 09 — Latency Report

**Generated:** 2026-02-27 01:03 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.011 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.022 | 0.012 | 0.022 |
| get_config | 0.088 | 0.088 | 0.096 | 0.080 | 0.096 |
| validate_config | 0.013 | 0.011 | 0.023 | 0.010 | 0.023 |
| get_mcp_tool_metrics | 0.360 | 0.252 | 0.819 | 0.228 | 0.819 |
| get_mcp_error_breakdown | 0.106 | 0.069 | 0.235 | 0.060 | 0.235 |
| get_capabilities | 0.473 | 0.450 | 0.599 | 0.401 | 0.599 |
| health_check | 0.108 | 0.094 | 0.186 | 0.076 | 0.186 |
| get_stats | 0.654 | 0.549 | 1.051 | 0.481 | 1.051 |
| list_pending | 0.139 | 0.068 | 0.447 | 0.053 | 0.447 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.096 |
| Telemetry | 2 | 0.819 |

## Aggregate

**P50:** 0.013 ms | **P95:** 0.549 ms | **Min:** 0.004 ms | **Max:** 1.051 ms

## P95 Gate

**Global P95:** 0.549 ms
**Threshold:** 50.0 ms
**Status:** PASS
