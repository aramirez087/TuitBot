# Session 09 — Latency Report

**Generated:** 2026-02-26 17:46 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.013 | 0.012 | 0.017 | 0.012 | 0.017 |
| kernel::search_tweets | 0.009 | 0.008 | 0.012 | 0.007 | 0.012 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.009 | 0.008 | 0.009 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.004 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| score_tweet | 0.015 | 0.013 | 0.022 | 0.012 | 0.022 |
| get_config | 0.084 | 0.082 | 0.091 | 0.081 | 0.091 |
| validate_config | 0.017 | 0.010 | 0.046 | 0.010 | 0.046 |
| get_mcp_tool_metrics | 1.177 | 0.798 | 2.820 | 0.665 | 2.820 |
| get_mcp_error_breakdown | 0.300 | 0.177 | 0.864 | 0.131 | 0.864 |
| get_capabilities | 0.807 | 0.879 | 0.957 | 0.622 | 0.957 |
| health_check | 0.317 | 0.232 | 0.668 | 0.164 | 0.668 |
| get_stats | 1.743 | 1.483 | 2.861 | 1.378 | 2.861 |
| list_pending | 0.509 | 0.236 | 1.639 | 0.151 | 1.639 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.091 |
| Telemetry | 2 | 2.820 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.483 ms | **Min:** 0.004 ms | **Max:** 2.861 ms

## P95 Gate

**Global P95:** 1.483 ms
**Threshold:** 50.0 ms
**Status:** PASS
