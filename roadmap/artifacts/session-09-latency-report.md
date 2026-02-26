# Session 09 — Latency Report

**Generated:** 2026-02-26 19:07 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.027 | 0.011 | 0.027 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.008 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.008 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.014 | 0.013 | 0.020 | 0.012 | 0.020 |
| get_config | 0.085 | 0.084 | 0.092 | 0.082 | 0.092 |
| validate_config | 0.017 | 0.011 | 0.044 | 0.009 | 0.044 |
| get_mcp_tool_metrics | 1.018 | 0.561 | 2.657 | 0.450 | 2.657 |
| get_mcp_error_breakdown | 0.322 | 0.133 | 0.827 | 0.102 | 0.827 |
| get_capabilities | 0.896 | 0.694 | 1.445 | 0.647 | 1.445 |
| health_check | 0.286 | 0.262 | 0.474 | 0.172 | 0.474 |
| get_stats | 1.720 | 1.253 | 3.771 | 1.123 | 3.771 |
| list_pending | 0.353 | 0.120 | 1.239 | 0.110 | 1.239 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.092 |
| Telemetry | 2 | 2.657 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.253 ms | **Min:** 0.004 ms | **Max:** 3.771 ms

## P95 Gate

**Global P95:** 1.253 ms
**Threshold:** 50.0 ms
**Status:** PASS
