# Session 09 — Latency Report

**Generated:** 2026-03-01 02:14 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.012 | 0.011 | 0.015 | 0.011 | 0.015 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.012 | 0.020 | 0.012 | 0.020 |
| get_config | 0.094 | 0.094 | 0.106 | 0.085 | 0.106 |
| validate_config | 0.013 | 0.010 | 0.024 | 0.010 | 0.024 |
| get_mcp_tool_metrics | 0.865 | 0.532 | 2.279 | 0.436 | 2.279 |
| get_mcp_error_breakdown | 0.326 | 0.170 | 0.967 | 0.159 | 0.967 |
| get_capabilities | 0.936 | 1.003 | 1.154 | 0.680 | 1.154 |
| health_check | 0.314 | 0.258 | 0.607 | 0.163 | 0.607 |
| get_stats | 1.967 | 1.601 | 3.367 | 1.117 | 3.367 |
| list_pending | 0.344 | 0.111 | 1.264 | 0.082 | 1.264 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.011 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.106 |
| Telemetry | 2 | 2.279 |

## Aggregate

**P50:** 0.012 ms | **P95:** 1.577 ms | **Min:** 0.003 ms | **Max:** 3.367 ms

## P95 Gate

**Global P95:** 1.577 ms
**Threshold:** 50.0 ms
**Status:** PASS
