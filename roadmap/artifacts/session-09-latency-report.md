# Session 09 — Latency Report

**Generated:** 2026-03-01 01:16 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.030 | 0.011 | 0.030 |
| kernel::search_tweets | 0.009 | 0.007 | 0.014 | 0.007 | 0.014 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.009 | 0.007 | 0.013 | 0.007 | 0.013 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.020 | 0.012 | 0.050 | 0.012 | 0.050 |
| get_config | 0.097 | 0.095 | 0.115 | 0.084 | 0.115 |
| validate_config | 0.078 | 0.010 | 0.349 | 0.010 | 0.349 |
| get_mcp_tool_metrics | 1.286 | 0.841 | 3.183 | 0.555 | 3.183 |
| get_mcp_error_breakdown | 0.241 | 0.167 | 0.512 | 0.134 | 0.512 |
| get_capabilities | 0.929 | 0.965 | 1.252 | 0.626 | 1.252 |
| health_check | 0.362 | 0.238 | 1.048 | 0.132 | 1.048 |
| get_stats | 1.761 | 1.249 | 3.418 | 1.064 | 3.418 |
| list_pending | 0.427 | 0.160 | 1.513 | 0.139 | 1.513 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.014 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.349 |
| Telemetry | 2 | 3.183 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.252 ms | **Min:** 0.003 ms | **Max:** 3.418 ms

## P95 Gate

**Global P95:** 1.252 ms
**Threshold:** 50.0 ms
**Status:** PASS
