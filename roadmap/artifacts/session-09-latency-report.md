# Session 09 — Latency Report

**Generated:** 2026-03-01 05:01 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.027 | 0.011 | 0.027 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.008 | 0.004 | 0.026 | 0.004 | 0.026 |
| score_tweet | 0.028 | 0.027 | 0.040 | 0.013 | 0.040 |
| get_config | 0.119 | 0.096 | 0.208 | 0.094 | 0.208 |
| validate_config | 0.071 | 0.010 | 0.313 | 0.009 | 0.313 |
| get_mcp_tool_metrics | 1.102 | 0.748 | 2.860 | 0.532 | 2.860 |
| get_mcp_error_breakdown | 0.220 | 0.159 | 0.542 | 0.086 | 0.542 |
| get_capabilities | 1.033 | 0.971 | 1.482 | 0.722 | 1.482 |
| health_check | 0.347 | 0.226 | 0.851 | 0.161 | 0.851 |
| get_stats | 2.869 | 2.154 | 4.394 | 1.462 | 4.394 |
| list_pending | 0.581 | 0.141 | 2.255 | 0.101 | 2.255 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.026 |
| Config | 3 | 0.313 |
| Telemetry | 2 | 2.860 |

## Aggregate

**P50:** 0.027 ms | **P95:** 2.154 ms | **Min:** 0.003 ms | **Max:** 4.394 ms

## P95 Gate

**Global P95:** 2.154 ms
**Threshold:** 50.0 ms
**Status:** PASS
