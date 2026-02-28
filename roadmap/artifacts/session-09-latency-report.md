# Session 09 — Latency Report

**Generated:** 2026-02-28 02:30 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.028 | 0.011 | 0.028 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.028 | 0.012 | 0.028 |
| get_config | 0.086 | 0.083 | 0.099 | 0.082 | 0.099 |
| validate_config | 0.017 | 0.010 | 0.046 | 0.010 | 0.046 |
| get_mcp_tool_metrics | 0.927 | 0.460 | 2.687 | 0.437 | 2.687 |
| get_mcp_error_breakdown | 0.214 | 0.121 | 0.616 | 0.088 | 0.616 |
| get_capabilities | 0.822 | 0.816 | 1.149 | 0.583 | 1.149 |
| health_check | 0.265 | 0.228 | 0.632 | 0.108 | 0.632 |
| get_stats | 1.474 | 1.186 | 2.907 | 0.994 | 2.907 |
| list_pending | 0.369 | 0.151 | 1.255 | 0.111 | 1.255 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.099 |
| Telemetry | 2 | 2.687 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.186 ms | **Min:** 0.003 ms | **Max:** 2.907 ms

## P95 Gate

**Global P95:** 1.186 ms
**Threshold:** 50.0 ms
**Status:** PASS
