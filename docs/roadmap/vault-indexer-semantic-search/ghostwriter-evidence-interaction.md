# Ghostwriter Evidence Interaction Model

## Overview

The Evidence Rail adds semantic evidence search to the Ghostwriter composer sidebar. Users search their vault by concept, pin relevant evidence, and feed it into draft generation — all without leaving the compose flow.

## Interaction Cycle

1. **Search** — User types a query or enables auto-query from draft text
2. **Review** — Evidence cards appear with match reason badges (Semantic, Keyword, Graph, Hybrid), snippet previews, and relevance scores
3. **Pin** — User pins up to 5 relevant results for use in generation
4. **Generate** — Pinned evidence flows into the compose request as additional LLM context with provenance tracking

## Search Modes

### Manual Search
- User types a query in the search bar
- 300ms debounce before API call
- Uses `mode: 'hybrid'` (both semantic and keyword) with `limit: 8`
- Results persist until the next search or dismiss

### Auto-Query
- Enabled via sparkle toggle button in the search row
- Extracts text from the currently focused block (tweet text in tweet mode, focused thread block in thread mode)
- 800ms debounce to avoid excessive API calls during typing
- Uses `mode: 'semantic'` with `limit: 5`
- Each new query aborts the previous in-flight request via AbortController
- Results are marked as "Suggested" to distinguish from manual search
- Block-level text produces more focused matches than full-draft concatenation

## Deduplication

Results are filtered client-side to remove:
1. **Dismissed chunks** — chunks the user explicitly dismissed this session
2. **Already-pinned chunks** — chunks currently in the pinned set
3. **Graph neighbor chunks** — chunks already surfaced via VaultSelectionReview's accepted graph neighbors (prevents redundant display)

## Pinned Evidence Lifecycle

- **Max 5 pins** per session — enforced by `canPin()` guard
- **Session-scoped** — cleared when the composer closes (state lives in ComposerInspector, not a global store)
- **Ordered** — pins maintain insertion order
- **Export** — `ComposerInspector.getPinnedEvidence()` returns the pinned array for the parent to include in generation requests
- **Provenance** — when pinned evidence is applied to a slot, a `ProvenanceRef` with `source_role: 'semantic_evidence'` is recorded

## Apply to Slot

When the user clicks "Apply to slot" on a pinned evidence card:
1. The evidence snippet is sent to `api.assist.improve()` along with the current block text and context
2. The refined text replaces the targeted slot (tweet text or thread block)
3. A `DraftInsert` is pushed to the undo history for reversal
4. A `ProvenanceRef` is recorded with `match_reason`, `similarity_score`, and `source_role: 'semantic_evidence'`

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+E` | Toggle Evidence Rail collapse |
| `Escape` (in search) | Clear search and collapse |

## Degraded State Matrix

| Condition | Behavior |
|-----------|----------|
| No embedding provider configured | Rail hidden entirely |
| Index empty (0 chunks) | "Building index..." with progress bar |
| Provider unavailable at search time | Error message, results empty |
| Stale index (<50% fresh) | Amber warning: "Index may show outdated results" |
| No search results (manual) | "No matching evidence found" |
| No search results (auto-query) | Silent (no empty message for auto-suggestions) |
| API error during search | Red error message with error text |

## Index Status Badge

An 8px colored dot in the rail header indicates index health:
- **Green** — freshness >= 95%
- **Amber (pulsing)** — freshness 50-94%
- **Red** — freshness < 50%
- **Gray** — no provider or no index

Click the badge to see a popover with chunk counts, model info, and last indexed timestamp.

## Analytics Events

All events prefixed with `evidence.` for namespace isolation:

| Event | When |
|-------|------|
| `evidence.rail_opened` | Rail mounts with provider configured |
| `evidence.search_executed` | Manual or auto-query search completes |
| `evidence.pinned` | User pins a result |
| `evidence.dismissed` | User dismisses a result |
| `evidence.applied_to_slot` | Pinned evidence is applied to a draft slot |
| `evidence.auto_query_toggled` | Auto-query toggle flipped |
| `evidence.contributed_to_draft` | Generation completes with pinned evidence |

## Architecture

- **State management**: Pure functions in `evidenceStore.ts`, instantiated as `$state()` in ComposerInspector
- **No global store**: Evidence is session-scoped; no lifecycle cleanup needed
- **Component hierarchy**: ComposerInspector -> InspectorContent -> EvidenceRail -> (EvidenceCard, IndexStatusBadge)
- **API surface**: Consumes `GET /api/vault/evidence` and `GET /api/vault/index-status` (built in Session 3)
