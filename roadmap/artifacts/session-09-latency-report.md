# Session 09 — Latency Report

**Generated:** 2026-03-02 00:24 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.019 | 0.011 | 0.019 |
| kernel::search_tweets | 0.008 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.016 | 0.014 | 0.025 | 0.013 | 0.025 |
| get_config | 0.093 | 0.093 | 0.102 | 0.088 | 0.102 |
| validate_config | 0.081 | 0.011 | 0.358 | 0.011 | 0.358 |
| get_mcp_tool_metrics | 1.014 | 0.656 | 2.737 | 0.454 | 2.737 |
| get_mcp_error_breakdown | 0.291 | 0.154 | 0.864 | 0.112 | 0.864 |
| get_capabilities | 0.866 | 0.879 | 1.078 | 0.694 | 1.078 |
| health_check | 0.263 | 0.177 | 0.662 | 0.127 | 0.662 |
| get_stats | 1.682 | 1.317 | 3.612 | 0.984 | 3.612 |
| list_pending | 0.291 | 0.102 | 1.038 | 0.072 | 1.038 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.012 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.358 |
| Telemetry | 2 | 2.737 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.102 ms | **Min:** 0.003 ms | **Max:** 3.612 ms

## P95 Gate

**Global P95:** 1.102 ms
**Threshold:** 50.0 ms
**Status:** PASS
