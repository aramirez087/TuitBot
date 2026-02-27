# Session 09 — Latency Report

**Generated:** 2026-02-27 03:39 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.030 | 0.012 | 0.030 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.003 | 0.005 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.017 | 0.012 | 0.037 | 0.012 | 0.037 |
| get_config | 0.083 | 0.081 | 0.094 | 0.079 | 0.094 |
| validate_config | 0.019 | 0.010 | 0.053 | 0.009 | 0.053 |
| get_mcp_tool_metrics | 1.029 | 0.534 | 3.012 | 0.485 | 3.012 |
| get_mcp_error_breakdown | 0.213 | 0.117 | 0.556 | 0.103 | 0.556 |
| get_capabilities | 0.992 | 0.913 | 1.579 | 0.692 | 1.579 |
| health_check | 0.377 | 0.230 | 1.002 | 0.159 | 1.002 |
| get_stats | 1.809 | 1.451 | 3.540 | 1.166 | 3.540 |
| list_pending | 0.509 | 0.166 | 1.760 | 0.154 | 1.760 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.094 |
| Telemetry | 2 | 3.012 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.579 ms | **Min:** 0.003 ms | **Max:** 3.540 ms

## P95 Gate

**Global P95:** 1.579 ms
**Threshold:** 50.0 ms
**Status:** PASS
