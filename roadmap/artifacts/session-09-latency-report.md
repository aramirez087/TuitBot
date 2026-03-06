# Session 09 — Latency Report

**Generated:** 2026-03-06 05:01 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.016 | 0.012 | 0.030 | 0.011 | 0.030 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.003 | 0.005 |
| score_tweet | 0.016 | 0.012 | 0.031 | 0.012 | 0.031 |
| get_config | 0.111 | 0.095 | 0.183 | 0.088 | 0.183 |
| validate_config | 0.015 | 0.012 | 0.027 | 0.011 | 0.027 |
| get_mcp_tool_metrics | 0.898 | 0.496 | 2.502 | 0.430 | 2.502 |
| get_mcp_error_breakdown | 0.222 | 0.141 | 0.641 | 0.072 | 0.641 |
| get_capabilities | 0.913 | 0.862 | 1.285 | 0.699 | 1.285 |
| health_check | 0.234 | 0.201 | 0.398 | 0.174 | 0.398 |
| get_stats | 1.227 | 1.004 | 2.251 | 0.856 | 2.251 |
| list_pending | 0.560 | 0.215 | 1.780 | 0.127 | 1.780 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.183 |
| Telemetry | 2 | 2.502 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.075 ms | **Min:** 0.003 ms | **Max:** 2.502 ms

## P95 Gate

**Global P95:** 1.075 ms
**Threshold:** 50.0 ms
**Status:** PASS
