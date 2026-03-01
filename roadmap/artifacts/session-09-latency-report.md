# Session 09 — Latency Report

**Generated:** 2026-03-01 02:54 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.020 | 0.022 | 0.028 | 0.012 | 0.028 |
| kernel::search_tweets | 0.010 | 0.008 | 0.016 | 0.008 | 0.016 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.013 | 0.014 | 0.019 | 0.007 | 0.019 |
| kernel::get_me | 0.014 | 0.014 | 0.014 | 0.014 | 0.014 |
| kernel::post_tweet | 0.004 | 0.004 | 0.007 | 0.004 | 0.007 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.012 | 0.023 | 0.012 | 0.023 |
| get_config | 0.129 | 0.104 | 0.177 | 0.100 | 0.177 |
| validate_config | 0.055 | 0.010 | 0.235 | 0.010 | 0.235 |
| get_mcp_tool_metrics | 1.197 | 1.010 | 2.552 | 0.606 | 2.552 |
| get_mcp_error_breakdown | 0.223 | 0.127 | 0.623 | 0.080 | 0.623 |
| get_capabilities | 1.147 | 0.996 | 1.842 | 0.841 | 1.842 |
| health_check | 0.336 | 0.225 | 0.804 | 0.185 | 0.804 |
| get_stats | 1.987 | 1.466 | 3.874 | 1.406 | 3.874 |
| list_pending | 0.424 | 0.110 | 1.658 | 0.088 | 1.658 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.022 |
| Kernel write | 2 | 0.007 |
| Config | 3 | 0.235 |
| Telemetry | 2 | 2.552 |

## Aggregate

**P50:** 0.022 ms | **P95:** 1.658 ms | **Min:** 0.003 ms | **Max:** 3.874 ms

## P95 Gate

**Global P95:** 1.658 ms
**Threshold:** 50.0 ms
**Status:** PASS
