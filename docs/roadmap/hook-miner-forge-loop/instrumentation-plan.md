# Instrumentation Plan — Hook Miner & Forge

**Date:** 2026-03-22
**Purpose:** Define how to measure adoption, fallback frequency, and sync reliability for Hook Miner and Forge after release.

---

## Event Catalog

All events are logged via `tracing::info!` on the backend (structured logging only, no DB writes). Frontend events fire through `trackFunnel()` and relay via `POST /api/telemetry/events`.

### Hook Miner Namespace (`hook_miner.`)

| Event | When It Fires | Properties |
|-------|---------------|------------|
| `hook_miner.angles_shown` | Angle cards rendered to user after mining | `angle_count`, `session_id`, `source_path_stem`, `local_eligible` |
| `hook_miner.angle_selected` | User clicks/selects an angle card | `angle_kind`, `session_id`, `source_path_stem`, `evidence_count` |
| `hook_miner.fallback_opened` | Fallback path triggered (weak signal, no neighbors, etc.) | `reason`, `session_id`, `accepted_count` |

### Forge Namespace (`forge.`)

| Event | When It Fires | Properties |
|-------|---------------|------------|
| `forge.prompt_shown` | Analytics sync consent banner shown on Activity page | `source_path_stem`, `local_eligible` |
| `forge.enabled` | User enables analytics sync (from prompt or settings) | `source_path_stem`, `enabled_from` |
| `forge.sync_succeeded` | Forge sync completes without error | `tweets_synced`, `threads_synced`, `entries_not_found`, `files_not_found` |
| `forge.sync_failed` | Forge sync fails | `reason`, `stage` |

---

## Property Reference

| Property | Type | Description | Privacy |
|----------|------|-------------|---------|
| `angle_count` | number | Number of angle cards shown | Safe |
| `angle_kind` | string | Category of angle (e.g., "contradiction", "data_point") | Safe — categorical, not content |
| `session_id` | string | Frontend session identifier | Safe — opaque ID |
| `source_path_stem` | string | Filename without extension or directory path | Safe — `sanitizePathStem()` strips dirs |
| `local_eligible` | boolean | Whether source is local filesystem (not cloud) | Safe |
| `evidence_count` | number | Number of evidence items on selected card | Safe |
| `reason` | string | Categorical reason for fallback or failure | Safe — enum-like, not raw error |
| `accepted_count` | number | Number of accepted neighbors before fallback | Safe |
| `enabled_from` | string | Where user enabled sync: "prompt" or "settings" | Safe |
| `tweets_synced` | number | Count of tweets with analytics written | Safe |
| `threads_synced` | number | Count of threads with analytics written | Safe |
| `entries_not_found` | number | DB entries without matching file | Safe |
| `files_not_found` | number | Files that couldn't be located on disk | Safe |
| `stage` | string | Sync stage where failure occurred | Safe — categorical |

---

## Adoption Metrics

### Hook Miner Adoption

**Selection rate** = `angle_selected` / `angles_shown`

Query structured logs for a time window:
```
event="hook_miner.angles_shown" | count() as shown
event="hook_miner.angle_selected" | count() as selected
```

Segment by `angle_kind` to find which angle types are most selected:
```
event="hook_miner.angle_selected" | stats count() by angle_kind
```

Segment by `local_eligible` to compare local-vault vs non-local usage:
```
event="hook_miner.angles_shown" | stats count() by local_eligible
```

### Forge Adoption

**Enable rate** = `forge.enabled` / `forge.prompt_shown`

Segment by `enabled_from` to see which entry point drives adoption:
```
event="forge.enabled" | stats count() by enabled_from
```

---

## Fallback Frequency

**Fallback ratio** = `fallback_opened` / (`angles_shown` + `fallback_opened`)

Breakdown by reason:
```
event="hook_miner.fallback_opened" | stats count() by reason
```

Expected reasons:
- `weak_signal` — neighbors accepted but signal quality below threshold
- `no_neighbors` — no graph neighbors available
- `synthesis_disabled` — user has synthesis toggled off
- `api_error` — angle mining API call failed
- `timeout` — angle mining exceeded time budget

A high `weak_signal` ratio suggests the mining threshold is too strict. A high `no_neighbors` ratio suggests users need better graph onboarding.

---

## Sync Reliability

**Success rate** = `sync_succeeded` / (`sync_succeeded` + `sync_failed`)

Failure breakdown:
```
event="forge.sync_failed" | stats count() by stage, reason
```

Volume tracking:
```
event="forge.sync_succeeded" | stats sum(tweets_synced), sum(threads_synced)
```

Data quality indicators from succeeded events:
- High `entries_not_found` → analytics DB has tweets we can't match to files
- High `files_not_found` → frontmatter references files that moved or were deleted

---

## Privacy Audit Checklist

Before release, verify:

- [ ] No event property contains raw note content, file body, or frontmatter values
- [ ] `source_path_stem` is always a filename stem (no directory components)
- [ ] Error `reason` values are categorical strings, not stack traces or raw error messages
- [ ] `session_id` is an opaque identifier, not derived from user data
- [ ] Backend only logs events — no DB persistence of telemetry data
- [ ] Telemetry is best-effort — failures don't block user workflows
- [ ] `sanitizePathStem()` is called on every path before inclusion in events

---

## Log Query Examples (Generic Tracing Format)

These examples use a generic structured-log query syntax. Adapt to your aggregation tool (Loki, CloudWatch Logs Insights, Datadog, etc.).

### Daily Hook Miner funnel
```
telemetry_event event=~"hook_miner.*"
| stats count() by event, bin(1d)
```

### Weekly Forge sync health
```
telemetry_event event=~"forge.sync_*"
| stats count() by event, bin(1w)
```

### Top fallback reasons (last 7 days)
```
telemetry_event event="hook_miner.fallback_opened"
| filter @timestamp > ago(7d)
| stats count() by properties.reason
| sort count desc
```

### Forge adoption funnel (prompt → enable)
```
telemetry_event event=~"forge.(prompt_shown|enabled)"
| stats count() by event
```
