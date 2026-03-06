# Session 09 — Latency Report

**Generated:** 2026-03-06 18:37 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.017 | 0.011 | 0.043 | 0.011 | 0.043 |
| kernel::search_tweets | 0.009 | 0.008 | 0.013 | 0.007 | 0.013 |
| kernel::get_followers | 0.006 | 0.006 | 0.009 | 0.006 | 0.009 |
| kernel::get_user_by_id | 0.008 | 0.008 | 0.010 | 0.008 | 0.010 |
| kernel::get_me | 0.008 | 0.008 | 0.008 | 0.008 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.018 | 0.013 | 0.039 | 0.011 | 0.039 |
| get_config | 0.091 | 0.087 | 0.104 | 0.087 | 0.104 |
| validate_config | 0.019 | 0.011 | 0.052 | 0.011 | 0.052 |
| get_mcp_tool_metrics | 1.538 | 0.860 | 3.600 | 0.667 | 3.600 |
| get_mcp_error_breakdown | 0.376 | 0.219 | 0.864 | 0.170 | 0.864 |
| get_capabilities | 0.978 | 0.903 | 1.334 | 0.735 | 1.334 |
| health_check | 0.286 | 0.287 | 0.420 | 0.161 | 0.420 |
| get_stats | 1.770 | 1.338 | 3.588 | 1.228 | 3.588 |
| list_pending | 0.399 | 0.184 | 1.347 | 0.114 | 1.347 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.013 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.104 |
| Telemetry | 2 | 3.600 |

## Aggregate

**P50:** 0.013 ms | **P95:** 1.347 ms | **Min:** 0.003 ms | **Max:** 3.600 ms

## P95 Gate

**Global P95:** 1.347 ms
**Threshold:** 50.0 ms
**Status:** PASS
