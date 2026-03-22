# Ghostwriter Entry Flow: Interaction Model

## State Machine

```
                    ┌──────────────┐
                    │  Selection   │
                    │   Loading    │
                    └──────┬───────┘
                           │
                    fetch selection
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────▼────┐ ┌────▼─────┐ ┌───▼────────┐
        │ Expired  │ │ Selection│ │  Selection  │
        │          │ │ Review   │ │  Review +   │
        └──────────┘ │ (no graph)│ │  Graph Cards│
                     └────┬──────┘ └──┬──────────┘
                          │           │
                    Generate hooks    │ accept/dismiss/toggle
                          │           │
                     ┌────▼───────────▼──┐
                     │   Hook Picker     │
                     └────┬──────────────┘
                          │
                    select hook + confirm
                          │
                     ┌────▼──────────┐
                     │   Compose     │
                     │   Workspace   │
                     └───────────────┘
```

## UI States

### 1. Selection Loading
- **Visual:** Shimmer bar + "Loading selection..."
- **When:** `getSelection()` in flight

### 2. Selection Expired
- **Visual:** FileText icon + "Selection expired." + hint + "Browse vault" button
- **When:** `getSelection()` rejects (session expired or not found)

### 3. Selection Review (no graph)
- **Visual:** Source meta, heading, text preview, tags, VaultFooter
- **When:** Selection loaded, `graph_state` is `fallback_active` or undefined
- **Graph section:** Hidden (no toggle shown)

### 4. Selection Review + Graph Cards
- **Visual:** Source meta, heading, text preview, tags, **toggle**, **GraphSuggestionCards**, VaultFooter
- **When:** Selection loaded with `graph_state` !== `fallback_active`

### 5. Graph Sub-States

| `graph_state` | UI | Copy |
|---|---|---|
| `available` (neighbors present) | Suggestion cards with reason badges | "Related notes" header + count |
| `available` (empty) | Empty message | "No linked notes found — using this note only." |
| `no_related_notes` | Empty message | "No linked notes found — using this note only." |
| `node_not_indexed` | Info message | "This note isn't indexed yet. Related notes will appear after your next vault sync." |
| `fallback_active` | Nothing rendered | Silent fallback to standard behavior |

## Suggestion Card Anatomy

```
┌────────────────────────────────────────────┐
│ Async Patterns                          [×]│  ← title + dismiss
│ Async patterns in Rust use tokio for...    │  ← snippet (120 char max)
│ 💡 linked note                          ●  │  ← reason badge + score dot
│ [Use as pro-tip]                           │  ← intent-based action
└────────────────────────────────────────────┘
```

### Reason badges
- `linked_note` → Lightbulb icon + "linked note"
- `backlink` → Link icon + "backlink"
- `mutual_link` → ArrowLeftRight icon + "mutual link"
- `shared_tag` → BookOpen icon + "shared tag: #tag"

### Intent → Action mapping
| Intent | Button label |
|---|---|
| `pro_tip` | "Use as pro-tip" |
| `evidence` | "Use as example" |
| `counterpoint` | "Use as counterpoint" |
| `related` | "Use as context" |

## Accept / Dismiss Behavior

- **Accept:** Adds neighbor to `acceptedNeighbors` map with role. Shows "N related notes will be included" summary. Node ID is merged into `ongenerate` call alongside the primary selection node.
- **Dismiss:** Adds node to `dismissedNodeIds` set. Card disappears from visible list. Persists across hook regeneration within the same session. Also removes from accepted if previously accepted.
- **Both are session-scoped** — reset when the component unmounts (new compose session).

## Synthesis Toggle

- Small pill button: "RELATED NOTES ON" / "RELATED NOTES OFF"
- `aria-pressed` attribute for accessibility
- When OFF: GraphSuggestionCards hidden, accepted summary hidden, generate uses only primary selection node
- When ON: GraphSuggestionCards visible, accepted neighbors included in generation
- Only shown when `graph_state` !== `fallback_active`
- Session-scoped (defaults to ON each session)

## Provenance Chain

When a neighbor is accepted and generation proceeds:

1. `VaultSelectionReview` builds `neighborProvenance` array: `{ node_id, edge_type: reason, edge_label: reason_label }`
2. Passed through `ongenerate` → `FromVaultPanel` → `ComposerInspector.handleGenerateFromVault`
3. `handleGenerateFromVault` builds `ProvenanceRef[]` with `edge_type`/`edge_label` for neighbor nodes
4. `ProvenanceRef[]` stored as `vaultProvenance`, read by `ComposeWorkspace` when submitting
5. Backend `ProvenanceRef` → `ProvenanceLink` insertion handles `edge_type`/`edge_label` columns (Session 3)

## Backward Compatibility

- Selections without `graph_neighbors`/`graph_state` render identically to pre-Session 4 behavior
- `ProvenanceRef.edge_type` and `edge_label` are optional — existing provenance flows unchanged
- `ongenerate` signature extended with optional trailing parameter — all existing callers unaffected
- No new API endpoints consumed — all data comes from existing `getSelection()` response
