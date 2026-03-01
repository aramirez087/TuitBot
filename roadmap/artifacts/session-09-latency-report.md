# Session 09 — Latency Report

**Generated:** 2026-03-01 03:40 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.029 | 0.024 | 0.059 | 0.015 | 0.059 |
| kernel::search_tweets | 0.016 | 0.015 | 0.021 | 0.014 | 0.021 |
| kernel::get_followers | 0.012 | 0.012 | 0.015 | 0.011 | 0.015 |
| kernel::get_user_by_id | 0.015 | 0.014 | 0.017 | 0.014 | 0.017 |
| kernel::get_me | 0.014 | 0.013 | 0.015 | 0.013 | 0.015 |
| kernel::post_tweet | 0.007 | 0.007 | 0.010 | 0.007 | 0.010 |
| kernel::reply_to_tweet | 0.007 | 0.007 | 0.007 | 0.007 | 0.007 |
| score_tweet | 0.025 | 0.023 | 0.036 | 0.022 | 0.036 |
| get_config | 0.126 | 0.102 | 0.180 | 0.091 | 0.180 |
| validate_config | 0.090 | 0.011 | 0.406 | 0.010 | 0.406 |
| get_mcp_tool_metrics | 1.193 | 0.869 | 2.789 | 0.658 | 2.789 |
| get_mcp_error_breakdown | 0.354 | 0.210 | 0.997 | 0.146 | 0.997 |
| get_capabilities | 0.956 | 0.922 | 1.394 | 0.725 | 1.394 |
| health_check | 0.269 | 0.212 | 0.559 | 0.130 | 0.559 |
| get_stats | 1.649 | 1.391 | 2.438 | 1.251 | 2.438 |
| list_pending | 0.319 | 0.138 | 1.126 | 0.085 | 1.126 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.024 |
| Kernel write | 2 | 0.010 |
| Config | 3 | 0.406 |
| Telemetry | 2 | 2.789 |

## Aggregate

**P50:** 0.024 ms | **P95:** 1.391 ms | **Min:** 0.007 ms | **Max:** 2.789 ms

## P95 Gate

**Global P95:** 1.391 ms
**Threshold:** 50.0 ms
**Status:** PASS
