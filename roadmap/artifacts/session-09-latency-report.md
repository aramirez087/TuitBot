# Session 09 — Latency Report

**Generated:** 2026-02-27 03:15 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.030 | 0.011 | 0.030 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.032 | 0.012 | 0.032 |
| get_config | 0.124 | 0.096 | 0.190 | 0.083 | 0.190 |
| validate_config | 0.030 | 0.021 | 0.070 | 0.019 | 0.070 |
| get_mcp_tool_metrics | 1.135 | 0.676 | 3.014 | 0.647 | 3.014 |
| get_mcp_error_breakdown | 0.321 | 0.170 | 0.966 | 0.141 | 0.966 |
| get_capabilities | 1.068 | 1.003 | 1.646 | 0.792 | 1.646 |
| health_check | 0.363 | 0.307 | 0.704 | 0.214 | 0.704 |
| get_stats | 2.490 | 1.881 | 5.155 | 1.377 | 5.155 |
| list_pending | 0.443 | 0.206 | 1.495 | 0.112 | 1.495 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.190 |
| Telemetry | 2 | 3.014 |

## Aggregate

**P50:** 0.021 ms | **P95:** 1.646 ms | **Min:** 0.003 ms | **Max:** 5.155 ms

## P95 Gate

**Global P95:** 1.646 ms
**Threshold:** 50.0 ms
**Status:** PASS
