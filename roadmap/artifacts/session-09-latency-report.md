# Session 09 — Latency Report

**Generated:** 2026-03-06 04:53 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.016 | 0.012 | 0.032 | 0.011 | 0.032 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.009 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.012 | 0.029 | 0.012 | 0.029 |
| get_config | 0.090 | 0.087 | 0.103 | 0.086 | 0.103 |
| validate_config | 0.015 | 0.013 | 0.027 | 0.012 | 0.027 |
| get_mcp_tool_metrics | 1.012 | 0.699 | 2.720 | 0.384 | 2.720 |
| get_mcp_error_breakdown | 0.226 | 0.151 | 0.542 | 0.076 | 0.542 |
| get_capabilities | 0.990 | 0.898 | 1.357 | 0.576 | 1.357 |
| health_check | 0.410 | 0.296 | 0.833 | 0.173 | 0.833 |
| get_stats | 1.857 | 1.519 | 3.801 | 1.144 | 3.801 |
| list_pending | 0.361 | 0.178 | 1.188 | 0.090 | 1.188 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.103 |
| Telemetry | 2 | 2.720 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.357 ms | **Min:** 0.003 ms | **Max:** 3.801 ms

## P95 Gate

**Global P95:** 1.357 ms
**Threshold:** 50.0 ms
**Status:** PASS
