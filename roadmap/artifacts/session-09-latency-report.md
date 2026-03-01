# Session 09 — Latency Report

**Generated:** 2026-03-01 05:25 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.011 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.012 | 0.029 | 0.012 | 0.029 |
| get_config | 0.095 | 0.096 | 0.103 | 0.085 | 0.103 |
| validate_config | 0.013 | 0.010 | 0.026 | 0.010 | 0.026 |
| get_mcp_tool_metrics | 1.047 | 0.645 | 2.966 | 0.409 | 2.966 |
| get_mcp_error_breakdown | 0.213 | 0.142 | 0.456 | 0.121 | 0.456 |
| get_capabilities | 0.831 | 0.778 | 1.086 | 0.596 | 1.086 |
| health_check | 0.252 | 0.180 | 0.556 | 0.148 | 0.556 |
| get_stats | 1.443 | 1.151 | 2.445 | 0.884 | 2.445 |
| list_pending | 0.301 | 0.120 | 0.951 | 0.087 | 0.951 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.103 |
| Telemetry | 2 | 2.966 |

## Aggregate

**P50:** 0.012 ms | **P95:** 1.086 ms | **Min:** 0.003 ms | **Max:** 2.966 ms

## P95 Gate

**Global P95:** 1.086 ms
**Threshold:** 50.0 ms
**Status:** PASS
