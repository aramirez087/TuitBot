# Session 09 — Latency Report

**Generated:** 2026-03-03 04:05 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.015 | 0.012 | 0.026 | 0.012 | 0.026 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::get_followers | 0.007 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.008 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.030 | 0.012 | 0.030 |
| get_config | 0.091 | 0.089 | 0.103 | 0.086 | 0.103 |
| validate_config | 0.015 | 0.012 | 0.027 | 0.012 | 0.027 |
| get_mcp_tool_metrics | 1.143 | 0.725 | 3.255 | 0.393 | 3.255 |
| get_mcp_error_breakdown | 0.229 | 0.162 | 0.551 | 0.125 | 0.551 |
| get_capabilities | 0.891 | 0.726 | 1.718 | 0.555 | 1.718 |
| health_check | 0.217 | 0.160 | 0.365 | 0.151 | 0.365 |
| get_stats | 1.811 | 1.628 | 2.703 | 1.203 | 2.703 |
| list_pending | 0.348 | 0.153 | 1.176 | 0.119 | 1.176 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.103 |
| Telemetry | 2 | 3.255 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.628 ms | **Min:** 0.003 ms | **Max:** 3.255 ms

## P95 Gate

**Global P95:** 1.628 ms
**Threshold:** 50.0 ms
**Status:** PASS
