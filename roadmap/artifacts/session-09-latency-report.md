# Session 09 — Latency Report

**Generated:** 2026-02-27 03:27 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.011 | 0.027 | 0.011 | 0.027 |
| kernel::search_tweets | 0.008 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.005 | 0.007 | 0.005 | 0.007 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.012 | 0.028 | 0.011 | 0.028 |
| get_config | 0.084 | 0.083 | 0.092 | 0.079 | 0.092 |
| validate_config | 0.019 | 0.010 | 0.056 | 0.010 | 0.056 |
| get_mcp_tool_metrics | 1.068 | 0.763 | 2.457 | 0.477 | 2.457 |
| get_mcp_error_breakdown | 0.253 | 0.135 | 0.739 | 0.087 | 0.739 |
| get_capabilities | 0.962 | 0.809 | 1.625 | 0.636 | 1.625 |
| health_check | 0.231 | 0.183 | 0.472 | 0.126 | 0.472 |
| get_stats | 1.810 | 1.503 | 3.085 | 1.148 | 3.085 |
| list_pending | 0.444 | 0.186 | 1.577 | 0.121 | 1.577 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.092 |
| Telemetry | 2 | 2.457 |

## Aggregate

**P50:** 0.012 ms | **P95:** 1.577 ms | **Min:** 0.003 ms | **Max:** 3.085 ms

## P95 Gate

**Global P95:** 1.577 ms
**Threshold:** 50.0 ms
**Status:** PASS
