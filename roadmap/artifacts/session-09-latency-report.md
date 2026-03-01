# Session 09 — Latency Report

**Generated:** 2026-03-01 22:24 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.013 | 0.018 | 0.013 | 0.018 |
| kernel::search_tweets | 0.009 | 0.009 | 0.012 | 0.008 | 0.012 |
| kernel::get_followers | 0.006 | 0.005 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.014 | 0.013 | 0.022 | 0.012 | 0.022 |
| get_config | 0.093 | 0.091 | 0.105 | 0.087 | 0.105 |
| validate_config | 0.021 | 0.011 | 0.061 | 0.010 | 0.061 |
| get_mcp_tool_metrics | 1.008 | 0.620 | 2.654 | 0.545 | 2.654 |
| get_mcp_error_breakdown | 0.258 | 0.159 | 0.712 | 0.081 | 0.712 |
| get_capabilities | 0.851 | 0.649 | 1.749 | 0.574 | 1.749 |
| health_check | 0.383 | 0.260 | 0.711 | 0.160 | 0.711 |
| get_stats | 1.803 | 1.649 | 3.110 | 1.256 | 3.110 |
| list_pending | 0.518 | 0.305 | 1.695 | 0.121 | 1.695 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.105 |
| Telemetry | 2 | 2.654 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.654 ms | **Min:** 0.003 ms | **Max:** 3.110 ms

## P95 Gate

**Global P95:** 1.654 ms
**Threshold:** 50.0 ms
**Status:** PASS
