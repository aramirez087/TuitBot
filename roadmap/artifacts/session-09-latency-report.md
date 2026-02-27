# Session 09 — Latency Report

**Generated:** 2026-02-27 00:02 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.012 | 0.021 | 0.012 | 0.021 |
| kernel::search_tweets | 0.009 | 0.008 | 0.014 | 0.008 | 0.014 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.012 | 0.025 | 0.011 | 0.025 |
| get_config | 0.084 | 0.082 | 0.097 | 0.079 | 0.097 |
| validate_config | 0.020 | 0.011 | 0.056 | 0.010 | 0.056 |
| get_mcp_tool_metrics | 1.017 | 0.681 | 2.673 | 0.367 | 2.673 |
| get_mcp_error_breakdown | 0.295 | 0.246 | 0.402 | 0.218 | 0.402 |
| get_capabilities | 0.966 | 0.890 | 1.440 | 0.711 | 1.440 |
| health_check | 0.251 | 0.172 | 0.663 | 0.121 | 0.663 |
| get_stats | 1.668 | 1.248 | 3.614 | 0.980 | 3.614 |
| list_pending | 0.463 | 0.162 | 1.715 | 0.093 | 1.715 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.014 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.097 |
| Telemetry | 2 | 2.673 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.440 ms | **Min:** 0.003 ms | **Max:** 3.614 ms

## P95 Gate

**Global P95:** 1.440 ms
**Threshold:** 50.0 ms
**Status:** PASS
