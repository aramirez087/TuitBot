# Session 09 — Latency Report

**Generated:** 2026-02-28 00:09 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.028 | 0.022 | 0.051 | 0.021 | 0.051 |
| kernel::search_tweets | 0.017 | 0.017 | 0.021 | 0.016 | 0.021 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.010 | 0.007 | 0.010 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.029 | 0.012 | 0.029 |
| get_config | 0.092 | 0.090 | 0.104 | 0.086 | 0.104 |
| validate_config | 0.021 | 0.011 | 0.063 | 0.010 | 0.063 |
| get_mcp_tool_metrics | 1.046 | 0.717 | 2.556 | 0.548 | 2.556 |
| get_mcp_error_breakdown | 0.264 | 0.135 | 0.840 | 0.083 | 0.840 |
| get_capabilities | 0.868 | 0.820 | 1.257 | 0.640 | 1.257 |
| health_check | 0.312 | 0.262 | 0.558 | 0.142 | 0.558 |
| get_stats | 1.546 | 1.352 | 2.601 | 1.095 | 2.601 |
| list_pending | 0.386 | 0.210 | 1.215 | 0.116 | 1.215 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.024 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 2.556 |

## Aggregate

**P50:** 0.022 ms | **P95:** 1.263 ms | **Min:** 0.003 ms | **Max:** 2.601 ms

## P95 Gate

**Global P95:** 1.263 ms
**Threshold:** 50.0 ms
**Status:** PASS
