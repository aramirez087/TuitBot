# Session 09 — Latency Report

**Generated:** 2026-02-26 18:09 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.022 | 0.012 | 0.062 | 0.011 | 0.062 |
| kernel::search_tweets | 0.009 | 0.007 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.006 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.003 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.015 | 0.012 | 0.025 | 0.012 | 0.025 |
| get_config | 0.083 | 0.080 | 0.092 | 0.080 | 0.092 |
| validate_config | 0.021 | 0.010 | 0.062 | 0.009 | 0.062 |
| get_mcp_tool_metrics | 1.262 | 1.133 | 2.666 | 0.585 | 2.666 |
| get_mcp_error_breakdown | 0.310 | 0.280 | 0.582 | 0.167 | 0.582 |
| get_capabilities | 1.021 | 0.988 | 1.269 | 0.782 | 1.269 |
| health_check | 0.294 | 0.289 | 0.370 | 0.235 | 0.370 |
| get_stats | 1.866 | 1.540 | 3.235 | 1.486 | 3.235 |
| list_pending | 0.514 | 0.214 | 1.651 | 0.130 | 1.651 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.016 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.092 |
| Telemetry | 2 | 2.666 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.540 ms | **Min:** 0.003 ms | **Max:** 3.235 ms

## P95 Gate

**Global P95:** 1.540 ms
**Threshold:** 50.0 ms
**Status:** PASS
