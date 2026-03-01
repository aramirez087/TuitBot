# Session 09 — Latency Report

**Generated:** 2026-03-01 23:31 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.016 | 0.013 | 0.029 | 0.012 | 0.029 |
| kernel::search_tweets | 0.009 | 0.007 | 0.014 | 0.007 | 0.014 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.007 | 0.007 | 0.007 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.015 | 0.012 | 0.028 | 0.012 | 0.028 |
| get_config | 0.091 | 0.088 | 0.107 | 0.085 | 0.107 |
| validate_config | 0.083 | 0.011 | 0.370 | 0.010 | 0.370 |
| get_mcp_tool_metrics | 1.040 | 0.540 | 3.037 | 0.462 | 3.037 |
| get_mcp_error_breakdown | 0.372 | 0.256 | 0.740 | 0.145 | 0.740 |
| get_capabilities | 1.055 | 0.874 | 1.811 | 0.680 | 1.811 |
| health_check | 0.327 | 0.262 | 0.612 | 0.224 | 0.612 |
| get_stats | 1.463 | 1.324 | 2.383 | 1.008 | 2.383 |
| list_pending | 0.458 | 0.130 | 1.833 | 0.069 | 1.833 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.014 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.370 |
| Telemetry | 2 | 3.037 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.379 ms | **Min:** 0.004 ms | **Max:** 3.037 ms

## P95 Gate

**Global P95:** 1.379 ms
**Threshold:** 50.0 ms
**Status:** PASS
