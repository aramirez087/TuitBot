# Session 09 — Latency Report

**Generated:** 2026-03-07 17:45 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.030 | 0.012 | 0.030 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.009 | 0.009 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.030 | 0.012 | 0.030 |
| get_config | 0.091 | 0.089 | 0.103 | 0.087 | 0.103 |
| validate_config | 0.015 | 0.012 | 0.027 | 0.011 | 0.027 |
| get_mcp_tool_metrics | 1.239 | 0.794 | 3.236 | 0.592 | 3.236 |
| get_mcp_error_breakdown | 0.283 | 0.161 | 0.793 | 0.135 | 0.793 |
| get_capabilities | 1.155 | 1.210 | 1.458 | 0.895 | 1.458 |
| health_check | 0.303 | 0.238 | 0.681 | 0.120 | 0.681 |
| get_stats | 1.715 | 1.517 | 3.038 | 1.124 | 3.038 |
| list_pending | 0.495 | 0.199 | 1.763 | 0.119 | 1.763 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.103 |
| Telemetry | 2 | 3.236 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.517 ms | **Min:** 0.003 ms | **Max:** 3.236 ms

## P95 Gate

**Global P95:** 1.517 ms
**Threshold:** 50.0 ms
**Status:** PASS
