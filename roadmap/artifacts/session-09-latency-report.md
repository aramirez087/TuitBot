# Session 09 — Latency Report

**Generated:** 2026-03-07 02:50 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.019 | 0.012 | 0.041 | 0.011 | 0.041 |
| kernel::search_tweets | 0.008 | 0.007 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.020 | 0.012 | 0.050 | 0.012 | 0.050 |
| get_config | 0.091 | 0.088 | 0.104 | 0.086 | 0.104 |
| validate_config | 0.019 | 0.012 | 0.049 | 0.011 | 0.049 |
| get_mcp_tool_metrics | 1.014 | 0.632 | 2.540 | 0.541 | 2.540 |
| get_mcp_error_breakdown | 0.162 | 0.156 | 0.316 | 0.076 | 0.316 |
| get_capabilities | 0.954 | 0.997 | 1.223 | 0.610 | 1.223 |
| health_check | 0.425 | 0.266 | 0.788 | 0.155 | 0.788 |
| get_stats | 1.848 | 1.835 | 2.823 | 1.158 | 2.823 |
| list_pending | 0.430 | 0.141 | 1.642 | 0.102 | 1.642 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.019 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 2.540 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.642 ms | **Min:** 0.003 ms | **Max:** 2.823 ms

## P95 Gate

**Global P95:** 1.642 ms
**Threshold:** 50.0 ms
**Status:** PASS
