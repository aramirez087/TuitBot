# Session 09 — Latency Report

**Generated:** 2026-03-04 01:50 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.012 | 0.021 | 0.012 | 0.021 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::post_tweet | 0.004 | 0.004 | 0.007 | 0.004 | 0.007 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.017 | 0.012 | 0.036 | 0.012 | 0.036 |
| get_config | 0.090 | 0.087 | 0.103 | 0.086 | 0.103 |
| validate_config | 0.021 | 0.012 | 0.056 | 0.012 | 0.056 |
| get_mcp_tool_metrics | 1.196 | 0.665 | 3.448 | 0.505 | 3.448 |
| get_mcp_error_breakdown | 0.259 | 0.142 | 0.779 | 0.086 | 0.779 |
| get_capabilities | 1.057 | 0.958 | 1.475 | 0.796 | 1.475 |
| health_check | 0.374 | 0.317 | 0.745 | 0.234 | 0.745 |
| get_stats | 2.524 | 2.028 | 3.528 | 1.682 | 3.528 |
| list_pending | 0.814 | 0.220 | 3.088 | 0.183 | 3.088 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.007 |
| Config | 3 | 0.103 |
| Telemetry | 2 | 3.448 |

## Aggregate

**P50:** 0.013 ms | **P95:** 2.028 ms | **Min:** 0.004 ms | **Max:** 3.528 ms

## P95 Gate

**Global P95:** 2.028 ms
**Threshold:** 50.0 ms
**Status:** PASS
