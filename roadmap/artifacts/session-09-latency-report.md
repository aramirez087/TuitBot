# Session 09 — Latency Report

**Generated:** 2026-03-01 03:21 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.019 | 0.012 | 0.046 | 0.011 | 0.046 |
| kernel::search_tweets | 0.009 | 0.007 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.007 | 0.007 | 0.007 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.012 | 0.025 | 0.012 | 0.025 |
| get_config | 0.089 | 0.087 | 0.101 | 0.084 | 0.101 |
| validate_config | 0.013 | 0.010 | 0.024 | 0.010 | 0.024 |
| get_mcp_tool_metrics | 1.125 | 0.661 | 3.037 | 0.531 | 3.037 |
| get_mcp_error_breakdown | 0.224 | 0.118 | 0.572 | 0.108 | 0.572 |
| get_capabilities | 0.918 | 0.916 | 1.268 | 0.633 | 1.268 |
| health_check | 0.367 | 0.399 | 0.442 | 0.221 | 0.442 |
| get_stats | 1.506 | 1.261 | 3.064 | 0.865 | 3.064 |
| list_pending | 0.385 | 0.143 | 1.404 | 0.096 | 1.404 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.015 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.101 |
| Telemetry | 2 | 3.037 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.268 ms | **Min:** 0.003 ms | **Max:** 3.064 ms

## P95 Gate

**Global P95:** 1.268 ms
**Threshold:** 50.0 ms
**Status:** PASS
