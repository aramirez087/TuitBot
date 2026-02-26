# Session 09 — Latency Report

**Generated:** 2026-02-26 03:55 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.013 | 0.037 | 0.012 | 0.037 |
| kernel::search_tweets | 0.011 | 0.008 | 0.022 | 0.008 | 0.022 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.007 | 0.009 |
| kernel::get_me | 0.007 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.005 | 0.004 | 0.005 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.018 | 0.013 | 0.040 | 0.012 | 0.040 |
| get_config | 0.084 | 0.082 | 0.096 | 0.080 | 0.096 |
| validate_config | 0.019 | 0.011 | 0.053 | 0.010 | 0.053 |
| get_mcp_tool_metrics | 1.394 | 0.900 | 3.316 | 0.748 | 3.316 |
| get_mcp_error_breakdown | 0.313 | 0.202 | 0.746 | 0.154 | 0.746 |
| get_capabilities | 0.857 | 0.782 | 1.220 | 0.733 | 1.220 |
| health_check | 0.329 | 0.293 | 0.532 | 0.189 | 0.532 |
| get_stats | 1.669 | 1.172 | 3.540 | 0.962 | 3.540 |
| list_pending | 0.431 | 0.228 | 1.382 | 0.143 | 1.382 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.022 |
| Kernel write | 2 | 0.005 |
| Config | 3 | 0.096 |
| Telemetry | 2 | 3.316 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.220 ms | **Min:** 0.004 ms | **Max:** 3.540 ms

## P95 Gate

**Global P95:** 1.220 ms
**Threshold:** 50.0 ms
**Status:** PASS
