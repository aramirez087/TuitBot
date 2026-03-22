# Draft Insertion Model

**Date:** 2026-03-21
**Session:** 5 — Draft Insertion & Suggestion Controls

## Overview

The draft insertion model allows users to accept related-note suggestions into specific slots of an existing draft without full regeneration. Each insertion is visible, reversible, and carries provenance.

## DraftInsert Type

```typescript
interface DraftInsert {
  id: string;            // Unique insert action ID
  blockId: string;       // Stable: ThreadBlock.id or 'tweet'
  slotLabel: string;     // Human-readable: "Opening hook", "Tweet 2", "Closing takeaway"
  previousText: string;  // For undo restoration
  insertedText: string;  // The refined text after applying the suggestion
  sourceNodeId: number;  // The neighbor node that sourced this
  sourceTitle: string;   // Display title of the source note
  provenance: ProvenanceRef;  // edge_type + edge_label
  timestamp: number;     // ms since epoch
}
```

## Slot Labeling Convention

Slots are identified by `blockId` (stable across reorders) with human-readable labels derived at render time:

| Position | Total | Label |
|----------|-------|-------|
| 0 | 1 | "Tweet" |
| 0 | N>1 | "Opening hook" |
| i | N>1 | "Tweet {i+1}" |
| N-1 | N>1 | "Closing takeaway" |

## Undo Stack Behavior

- **Structure:** LIFO (last-in, first-out) append-only history
- **Granularity:** Per-insert undo — each suggestion acceptance is a separate entry
- **Priority:** Insert undo takes precedence over workspace-level undo (full snapshot restore)
- **Mechanism:** `popInsert()` removes the most recent entry and restores `previousText` to the target block
- **Targeted undo:** `undoInsertById()` removes a specific insert, restoring that block's text
- **Immutability:** All state transitions produce new objects; no mutation of existing state

## How Inserts Differ from Full Regeneration

| Aspect | Full Regeneration | Slot Insert |
|--------|-------------------|-------------|
| Scope | Replaces entire draft | Modifies one block |
| Trigger | Hook selection + "Use this hook" | "Apply" on SlotTargetPanel |
| LLM call | `api.assist.thread()` or `api.assist.tweet()` | `api.assist.improve()` with slot context |
| Undo | Workspace snapshot restore | Per-insert stack pop |
| Provenance | All refs from generation | Incremental ref addition |

## Provenance Flow

1. User accepts a neighbor suggestion in `SlotTargetPanel`
2. `ComposerInspector.handleSlotInsert()` calls `api.assist.improve()` with the slot's current text + neighbor context
3. A `DraftInsert` is created with `provenance: { node_id, edge_type, edge_label }` from the neighbor
4. The provenance ref is appended to `vaultProvenance[]`
5. At submission time, `buildComposeRequest()` includes all provenance refs in `ComposeRequest.provenance`

## Citation Chip Visual Model

- **Primary citations** ("Based on:"): Blue accent, `FileText` icon — from the original note selection
- **Graph-sourced citations** ("Related notes:"): Purple accent, `Link` icon — from accepted neighbor inserts
- Each graph citation shows the source note title and target slot label
- Undo button on each graph citation removes the insert and restores the slot

## Thread Flow Card Badges

When a thread block has active inserts, small badges appear below the textarea:
- Each badge shows the source note title with a `Link` icon
- An undo button (`Undo2` icon) allows reverting that specific insert
- Badges use the accent color for visual consistency

## Privacy Implications

None. Inserts use the same data already available through the vault selection and graph neighbor APIs. No new data is fetched or exposed. The `api.assist.improve()` call sends the same content the user already sees in their draft.

## Component Architecture

```
ComposeWorkspace
  └── ComposerInspector (owns draftInsertState, handleSlotInsert, handleUndoInsert)
        └── InspectorContent (passes props through)
              └── FromVaultPanel (passes props through)
                    └── VaultSelectionReview (renders SlotTargetPanel when draft exists)
                          └── SlotTargetPanel (slot selector, apply/undo UI)
```

Insert indicators flow separately through the editor:
```
ComposeWorkspace
  └── ComposerCanvas
        └── ThreadFlowLane → ThreadFlowCard (inserts prop, badge rendering)
        └── CitationChips (graphInserts prop, graph citation section)
```
