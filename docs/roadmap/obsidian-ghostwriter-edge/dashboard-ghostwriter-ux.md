# Dashboard Ghostwriter UX — Block Selection, Ingress, Citations

## Overview

Session 4 transforms the vault panel from a whole-note search tool into a true Ghostwriter workspace. Users can now receive exact block selections from Obsidian, review them before generation, and get heading-level deep-links in citations.

## Architecture

### Ingress Flow

```
Obsidian Plugin
  ↓ POST /api/vault/send-selection
  ↓ (server stores in vault_selections, emits SelectionReceived WS event,
  ↓  returns composer_url: "/compose?selection=<uuid>")
  ↓
Dashboard receives selection via:
  Path A: URL param ?selection=<session_id>  (user clicks link from Obsidian)
  Path B: SelectionReceived WebSocket event  (dashboard already open)
  ↓
DraftStudioShell parses param / subscribes to WS
  ↓ selectionSessionId prop
DraftStudioComposerZone → ComposeWorkspace
  ↓ auto-opens inspector + vault panel
ComposerInspector → InspectorContent → FromVaultPanel
  ↓ fetches GET /api/vault/selection/{session_id}
  ↓ enters selection review state
User sees source metadata + text preview
  ↓ clicks "Generate from selection"
Generation pipeline runs with provenance
```

### Component Prop Chain

```
DraftStudioShell
  └─ selectionSessionId, lastConsumedSelectionId (local state)
  └─ DraftStudioComposerZone
      └─ selectionSessionId, onSelectionConsumed (props)
      └─ ComposeWorkspace
          └─ selectionSessionId, onSelectionConsumed (props)
          └─ auto-opens vault panel via $effect
          └─ ComposerInspector
              └─ selectionSessionId, onSelectionConsumed (props)
              └─ InspectorContent
                  └─ selectionSessionId, onSelectionConsumed (props)
                  └─ selection-dot indicator on "From vault" button
                  └─ FromVaultPanel
                      └─ selectionSessionId, onSelectionConsumed (props)
                      └─ hydrateSelection() fetches + populates review state
```

## Decisions

### Decision 1: Two ingress paths (URL param + WebSocket)

URL param covers the primary flow where user clicks the composer URL from Obsidian. WebSocket covers the "dashboard already open" case. Both converge on the same `selectionSessionId` state in `DraftStudioShell`.

Deduplication via `lastConsumedSelectionId` prevents double-processing.

### Decision 2: `api.vault.getSelection()` client method

Added `getSelection(sessionId: string)` to the `api.vault` namespace. Returns `VaultSelectionResponse` type matching the `GET /api/vault/selection/{session_id}` endpoint from Session 3. The `selected_text` field is `null` in Cloud mode (privacy gate enforced server-side).

### Decision 3: Selection hydration as review state

When a selection arrives, FromVaultPanel enters a "selection review" state showing source metadata, heading context, selected text (when available), and frontmatter tags. User must explicitly click "Generate from selection" to proceed.

This is better than auto-generating because:
- Power users want to see what's being used
- Cloud mode may not have text — review state shows metadata-only gracefully
- Users can switch format (tweet/thread) before generation

### Decision 4: ProvenanceRef construction at generation time

When generating from a selection, `resolved_node_id` and `resolved_chunk_id` from the selection response are used to construct `ProvenanceRef`. These flow through the existing compose/draft provenance pipeline — no new pipeline needed.

### Decision 5: Heading-anchor deep-links in citations

`buildObsidianUri()` now accepts an optional `heading` parameter. The deepest heading segment is extracted (e.g., "Overview > Strategy" → "Strategy") and appended as a `#fragment`. Obsidian supports heading fragment links in `obsidian://open` URIs.

### Decision 6: WebSocket store `SelectionReceived` event

Added to the `WsEvent.type` union. `DraftStudioShell` subscribes via `$effect` on the `$wsEvents` store. The event carries `session_id` which is used to fetch the selection.

## Cloud Mode Degradation

In Cloud mode, `selected_text` is `null` in the GET response (privacy gate from Session 3). The selection review state handles this:
- Shows file path + heading context as source preview
- Shows "Text not shown in cloud mode for privacy" note
- CTA says "Generate from selection" (works via resolved IDs)
- Generation uses `resolved_node_id` / `resolved_chunk_id` to retrieve context server-side

## CTA Copy Changes

| Context | Old Label | New Label |
|---|---|---|
| Manual chunk selection | Extract Highlights | Extract key points |
| Selection from Obsidian | — | Generate from selection |
| Selection mode footer | — | Generate from selection |
| Generating state | Extracting... | Generating... |

## Selection TTL Handling

Selections expire after 30 minutes (server-enforced). If the fetch returns 404:
- `selectionExpired` flag set
- Shows "Selection expired" message
- Offers "Browse vault" button to fall back to normal search
- `onSelectionConsumed()` still called to clean up parent state
