# Session 09 — Latency Report

**Generated:** 2026-03-02 01:39 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.011 | 0.025 | 0.011 | 0.025 |
| kernel::search_tweets | 0.008 | 0.007 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.012 | 0.027 | 0.012 | 0.027 |
| get_config | 0.093 | 0.090 | 0.102 | 0.087 | 0.102 |
| validate_config | 0.014 | 0.011 | 0.027 | 0.010 | 0.027 |
| get_mcp_tool_metrics | 1.104 | 0.632 | 3.333 | 0.432 | 3.333 |
| get_mcp_error_breakdown | 0.251 | 0.146 | 0.631 | 0.135 | 0.631 |
| get_capabilities | 0.968 | 0.829 | 1.719 | 0.669 | 1.719 |
| health_check | 0.300 | 0.261 | 0.679 | 0.103 | 0.679 |
| get_stats | 1.513 | 1.052 | 3.244 | 0.926 | 3.244 |
| list_pending | 0.380 | 0.124 | 1.455 | 0.075 | 1.455 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.102 |
| Telemetry | 2 | 3.333 |

## Aggregate

**P50:** 0.012 ms | **P95:** 1.347 ms | **Min:** 0.003 ms | **Max:** 3.333 ms

## P95 Gate

**Global P95:** 1.347 ms
**Threshold:** 50.0 ms
**Status:** PASS
