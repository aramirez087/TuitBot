# Session 09 — Latency Report

**Generated:** 2026-03-01 23:12 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.018 | 0.013 | 0.034 | 0.011 | 0.034 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.007 | 0.006 | 0.010 | 0.006 | 0.010 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.011 | 0.008 | 0.018 | 0.008 | 0.018 |
| kernel::post_tweet | 0.004 | 0.004 | 0.007 | 0.003 | 0.007 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.028 | 0.027 | 0.038 | 0.023 | 0.038 |
| get_config | 0.169 | 0.165 | 0.184 | 0.163 | 0.184 |
| validate_config | 0.096 | 0.012 | 0.409 | 0.009 | 0.409 |
| get_mcp_tool_metrics | 2.547 | 1.897 | 5.672 | 0.660 | 5.672 |
| get_mcp_error_breakdown | 0.357 | 0.233 | 0.886 | 0.122 | 0.886 |
| get_capabilities | 1.332 | 1.495 | 1.675 | 0.941 | 1.675 |
| health_check | 0.286 | 0.211 | 0.662 | 0.140 | 0.662 |
| get_stats | 1.547 | 1.078 | 3.210 | 1.021 | 3.210 |
| list_pending | 0.363 | 0.117 | 1.337 | 0.098 | 1.337 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.021 |
| Kernel write | 2 | 0.007 |
| Config | 3 | 0.409 |
| Telemetry | 2 | 5.672 |

## Aggregate

**P50:** 0.029 ms | **P95:** 1.675 ms | **Min:** 0.003 ms | **Max:** 5.672 ms

## P95 Gate

**Global P95:** 1.675 ms
**Threshold:** 50.0 ms
**Status:** PASS
