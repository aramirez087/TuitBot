# Session 09 — Latency Report

**Generated:** 2026-03-06 03:57 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.026 | 0.021 | 0.047 | 0.020 | 0.047 |
| kernel::search_tweets | 0.016 | 0.015 | 0.021 | 0.014 | 0.021 |
| kernel::get_followers | 0.012 | 0.006 | 0.032 | 0.006 | 0.032 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.012 | 0.029 | 0.012 | 0.029 |
| get_config | 0.090 | 0.088 | 0.102 | 0.085 | 0.102 |
| validate_config | 0.015 | 0.013 | 0.027 | 0.012 | 0.027 |
| get_mcp_tool_metrics | 1.076 | 0.636 | 2.984 | 0.503 | 2.984 |
| get_mcp_error_breakdown | 0.268 | 0.216 | 0.547 | 0.151 | 0.547 |
| get_capabilities | 1.035 | 0.907 | 1.795 | 0.732 | 1.795 |
| health_check | 0.259 | 0.185 | 0.524 | 0.134 | 0.524 |
| get_stats | 1.966 | 1.885 | 3.127 | 1.155 | 3.127 |
| list_pending | 0.433 | 0.134 | 1.644 | 0.125 | 1.644 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.032 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.102 |
| Telemetry | 2 | 2.984 |

## Aggregate

**P50:** 0.023 ms | **P95:** 1.795 ms | **Min:** 0.003 ms | **Max:** 3.127 ms

## P95 Gate

**Global P95:** 1.795 ms
**Threshold:** 50.0 ms
**Status:** PASS
