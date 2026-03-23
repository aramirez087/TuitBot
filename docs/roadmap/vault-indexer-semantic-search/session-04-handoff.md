# Session 04 Handoff: Ghostwriter Evidence Panel

## What Changed

Added the Evidence Rail — a semantic evidence search panel inside the existing Ghostwriter composer sidebar. Users can search their vault by concept, pin relevant evidence, and feed it into draft generation without leaving the compose flow.

### Files Created

| File | Purpose |
|------|---------|
| `dashboard/src/lib/stores/evidenceStore.ts` | Pure functions for evidence state: pin/unpin/dismiss/filter/toggle |
| `dashboard/src/lib/analytics/evidenceFunnel.ts` | Analytics event helpers (7 events, `evidence.*` namespace) |
| `dashboard/src/lib/components/composer/IndexStatusBadge.svelte` | 8px colored dot with tooltip/popover showing index health |
| `dashboard/src/lib/components/composer/EvidenceCard.svelte` | Single evidence result card with match reason badge, snippet, actions |
| `dashboard/src/lib/components/composer/EvidenceRail.svelte` | Main evidence panel: search bar, pinned section, result cards, auto-query, empty/degraded states |
| `dashboard/tests/unit/evidenceStore.test.ts` | 19 unit tests for evidence store pure functions |
| `dashboard/tests/unit/IndexStatusBadge.test.ts` | 8 component tests for badge color/tooltip/pulse |
| `dashboard/tests/unit/EvidenceRail.test.ts` | 9 component tests for visibility, search, pin, collapse, degraded states |
| `docs/roadmap/vault-indexer-semantic-search/ghostwriter-evidence-interaction.md` | Interaction model documentation |
| `docs/roadmap/vault-indexer-semantic-search/session-04-handoff.md` | This file |

### Files Modified

| File | Change |
|------|--------|
| `dashboard/src/lib/api/types.ts` | Re-exported `PinnedEvidence` from evidenceStore for import convenience |
| `dashboard/src/lib/components/composer/InspectorContent.svelte` | Added EvidenceRail section between Voice and AI; added evidence-related props |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Added evidence state management, `getPinnedEvidence()` export, `handleApplyEvidence()` handler with provenance tracking |

## Decisions Made

### D1: Evidence state lives in ComposerInspector, not a global store
Evidence is session-scoped (cleared when composer closes). A global store would require manual lifecycle cleanup. Pure function helpers in `evidenceStore.ts` handle logic; the component owns state.

### D2: EvidenceRail is a collapsible section, not a panel mode
Renders as an always-present collapsible section in InspectorContent, independent of `notesPanelMode`. Coexists with From Vault / From Notes rather than replacing them.

### D3: Auto-query seeds from focused block text, not full draft
Block-level text produces more focused semantic matches. Full-draft queries would return broad, unfocused results. 800ms debounce fires on the actively edited block.

### D4: Deduplication by chunk_id against graph neighbors
Client-side filter removes any `chunk_id` already in VaultSelectionReview's accepted neighbors. Graph neighbors take priority since they include richer relationship context.

### D5: Pinned evidence flows into generation via ComposerInspector export
`getPinnedEvidence()` follows the same pattern as `getVaultProvenance()` and `getVaultHookStyle()`. Parent orchestrates what goes into generation; ComposerInspector tracks state.

### D6: No backend changes needed
All Session 4 work is frontend-only. The `/api/vault/evidence` and `/api/vault/index-status` endpoints from Session 3 are consumed as-is.

### D7: Auto-query uses AbortController for cancellation
Each debounce creates an AbortController. New debounce aborts previous. Prevents stale results from appearing after the user moves on.

## Exit Criteria Status

- [x] User can find relevant support from the sidebar without losing their place in compose
- [x] Automatic query seeding feels helpful instead of surprising (opt-in toggle, block-level focus, 800ms debounce)
- [x] Empty and degraded states preserve trust and current workflow continuity (5 states handled)
- [x] All frontend tests pass (1076 tests, 55 files, 0 failures)
- [x] All Rust CI passes (fmt, clippy, test — no regressions)
- [x] svelte-check passes with 0 errors

## Residual Risks

| Risk | Severity | Status |
|------|----------|--------|
| `focusedBlockIndex` not wired from ComposeWorkspace | Low | Auto-query falls back to first thread block or tweetText. Full focus tracking can be added when ComposeWorkspace gains block focus events. |
| Reindex button shown but disabled | Low | Wired as disabled with tooltip "Reindex available in a future update". Reindex API is Session 6. |
| Pinned evidence not yet consumed by generation endpoint | Low | `getPinnedEvidence()` is exported and ready. Session 5 will integrate with the compose request's context field. |
| `graphNeighborChunkIds` set not populated | Low | Currently empty set when no active vault selection. Requires building the set from accepted neighbors when VaultSelectionReview is active — can be wired in Session 5. |

## What Session 5 Needs

1. **Generation integration**: Read `getPinnedEvidence()` from ComposeWorkspace when building the compose request. Include pinned snippets + chunk_ids in the LLM context payload.
2. **Citation rendering**: Display provenance links with `source_role: 'semantic_evidence'` using heading-anchor deep-links in the draft preview and DraftStudio.
3. **Cleanup task registration**: Wire `vault_selections::cleanup_expired()` into the existing hourly cleanup loop.
4. **focusedBlockIndex wiring**: Add block focus tracking from ComposeWorkspace to ComposerInspector for precise auto-query targeting in thread mode.
5. **graphNeighborChunkIds population**: Build the dedup set from VaultSelectionReview's accepted neighbors when a selection session is active.
