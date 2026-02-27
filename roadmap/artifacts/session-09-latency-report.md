# Session 09 — Latency Report

**Generated:** 2026-02-27 23:24 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.018 | 0.012 | 0.043 | 0.012 | 0.043 |
| kernel::search_tweets | 0.011 | 0.008 | 0.018 | 0.008 | 0.018 |
| kernel::get_followers | 0.007 | 0.006 | 0.011 | 0.005 | 0.011 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.019 | 0.017 | 0.030 | 0.012 | 0.030 |
| get_config | 0.084 | 0.081 | 0.096 | 0.080 | 0.096 |
| validate_config | 0.020 | 0.010 | 0.058 | 0.010 | 0.058 |
| get_mcp_tool_metrics | 1.124 | 0.680 | 3.012 | 0.587 | 3.012 |
| get_mcp_error_breakdown | 0.464 | 0.259 | 1.284 | 0.213 | 1.284 |
| get_capabilities | 2.354 | 2.221 | 3.160 | 1.120 | 3.160 |
| health_check | 0.600 | 0.518 | 1.055 | 0.432 | 1.055 |
| get_stats | 2.451 | 2.150 | 4.409 | 1.654 | 4.409 |
| list_pending | 1.004 | 0.286 | 3.931 | 0.183 | 3.931 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.018 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.096 |
| Telemetry | 2 | 3.012 |

## Aggregate

**P50:** 0.018 ms | **P95:** 3.012 ms | **Min:** 0.003 ms | **Max:** 4.409 ms

## P95 Gate

**Global P95:** 3.012 ms
**Threshold:** 50.0 ms
**Status:** PASS
