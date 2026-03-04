# Session 09 — Latency Report

**Generated:** 2026-03-04 04:00 UTC

**Tools benchmarked:** 16

## Per-tool Results

| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|------|----------|----------|----------|----------|----------|
| kernel::get_tweet | 0.014 | 0.012 | 0.019 | 0.011 | 0.019 |
| kernel::search_tweets | 0.009 | 0.008 | 0.014 | 0.007 | 0.014 |
| kernel::get_followers | 0.006 | 0.006 | 0.008 | 0.005 | 0.008 |
| kernel::get_user_by_id | 0.007 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::get_me | 0.008 | 0.007 | 0.008 | 0.007 | 0.008 |
| kernel::post_tweet | 0.004 | 0.004 | 0.006 | 0.003 | 0.006 |
| kernel::reply_to_tweet | 0.004 | 0.003 | 0.004 | 0.003 | 0.004 |
| score_tweet | 0.017 | 0.013 | 0.036 | 0.012 | 0.036 |
| get_config | 0.090 | 0.087 | 0.102 | 0.085 | 0.102 |
| validate_config | 0.097 | 0.012 | 0.440 | 0.011 | 0.440 |
| get_mcp_tool_metrics | 1.337 | 0.967 | 2.986 | 0.573 | 2.986 |
| get_mcp_error_breakdown | 0.300 | 0.208 | 0.776 | 0.116 | 0.776 |
| get_capabilities | 0.967 | 1.013 | 1.283 | 0.629 | 1.283 |
| health_check | 0.327 | 0.286 | 0.517 | 0.227 | 0.517 |
| get_stats | 1.805 | 1.467 | 3.271 | 1.165 | 3.271 |
| list_pending | 0.408 | 0.173 | 1.405 | 0.128 | 1.405 |

## Category Breakdown

| Category | Tools | P95 (ms) |
|----------|-------|----------|
| Kernel read | 5 | 0.014 |
| Kernel write | 2 | 0.006 |
| Config | 3 | 0.440 |
| Telemetry | 2 | 2.986 |

## Aggregate

**P50:** 0.014 ms | **P95:** 1.405 ms | **Min:** 0.003 ms | **Max:** 3.271 ms

## P95 Gate

**Global P95:** 1.405 ms
**Threshold:** 50.0 ms
**Status:** PASS
