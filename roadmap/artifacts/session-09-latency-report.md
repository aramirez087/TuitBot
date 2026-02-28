# Session 09 — Latency Report

**Generated:** 2026-02-28 23:53 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.030 | 0.024 | 0.045 | 0.022 | 0.045 |
| kernel::search_tweets | 0.016 | 0.014 | 0.021 | 0.014 | 0.021 |
| kernel::get_followers | 0.012 | 0.011 | 0.014 | 0.011 | 0.014 |
| kernel::get_user_by_id | 0.014 | 0.014 | 0.015 | 0.013 | 0.015 |
| kernel::get_me | 0.015 | 0.014 | 0.018 | 0.014 | 0.018 |
| kernel::post_tweet | 0.009 | 0.007 | 0.013 | 0.007 | 0.013 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.003 | 0.005 |
| score_tweet | 0.040 | 0.024 | 0.098 | 0.022 | 0.098 |
| get_config | 0.206 | 0.185 | 0.255 | 0.167 | 0.255 |
| validate_config | 0.022 | 0.011 | 0.061 | 0.010 | 0.061 |
| get_mcp_tool_metrics | 1.269 | 0.963 | 2.814 | 0.718 | 2.814 |
| get_mcp_error_breakdown | 0.336 | 0.199 | 0.792 | 0.191 | 0.792 |
| get_capabilities | 1.621 | 1.492 | 2.319 | 1.272 | 2.319 |
| health_check | 0.426 | 0.266 | 1.070 | 0.240 | 1.070 |
| get_stats | 2.327 | 1.928 | 4.128 | 1.756 | 4.128 |
| list_pending | 0.435 | 0.160 | 1.559 | 0.132 | 1.559 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.037 |
| Kernel write | 2 | 0.013 |
| Config | 3 | 0.255 |
| Telemetry | 2 | 2.814 |

## Aggregate

**P50:** 0.033 ms | **P95:** 1.928 ms | **Min:** 0.003 ms | **Max:** 4.128 ms

## P95 Gate

**Global P95:** 1.928 ms
**Threshold:** 50.0 ms
**Status:** PASS
