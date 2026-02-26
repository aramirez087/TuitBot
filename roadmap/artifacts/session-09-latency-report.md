# Session 09 — Latency Report

**Generated:** 2026-02-26 18:21 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.016 | 0.011 | 0.016 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.007 | 0.006 | 0.007 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.013 | 0.018 | 0.012 | 0.018 |
| get_config | 0.085 | 0.083 | 0.093 | 0.081 | 0.093 |
| validate_config | 0.019 | 0.011 | 0.051 | 0.010 | 0.051 |
| get_mcp_tool_metrics | 1.056 | 0.683 | 2.680 | 0.434 | 2.680 |
| get_mcp_error_breakdown | 0.304 | 0.262 | 0.627 | 0.094 | 0.627 |
| get_capabilities | 1.102 | 0.918 | 1.725 | 0.782 | 1.725 |
| health_check | 0.317 | 0.230 | 0.770 | 0.165 | 0.770 |
| get_stats | 1.771 | 1.513 | 3.168 | 1.201 | 3.168 |
| list_pending | 0.321 | 0.136 | 1.031 | 0.064 | 1.031 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.093 |
| Telemetry | 2 | 2.680 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.513 ms | **Min:** 0.003 ms | **Max:** 3.168 ms

## P95 Gate

**Global P95:** 1.513 ms
**Threshold:** 50.0 ms
**Status:** PASS
