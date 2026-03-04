# Session 09 — Latency Report

**Generated:** 2026-03-04 03:52 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.029 | 0.011 | 0.029 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.007 | 0.007 | 0.007 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.017 | 0.012 | 0.037 | 0.011 | 0.037 |
| get_config | 0.089 | 0.085 | 0.104 | 0.085 | 0.104 |
| validate_config | 0.021 | 0.012 | 0.057 | 0.011 | 0.057 |
| get_mcp_tool_metrics | 0.996 | 0.709 | 2.435 | 0.551 | 2.435 |
| get_mcp_error_breakdown | 0.188 | 0.143 | 0.368 | 0.119 | 0.368 |
| get_capabilities | 0.876 | 0.775 | 1.385 | 0.686 | 1.385 |
| health_check | 0.260 | 0.210 | 0.499 | 0.136 | 0.499 |
| get_stats | 1.675 | 1.190 | 3.095 | 1.027 | 3.095 |
| list_pending | 0.425 | 0.117 | 1.674 | 0.077 | 1.674 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 2.435 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.385 ms | **Min:** 0.003 ms | **Max:** 3.095 ms

## P95 Gate

**Global P95:** 1.385 ms
**Threshold:** 50.0 ms
**Status:** PASS
