# Session 09 — Latency Report

**Generated:** 2026-03-01 20:36 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.012 | 0.038 | 0.012 | 0.038 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.005 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.003 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.019 | 0.012 | 0.047 | 0.012 | 0.047 |
| get_config | 0.090 | 0.089 | 0.102 | 0.084 | 0.102 |
| validate_config | 0.019 | 0.010 | 0.053 | 0.010 | 0.053 |
| get_mcp_tool_metrics | 0.960 | 0.562 | 2.623 | 0.479 | 2.623 |
| get_mcp_error_breakdown | 0.220 | 0.133 | 0.608 | 0.095 | 0.608 |
| get_capabilities | 0.824 | 0.774 | 1.050 | 0.700 | 1.050 |
| health_check | 0.349 | 0.223 | 0.839 | 0.164 | 0.839 |
| get_stats | 1.553 | 1.335 | 2.784 | 0.913 | 2.784 |
| list_pending | 0.482 | 0.219 | 1.386 | 0.090 | 1.386 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.102 |
| Telemetry | 2 | 2.623 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.335 ms | **Min:** 0.003 ms | **Max:** 2.784 ms

## P95 Gate

**Global P95:** 1.335 ms
**Threshold:** 50.0 ms
**Status:** PASS
