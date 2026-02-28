# Session 09 — Latency Report

**Generated:** 2026-02-28 03:30 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.027 | 0.027 | 0.040 | 0.013 | 0.040 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.008 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.023 | 0.012 | 0.023 |
| get_config | 0.087 | 0.085 | 0.096 | 0.083 | 0.096 |
| validate_config | 0.013 | 0.010 | 0.024 | 0.010 | 0.024 |
| get_mcp_tool_metrics | 0.956 | 0.520 | 2.799 | 0.458 | 2.799 |
| get_mcp_error_breakdown | 0.197 | 0.122 | 0.520 | 0.107 | 0.520 |
| get_capabilities | 0.926 | 0.757 | 1.644 | 0.681 | 1.644 |
| health_check | 0.400 | 0.449 | 0.586 | 0.172 | 0.586 |
| get_stats | 1.584 | 1.251 | 2.797 | 1.247 | 2.797 |
| list_pending | 0.375 | 0.122 | 1.313 | 0.105 | 1.313 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.029 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.096 |
| Telemetry | 2 | 2.799 |

## Aggregate

**P50:** 0.024 ms | **P95:** 1.313 ms | **Min:** 0.003 ms | **Max:** 2.799 ms

## P95 Gate

**Global P95:** 1.313 ms
**Threshold:** 50.0 ms
**Status:** PASS
