# Session 09 — Latency Report

**Generated:** 2026-03-06 00:07 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.012 | 0.040 | 0.011 | 0.040 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.005 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.017 | 0.012 | 0.037 | 0.012 | 0.037 |
| get_config | 0.089 | 0.087 | 0.101 | 0.085 | 0.101 |
| validate_config | 0.021 | 0.012 | 0.057 | 0.011 | 0.057 |
| get_mcp_tool_metrics | 1.107 | 0.645 | 3.073 | 0.479 | 3.073 |
| get_mcp_error_breakdown | 0.267 | 0.212 | 0.571 | 0.161 | 0.571 |
| get_capabilities | 1.101 | 0.941 | 1.619 | 0.798 | 1.619 |
| health_check | 0.372 | 0.233 | 1.057 | 0.110 | 1.057 |
| get_stats | 1.931 | 1.683 | 3.175 | 1.405 | 3.175 |
| list_pending | 0.394 | 0.123 | 1.518 | 0.100 | 1.518 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.101 |
| Telemetry | 2 | 3.073 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.619 ms | **Min:** 0.003 ms | **Max:** 3.175 ms

## P95 Gate

**Global P95:** 1.619 ms
**Threshold:** 50.0 ms
**Status:** PASS
