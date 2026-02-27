# Session 09 — Latency Report

**Generated:** 2026-02-27 19:16 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.026 | 0.022 | 0.042 | 0.021 | 0.042 |
| kernel::search_tweets | 0.016 | 0.011 | 0.027 | 0.008 | 0.027 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.010 | 0.007 | 0.010 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.018 | 0.012 | 0.041 | 0.012 | 0.041 |
| get_config | 0.083 | 0.081 | 0.093 | 0.079 | 0.093 |
| validate_config | 0.532 | 0.010 | 2.621 | 0.010 | 2.621 |
| get_mcp_tool_metrics | 1.078 | 0.580 | 3.031 | 0.464 | 3.031 |
| get_mcp_error_breakdown | 0.311 | 0.208 | 0.683 | 0.163 | 0.683 |
| get_capabilities | 0.906 | 0.701 | 1.507 | 0.686 | 1.507 |
| health_check | 0.341 | 0.232 | 0.609 | 0.120 | 0.609 |
| get_stats | 1.550 | 1.376 | 2.579 | 1.144 | 2.579 |
| list_pending | 0.354 | 0.138 | 1.297 | 0.082 | 1.297 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.027 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 2.621 |
| Telemetry | 2 | 3.031 |

## Aggregate

**P50:** 0.025 ms | **P95:** 1.455 ms | **Min:** 0.003 ms | **Max:** 3.031 ms

## P95 Gate

**Global P95:** 1.455 ms
**Threshold:** 50.0 ms
**Status:** PASS
