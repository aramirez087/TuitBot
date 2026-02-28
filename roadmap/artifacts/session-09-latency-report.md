# Session 09 — Latency Report

**Generated:** 2026-02-28 17:21 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.032 | 0.033 | 0.043 | 0.016 | 0.043 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.008 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.019 | 0.013 | 0.048 | 0.011 | 0.048 |
| get_config | 0.088 | 0.086 | 0.102 | 0.083 | 0.102 |
| validate_config | 0.018 | 0.011 | 0.046 | 0.010 | 0.046 |
| get_mcp_tool_metrics | 1.017 | 0.519 | 3.058 | 0.440 | 3.058 |
| get_mcp_error_breakdown | 0.315 | 0.173 | 0.952 | 0.106 | 0.952 |
| get_capabilities | 0.902 | 0.830 | 1.167 | 0.748 | 1.167 |
| health_check | 0.284 | 0.161 | 0.773 | 0.146 | 0.773 |
| get_stats | 1.641 | 1.279 | 3.233 | 1.124 | 3.233 |
| list_pending | 0.459 | 0.186 | 1.648 | 0.116 | 1.648 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.043 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.102 |
| Telemetry | 2 | 3.058 |

## Aggregate

**P50:** 0.033 ms | **P95:** 1.279 ms | **Min:** 0.003 ms | **Max:** 3.233 ms

## P95 Gate

**Global P95:** 1.279 ms
**Threshold:** 50.0 ms
**Status:** PASS
