# Session 09 — Latency Report

**Generated:** 2026-02-26 17:16 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.024 | 0.024 | 0.028 | 0.021 | 0.028 |
| kernel::search_tweets | 0.016 | 0.016 | 0.018 | 0.014 | 0.018 |
| kernel::get_followers | 0.011 | 0.011 | 0.017 | 0.007 | 0.017 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.007 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.057 | 0.014 | 0.227 | 0.014 | 0.227 |
| get_config | 0.085 | 0.083 | 0.095 | 0.079 | 0.095 |
| validate_config | 0.075 | 0.011 | 0.333 | 0.010 | 0.333 |
| get_mcp_tool_metrics | 1.085 | 0.697 | 2.925 | 0.478 | 2.925 |
| get_mcp_error_breakdown | 0.245 | 0.202 | 0.572 | 0.099 | 0.572 |
| get_capabilities | 0.863 | 0.784 | 1.210 | 0.719 | 1.210 |
| health_check | 0.363 | 0.251 | 0.850 | 0.200 | 0.850 |
| get_stats | 1.771 | 1.377 | 3.392 | 1.340 | 3.392 |
| list_pending | 0.397 | 0.232 | 1.159 | 0.162 | 1.159 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.024 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.333 |
| Telemetry | 2 | 2.925 |

## Aggregate

**P50:** 0.024 ms | **P95:** 1.367 ms | **Min:** 0.003 ms | **Max:** 3.392 ms

## P95 Gate

**Global P95:** 1.367 ms
**Threshold:** 50.0 ms
**Status:** PASS
