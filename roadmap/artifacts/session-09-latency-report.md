# Session 09 — Latency Report

**Generated:** 2026-03-01 04:40 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.011 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.012 | 0.034 | 0.012 | 0.034 |
| get_config | 0.106 | 0.092 | 0.166 | 0.089 | 0.166 |
| validate_config | 0.098 | 0.012 | 0.443 | 0.011 | 0.443 |
| get_mcp_tool_metrics | 1.140 | 0.917 | 2.464 | 0.556 | 2.464 |
| get_mcp_error_breakdown | 0.387 | 0.213 | 0.752 | 0.198 | 0.752 |
| get_capabilities | 0.947 | 0.875 | 1.358 | 0.673 | 1.358 |
| health_check | 0.285 | 0.210 | 0.682 | 0.133 | 0.682 |
| get_stats | 1.542 | 1.191 | 2.499 | 1.157 | 2.499 |
| list_pending | 0.352 | 0.096 | 1.403 | 0.066 | 1.403 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.443 |
| Telemetry | 2 | 2.464 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.358 ms | **Min:** 0.003 ms | **Max:** 2.499 ms

## P95 Gate

**Global P95:** 1.358 ms
**Threshold:** 50.0 ms
**Status:** PASS
