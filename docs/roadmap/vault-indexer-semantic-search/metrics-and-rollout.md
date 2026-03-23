# Metrics and Rollout Plan — Semantic Evidence Search

## Event Catalog

All events use the `evidence.` namespace prefix and flow through two channels:
1. **Frontend**: `console.info('[tuitbot:funnel]', ...)` via `trackFunnel()`
2. **Backend**: `POST /api/telemetry/events` via the evidence funnel relay buffer (flushes at 10 events)

### Event Definitions

| Event | Properties | Description |
|-------|-----------|-------------|
| `evidence.rail_opened` | `session_id`, `has_selection` | User opens Ghostwriter with Evidence Rail visible |
| `evidence.search_executed` | `query_length`, `result_count`, `mode` | Manual or auto-query search completes |
| `evidence.search_latency` | `latency_ms`, `mode`, `fallback` | Frontend-measured search round-trip time |
| `evidence.pinned` | `chunk_id`, `match_reason`, `score` | Evidence card pinned to workspace |
| `evidence.dismissed` | `chunk_id`, `match_reason` | Evidence card dismissed |
| `evidence.applied_to_slot` | `chunk_id`, `slot_index`, `slot_label`, `match_reason` | Pinned evidence applied to a draft slot |
| `evidence.auto_query_toggled` | `enabled` | Auto-suggestion toggle state changed |
| `evidence.contributed_to_draft` | `pinned_count`, `applied_count`, `session_id` | Session-level summary of evidence contribution |
| `evidence.draft_mode` | `mode`, `pinned_count`, `applied_count` | Draft mode with evidence usage stats |
| `evidence.strengthen_draft` | `block_count`, `pinned_count` | "Strengthen draft" invoked with N pinned items across M blocks |

### Backend Tracing Spans

| Span | Fields | Description |
|------|--------|-------------|
| `evidence_search_completed` | `latency_ms`, `fallback`, `mode`, `result_count` | Server-side search timing and fallback detection |

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Search latency (P50)** | < 50ms | `evidence.search_latency.latency_ms` P50 |
| **Search latency (P95)** | < 100ms | `evidence.search_latency.latency_ms` P95 |
| **Fallback rate** | < 5% | `evidence.search_latency` where `fallback=true` / total queries (when provider configured) |
| **Adoption** | Track baseline | % of Ghostwriter sessions where `evidence.rail_opened` fires |
| **Pin rate** | Track baseline | Avg `evidence.pinned` events per session |
| **Apply rate** | > 30% of pins | `evidence.applied_to_slot` / `evidence.pinned` |
| **Strengthen usage** | Track baseline | % of sessions with `evidence.strengthen_draft` |

## Rollout Plan

### Phase 1: Ship (Day 0)

- **Strategy**: Default-on, no feature flag
- **Rationale**: Semantic search is additive and fail-open. If no embedding provider is configured, the Evidence Rail is completely hidden. If the provider is down, keyword fallback is silent and non-disruptive.
- **Gate**: Evidence Rail renders only when `provider_configured === true`

### Phase 2: Monitor (Days 1–7)

- Monitor `evidence_search_completed` tracing spans for latency outliers
- Monitor `evidence.search_latency` frontend events for client-perceived latency
- Monitor fallback rate — alert if >10% with provider configured
- Watch for error spikes in `evidence.rs` tracing

### Phase 3: Review (Day 7)

- Pull adoption, pin rate, apply rate, and strengthen usage from telemetry
- Compare keyword-only vs hybrid vs semantic mode distribution
- Evaluate whether auto-query debounce (800ms) needs tuning based on latency data
- Decision point: tune, iterate, or mark stable

### Deployment Modes

| Mode | Embedding Support | Privacy Envelope |
|------|-------------------|------------------|
| Desktop | Ollama (local) or OpenAI (remote) | Vectors stored locally |
| Self-host | Ollama or OpenAI | Vectors stored on user's server |
| Cloud | OpenAI | Server-side processing, snippets truncated to 120 chars |

### Graceful Degradation

| Condition | Behavior |
|-----------|----------|
| No embedding provider | Evidence Rail hidden |
| Provider unreachable | Keyword fallback, "Provider unreachable" in badge |
| Stale index (<50%) | Amber badge, warning banner |
| Empty vault | Gray badge, "Building index..." with progress bar |
| Feature disabled by user | N/A — no toggle; rail hidden when provider not configured |
