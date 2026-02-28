# Session 09 — Latency Report

**Generated:** 2026-02-28 02:54 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.025 | 0.022 | 0.040 | 0.012 | 0.040 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.010 | 0.007 | 0.010 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.012 | 0.029 | 0.011 | 0.029 |
| get_config | 0.088 | 0.086 | 0.102 | 0.083 | 0.102 |
| validate_config | 0.013 | 0.010 | 0.024 | 0.009 | 0.024 |
| get_mcp_tool_metrics | 1.022 | 0.587 | 2.711 | 0.565 | 2.711 |
| get_mcp_error_breakdown | 0.348 | 0.135 | 1.082 | 0.128 | 1.082 |
| get_capabilities | 0.981 | 0.799 | 1.460 | 0.632 | 1.460 |
| health_check | 0.176 | 0.160 | 0.213 | 0.150 | 0.213 |
| get_stats | 1.900 | 1.574 | 2.982 | 1.135 | 2.982 |
| list_pending | 0.378 | 0.182 | 1.146 | 0.143 | 1.146 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.032 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.102 |
| Telemetry | 2 | 2.711 |

## Aggregate

**P50:** 0.022 ms | **P95:** 1.471 ms | **Min:** 0.003 ms | **Max:** 2.982 ms

## P95 Gate

**Global P95:** 1.471 ms
**Threshold:** 50.0 ms
**Status:** PASS
