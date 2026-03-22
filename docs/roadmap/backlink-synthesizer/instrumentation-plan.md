# Instrumentation Plan — Backlink Synthesizer

## Event Catalog

All events are namespaced under `backlink.*` and emitted via `trackFunnel()` in `dashboard/src/lib/analytics/backlinkFunnel.ts`.

| Event | Fired When | Key Properties | Component |
|-------|-----------|----------------|-----------|
| `backlink.suggestions_shown` | Graph neighbors render (once per session) | `count`, `session_id`, `graph_state` | GraphSuggestionCards |
| `backlink.empty_graph` | Empty state renders | `graph_state`, `session_id` | GraphSuggestionCards |
| `backlink.suggestion_accepted` | User clicks "Include" on a card | `node_id`, `intent`, `session_id` | VaultSelectionReview |
| `backlink.suggestion_dismissed` | User clicks "Skip" on a card | `node_id`, `session_id` | VaultSelectionReview |
| `backlink.suggestion_restored` | User restores a skipped card | `node_id`, `session_id` | VaultSelectionReview |
| `backlink.synthesis_toggled` | User toggles "Use related notes" | `enabled`, `session_id` | VaultSelectionReview |
| `backlink.hooks_generated` | Hook options are generated | `count`, `session_id` | VaultSelectionReview |
| `backlink.slot_targeted` | User applies a neighbor to a draft slot | `slot_label`, `source_node_id`, `session_id` | ComposerInspector |
| `backlink.insert_undone` | User undoes a slot insert | `insert_id`, `slot_label`, `session_id` | ComposerInspector |
| `backlink.citation_clicked` | User expands or opens a citation chip | `node_id`, `action` (`expand`/`open_obsidian`) | CitationChips |
| `backlink.draft_completed` | Draft submitted with backlink provenance | `insert_count`, `provenance_count`, `mode`, `session_id` | ComposeWorkspace |

## Funnel Shape

```
suggestions_shown (or empty_graph)
  └─ suggestion_accepted / suggestion_dismissed
       └─ synthesis_toggled (optional)
            └─ hooks_generated (optional)
                 └─ slot_targeted (optional, repeatable)
                      └─ insert_undone (optional)
                           └─ draft_completed
```

## Success Metrics

| Metric | Definition | Target |
|--------|-----------|--------|
| **Suggestion acceptance rate** | `accepted / (accepted + dismissed)` | ≥ 40% |
| **Synthesis opt-in rate** | `toggled_on / suggestions_shown` | ≥ 60% |
| **Slot refinement rate** | `slot_targeted / draft_completed` (among sessions with suggestions) | ≥ 20% |
| **Undo rate** | `insert_undone / slot_targeted` | ≤ 30% (lower = better quality) |
| **End-to-end conversion** | `draft_completed / suggestions_shown` | ≥ 25% |

## Backend Relay

### Endpoint
`POST /api/telemetry/events`

### Schema
```json
{
  "events": [
    {
      "event": "backlink.suggestion_accepted",
      "properties": { "node_id": 42, "session_id": "abc-123" },
      "timestamp": "2026-03-21T12:00:00Z"
    }
  ]
}
```

### Constraints
- Max 50 events per batch
- Event names must start with `backlink.`
- Returns 204 No Content on success
- Currently logs via `tracing::info!` — future: persist to analytics store

### Frontend Buffering
`backlinkFunnel.ts` provides `bufferEvent()` / `flushToBackend()` for optional batched relay. Currently disabled (console-only). Enable by calling `flushToBackend()` on page unload or at periodic intervals.
