# Session 09 — Latency Report

**Generated:** 2026-03-04 04:13 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.012 | 0.026 | 0.011 | 0.026 |
| kernel::search_tweets | 0.008 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.013 | 0.031 | 0.012 | 0.031 |
| get_config | 0.093 | 0.089 | 0.109 | 0.087 | 0.109 |
| validate_config | 0.015 | 0.011 | 0.026 | 0.011 | 0.026 |
| get_mcp_tool_metrics | 1.072 | 0.615 | 2.915 | 0.600 | 2.915 |
| get_mcp_error_breakdown | 0.258 | 0.155 | 0.718 | 0.090 | 0.718 |
| get_capabilities | 0.898 | 0.795 | 1.417 | 0.676 | 1.417 |
| health_check | 0.264 | 0.277 | 0.494 | 0.097 | 0.494 |
| get_stats | 1.578 | 1.423 | 2.681 | 1.142 | 2.681 |
| list_pending | 0.364 | 0.141 | 1.276 | 0.107 | 1.276 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.109 |
| Telemetry | 2 | 2.915 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.417 ms | **Min:** 0.003 ms | **Max:** 2.915 ms

## P95 Gate

**Global P95:** 1.417 ms
**Threshold:** 50.0 ms
**Status:** PASS
