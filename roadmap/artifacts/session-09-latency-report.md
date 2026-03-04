# Session 09 — Latency Report

**Generated:** 2026-03-04 04:37 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.012 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.008 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.008 | 0.003 | 0.008 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.012 | 0.029 | 0.012 | 0.029 |
| get_config | 0.090 | 0.087 | 0.104 | 0.085 | 0.104 |
| validate_config | 0.014 | 0.011 | 0.026 | 0.011 | 0.026 |
| get_mcp_tool_metrics | 0.994 | 0.647 | 2.545 | 0.534 | 2.545 |
| get_mcp_error_breakdown | 0.334 | 0.149 | 1.072 | 0.120 | 1.072 |
| get_capabilities | 1.039 | 0.937 | 1.617 | 0.547 | 1.617 |
| health_check | 0.283 | 0.257 | 0.419 | 0.172 | 0.419 |
| get_stats | 1.526 | 1.276 | 3.036 | 0.929 | 3.036 |
| list_pending | 0.417 | 0.171 | 1.484 | 0.098 | 1.484 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.008 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 2.545 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.312 ms | **Min:** 0.003 ms | **Max:** 3.036 ms

## P95 Gate

**Global P95:** 1.312 ms
**Threshold:** 50.0 ms
**Status:** PASS
