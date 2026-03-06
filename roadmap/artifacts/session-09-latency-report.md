# Session 09 — Latency Report

**Generated:** 2026-03-06 04:42 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.012 | 0.037 | 0.011 | 0.037 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.017 | 0.012 | 0.038 | 0.012 | 0.038 |
| get_config | 0.092 | 0.089 | 0.102 | 0.087 | 0.102 |
| validate_config | 0.076 | 0.013 | 0.327 | 0.013 | 0.327 |
| get_mcp_tool_metrics | 1.197 | 0.701 | 3.051 | 0.664 | 3.051 |
| get_mcp_error_breakdown | 0.312 | 0.176 | 0.881 | 0.118 | 0.881 |
| get_capabilities | 0.972 | 0.922 | 1.426 | 0.741 | 1.426 |
| health_check | 0.240 | 0.218 | 0.364 | 0.164 | 0.364 |
| get_stats | 1.652 | 1.256 | 3.266 | 0.853 | 3.266 |
| list_pending | 0.360 | 0.154 | 1.275 | 0.079 | 1.275 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.327 |
| Telemetry | 2 | 3.051 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.275 ms | **Min:** 0.003 ms | **Max:** 3.266 ms

## P95 Gate

**Global P95:** 1.275 ms
**Threshold:** 50.0 ms
**Status:** PASS
