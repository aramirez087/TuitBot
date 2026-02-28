# Task 07 — Observability Eval Results

**Generated:** 2026-02-28 18:36 UTC

## Scenarios

| Scenario | Description | Steps | Total (ms) | Success | Schema Valid | Telemetry Entries |
|----------|-------------|-------|------------|---------|--------------|-------------------|
| A | Raw direct reply flow: draft -> queue | 2 | 9 | PASS | PASS | 1 |
| B | Composite flow: find -> draft -> queue | 3 | 10 | PASS | PASS | 3 |
| C | Blocked-by-policy mutation with telemetry verification | 2 | 0 | PASS | PASS | 1 |

## Step Details

### Scenario A: Raw direct reply flow: draft -> queue

| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |
|------|-------------|---------|--------------|-------|--------|
| draft_replies_for_candidates | 5 | PASS | PASS | - | - |
| propose_and_queue_replies | 4 | PASS | PASS | - | allow |

### Scenario B: Composite flow: find -> draft -> queue

| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |
|------|-------------|---------|--------------|-------|--------|
| find_reply_opportunities | 2 | PASS | PASS | - | - |
| draft_replies_for_candidates | 5 | PASS | PASS | - | - |
| propose_and_queue_replies | 3 | PASS | PASS | - | allow |

### Scenario C: Blocked-by-policy mutation with telemetry verification

| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |
|------|-------------|---------|--------------|-------|--------|
| propose_and_queue_replies | 0 | FAIL | PASS | policy_denied_blocked | deny |
| get_mcp_error_breakdown | 0 | PASS | PASS | - | - |

## Quality Gates

| Gate | Rate | Threshold | Status |
|------|------|-----------|--------|
| Schema validation | 100.0% | 95% | PASS |
| Unknown errors | 0.0% | 5% | PASS |

**Overall: ALL GATES PASS**
