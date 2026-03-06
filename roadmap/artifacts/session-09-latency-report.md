# Session 09 — Latency Report

**Generated:** 2026-03-06 04:26 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.029 | 0.023 | 0.058 | 0.020 | 0.058 |
| kernel::search_tweets | 0.014 | 0.010 | 0.024 | 0.008 | 0.024 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.003 | 0.005 |
| score_tweet | 0.018 | 0.013 | 0.036 | 0.012 | 0.036 |
| get_config | 0.094 | 0.093 | 0.103 | 0.086 | 0.103 |
| validate_config | 0.074 | 0.013 | 0.320 | 0.012 | 0.320 |
| get_mcp_tool_metrics | 1.053 | 0.716 | 2.531 | 0.529 | 2.531 |
| get_mcp_error_breakdown | 0.289 | 0.252 | 0.445 | 0.183 | 0.445 |
| get_capabilities | 0.929 | 0.774 | 1.565 | 0.692 | 1.565 |
| health_check | 0.372 | 0.314 | 0.793 | 0.199 | 0.793 |
| get_stats | 1.798 | 1.484 | 3.391 | 1.246 | 3.391 |
| list_pending | 0.408 | 0.137 | 1.525 | 0.081 | 1.525 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.024 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.320 |
| Telemetry | 2 | 2.531 |

## Aggregate

**P50:** 0.023 ms | **P95:** 1.507 ms | **Min:** 0.003 ms | **Max:** 3.391 ms

## P95 Gate

**Global P95:** 1.507 ms
**Threshold:** 50.0 ms
**Status:** PASS
