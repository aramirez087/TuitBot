# Session 09 — Latency Report

**Generated:** 2026-03-01 01:50 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.016 | 0.012 | 0.033 | 0.012 | 0.033 |
| kernel::search_tweets | 0.009 | 0.008 | 0.014 | 0.007 | 0.014 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.015 | 0.008 | 0.031 | 0.007 | 0.031 |
| kernel::post_tweet | 0.013 | 0.009 | 0.029 | 0.007 | 0.029 |
| kernel::reply_to_tweet | 0.008 | 0.008 | 0.010 | 0.007 | 0.010 |
| score_tweet | 0.026 | 0.017 | 0.053 | 0.012 | 0.053 |
| get_config | 0.102 | 0.103 | 0.112 | 0.089 | 0.112 |
| validate_config | 0.013 | 0.010 | 0.025 | 0.010 | 0.025 |
| get_mcp_tool_metrics | 1.117 | 0.674 | 2.891 | 0.667 | 2.891 |
| get_mcp_error_breakdown | 0.283 | 0.159 | 0.757 | 0.151 | 0.757 |
| get_capabilities | 1.170 | 0.914 | 2.229 | 0.717 | 2.229 |
| health_check | 0.279 | 0.247 | 0.453 | 0.199 | 0.453 |
| get_stats | 1.515 | 1.132 | 3.098 | 0.996 | 3.098 |
| list_pending | 0.285 | 0.127 | 0.895 | 0.116 | 0.895 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.031 |
| Kernel write | 2 | 0.029 |
| Config | 3 | 0.112 |
| Telemetry | 2 | 2.891 |

## Aggregate

**P50:** 0.029 ms | **P95:** 1.202 ms | **Min:** 0.005 ms | **Max:** 3.098 ms

## P95 Gate

**Global P95:** 1.202 ms
**Threshold:** 50.0 ms
**Status:** PASS
