# Session 09 — Latency Report

**Generated:** 2026-03-07 23:19 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.012 | 0.039 | 0.011 | 0.039 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.009 | 0.008 | 0.011 | 0.008 | 0.011 |
| kernel::get_me | 0.008 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.019 | 0.013 | 0.042 | 0.012 | 0.042 |
| get_config | 0.091 | 0.088 | 0.107 | 0.086 | 0.107 |
| validate_config | 0.020 | 0.012 | 0.051 | 0.011 | 0.051 |
| get_mcp_tool_metrics | 0.984 | 0.681 | 2.341 | 0.475 | 2.341 |
| get_mcp_error_breakdown | 0.319 | 0.199 | 0.922 | 0.081 | 0.922 |
| get_capabilities | 0.993 | 0.848 | 1.546 | 0.767 | 1.546 |
| health_check | 0.258 | 0.230 | 0.488 | 0.115 | 0.488 |
| get_stats | 1.587 | 1.155 | 2.802 | 1.143 | 2.802 |
| list_pending | 0.584 | 0.258 | 1.722 | 0.215 | 1.722 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.107 |
| Telemetry | 2 | 2.341 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.546 ms | **Min:** 0.003 ms | **Max:** 2.802 ms

## P95 Gate

**Global P95:** 1.546 ms
**Threshold:** 50.0 ms
**Status:** PASS
