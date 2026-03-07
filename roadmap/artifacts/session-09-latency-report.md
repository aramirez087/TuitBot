# Session 09 — Latency Report

**Generated:** 2026-03-07 01:58 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.029 | 0.011 | 0.029 |
| kernel::search_tweets | 0.009 | 0.007 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.052 | 0.014 | 0.175 | 0.012 | 0.175 |
| get_config | 0.093 | 0.091 | 0.104 | 0.088 | 0.104 |
| validate_config | 0.075 | 0.012 | 0.326 | 0.011 | 0.326 |
| get_mcp_tool_metrics | 1.236 | 0.909 | 2.901 | 0.469 | 2.901 |
| get_mcp_error_breakdown | 0.284 | 0.167 | 0.802 | 0.110 | 0.802 |
| get_capabilities | 1.018 | 0.840 | 1.491 | 0.812 | 1.491 |
| health_check | 0.250 | 0.198 | 0.451 | 0.172 | 0.451 |
| get_stats | 1.575 | 1.266 | 3.164 | 0.887 | 3.164 |
| list_pending | 0.499 | 0.159 | 1.817 | 0.139 | 1.817 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.326 |
| Telemetry | 2 | 2.901 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.476 ms | **Min:** 0.003 ms | **Max:** 3.164 ms

## P95 Gate

**Global P95:** 1.476 ms
**Threshold:** 50.0 ms
**Status:** PASS
