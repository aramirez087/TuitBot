# Session 09 — Latency Report

**Generated:** 2026-02-26 19:33 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.013 | 0.017 | 0.012 | 0.017 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.008 | 0.012 |
| kernel::get_followers | 0.007 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.008 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.020 | 0.013 | 0.049 | 0.013 | 0.049 |
| get_config | 0.086 | 0.084 | 0.093 | 0.083 | 0.093 |
| validate_config | 0.020 | 0.011 | 0.054 | 0.010 | 0.054 |
| get_mcp_tool_metrics | 0.200 | 0.176 | 0.304 | 0.166 | 0.304 |
| get_mcp_error_breakdown | 0.056 | 0.052 | 0.076 | 0.049 | 0.076 |
| get_capabilities | 0.350 | 0.336 | 0.389 | 0.331 | 0.389 |
| health_check | 0.059 | 0.053 | 0.079 | 0.052 | 0.079 |
| get_stats | 0.357 | 0.340 | 0.445 | 0.324 | 0.445 |
| list_pending | 0.060 | 0.049 | 0.102 | 0.047 | 0.102 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.093 |
| Telemetry | 2 | 0.304 |

## Aggregate

**P50:** 0.013 ms | **P95:** 0.340 ms | **Min:** 0.003 ms | **Max:** 0.445 ms

## P95 Gate

**Global P95:** 0.340 ms
**Threshold:** 50.0 ms
**Status:** PASS
