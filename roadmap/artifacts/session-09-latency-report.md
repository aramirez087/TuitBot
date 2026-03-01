# Session 09 — Latency Report

**Generated:** 2026-03-01 02:37 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.012 | 0.011 | 0.015 | 0.011 | 0.015 |
| kernel::search_tweets | 0.008 | 0.008 | 0.011 | 0.007 | 0.011 |
| kernel::get_followers | 0.007 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.010 | 0.007 | 0.010 |
| kernel::get_me | 0.009 | 0.009 | 0.009 | 0.008 | 0.009 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| score_tweet | 0.015 | 0.014 | 0.022 | 0.013 | 0.022 |
| get_config | 0.119 | 0.100 | 0.195 | 0.095 | 0.195 |
| validate_config | 0.014 | 0.011 | 0.023 | 0.010 | 0.023 |
| get_mcp_tool_metrics | 0.912 | 0.583 | 2.346 | 0.497 | 2.346 |
| get_mcp_error_breakdown | 0.252 | 0.149 | 0.693 | 0.113 | 0.693 |
| get_capabilities | 0.894 | 0.786 | 1.395 | 0.712 | 1.395 |
| health_check | 0.249 | 0.197 | 0.546 | 0.100 | 0.546 |
| get_stats | 1.648 | 1.100 | 3.712 | 1.048 | 3.712 |
| list_pending | 0.475 | 0.308 | 1.174 | 0.163 | 1.174 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.195 |
| Telemetry | 2 | 2.346 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.174 ms | **Min:** 0.004 ms | **Max:** 3.712 ms

## P95 Gate

**Global P95:** 1.174 ms
**Threshold:** 50.0 ms
**Status:** PASS
