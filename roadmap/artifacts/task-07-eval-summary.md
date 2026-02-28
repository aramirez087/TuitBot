# Task 07 — Observability Eval Results

**Generated:** 2026-02-28 04:01 UTC

## Scenarios

| Scenario | Description | Steps | Total (ms) | Success | Schema Valid | Telemetry Entries |
|----------|-------------|-------|------------|---------|--------------|-------------------|
| A | Raw direct reply flow: draft -> queue | 2 | 11 | PASS | PASS | 1 |
| B | Composite flow: find -> draft -> queue | 3 | 11 | PASS | PASS | 3 |
| C | Blocked-by-policy mutation with telemetry verification | 2 | 2 | PASS | PASS | 1 |

## Step Details

### Scenario A: Raw direct reply flow: draft -> queue

| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |
|------|-------------|---------|--------------|-------|--------|
| draft_replies_for_candidates | 7 | PASS | PASS | - | - |
| propose_and_queue_replies | 4 | PASS | PASS | - | allow |

### Scenario B: Composite flow: find -> draft -> queue

| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |
|------|-------------|---------|--------------|-------|--------|
| find_reply_opportunities | 2 | PASS | PASS | - | - |
| draft_replies_for_candidates | 5 | PASS | PASS | - | - |
| propose_and_queue_replies | 4 | PASS | PASS | - | allow |

### Scenario C: Blocked-by-policy mutation with telemetry verification

| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |
|------|-------------|---------|--------------|-------|--------|
| propose_and_queue_replies | 1 | FAIL | PASS | policy_denied_blocked | deny |
| get_mcp_error_breakdown | 1 | PASS | PASS | - | - |

## Quality Gates

| Gate | Rate | Threshold | Status |
|------|------|-----------|--------|
| Schema validation | 100.0% | 95% | PASS |
| Unknown errors | 0.0% | 5% | PASS |

**Overall: ALL GATES PASS**
