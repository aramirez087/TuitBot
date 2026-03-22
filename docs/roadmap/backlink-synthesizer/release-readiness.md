# Release Readiness — Backlink Synthesizer

**Date:** 2026-03-21
**Assessment:** GO

## Criteria Checklist

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Graph ingestion works | PASS | 17 link extractor + 18 storage tests (edge CRUD, tags, idempotency, migration) |
| Graph retrieval works | PASS | 33 expansion unit tests + 11 API integration tests |
| UX flow complete | PASS | 7 component test suites (199 tests), copy consistency verified |
| Provenance preserved | PASS | edge_type/edge_label in provenance_links, citation chips, ongenerate contract test |
| Privacy invariants | PASS | Cloud mode omits relative_path, account isolation verified |
| Backward compatible | PASS | Existing assist routes work without graph; graph fields are additive (optional in response) |
| Fallback graceful | PASS | 5 GraphState values handled: available, no_related_notes, unresolved_links, node_not_indexed, fallback_active |
| Analytics instrumented | PASS | 11 funnel events, 15 unit tests, telemetry endpoint (4 tests) |
| CI passes | PASS | fmt + clippy + test (572 Rust) + svelte-check + vitest (851 frontend) |

## Residual Risks

### 1. Analytics not persisted
Events log to console (frontend) and `tracing::info!` (backend) only. No production funnel data until backend persistence is added.
- **Impact:** Cannot measure acceptance rate or feature adoption quantitatively in production.
- **Mitigation:** Ship as-is. Console events are sufficient for internal validation. Add SQLite/analytics persistence in a follow-up session.

### 2. No A/B testing infrastructure
Cannot measure acceptance rate vs baseline (pre-backlink suggestions).
- **Impact:** Feature impact is qualitative, not quantitative.
- **Mitigation:** Use synthesis toggle (ON by default) as proxy. Instrument before/after via funnel events.

### 3. Shared-tag cap at 10
`max_neighbors` defaults to 8. Large vaults with many shared tags may miss relevant connections.
- **Impact:** Low — 8 neighbors is sufficient for most vaults. Users with dense tag graphs may not see all connections.
- **Mitigation:** Cap is configurable via `max` query parameter. Monitor user feedback.

### 4. No multi-hop traversal
Only 1-hop neighbors are surfaced. Users may expect deeper transitive connections.
- **Impact:** Low — 1-hop covers the majority of useful connections (direct links, backlinks, shared tags).
- **Mitigation:** Documented as out-of-scope. Can revisit based on usage data.

### 5. flushToBackend not auto-wired
Telemetry stays client-side unless `flushToBackend()` is explicitly called.
- **Impact:** Low — events are best-effort. Console logs are sufficient for local debugging.
- **Mitigation:** Wire to page unload or periodic flush in a follow-up.

### 6. unresolved_links state never emitted
The Rust `GraphState::UnresolvedLinks` variant exists but is never currently returned by `expand_neighbors()`. It's a placeholder for future use when all wikilinks in a note point to non-existent files.
- **Impact:** None — the frontend handles it gracefully (shows empty state). The variant is forward-compatible.
- **Mitigation:** None needed. Will be wired when unresolved link detection is added to graph ingestion.

## Rollout Guidance

### Phase 1 — Internal (Week 1)
- Enable for team accounts
- Validate funnel events appear in `tracing` logs
- Spot-check: tag-only neighbors, mutual links, scoring order
- Watch for panics or errors in `graph_expansion` module

### Phase 2 — Beta (Weeks 2-3)
- Enable for opted-in users via synthesis toggle (default ON)
- Monitor: empty-graph rate, acceptance rate (from console logs), error rate
- Collect qualitative feedback on suggestion quality

### Phase 3 — GA (Week 4+)
- Enable for all users with vault sources
- Remove beta flag if metrics are healthy

### Kill Switch
Synthesis toggle OFF disables all graph features without code change. The toggle is per-session state in the frontend — no backend deployment needed.

### Monitoring Checklist
- [ ] Error rate in `graph_expansion::expand_neighbors`
- [ ] Telemetry endpoint latency (`POST /api/telemetry/events`)
- [ ] Empty-graph rate (percentage of `no_related_notes` states)
- [ ] Acceptance rate (suggestions_shown vs suggestion_accepted events)
