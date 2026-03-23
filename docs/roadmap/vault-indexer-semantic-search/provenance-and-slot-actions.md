# Provenance and Slot Actions — Design Document

## Overview

Session 5 connects semantic evidence to tweet and thread editing via slot-targeted apply, whole-draft strengthen, and traceable citation rendering with per-insert undo.

## Action Model

### Single-Slot Apply

A user can apply evidence to a specific draft slot:

1. **Tweet mode**: Click "Apply to draft" on an evidence card. Calls `api.assist.improve()` with evidence context. Creates one `DraftInsert` targeting the tweet block.
2. **Thread mode (focused)**: Click "Apply to draft" — defaults to the focused block index. Creates one `DraftInsert` for that block.
3. **Thread mode (slot picker)**: Click the apply dropdown to choose a specific slot (Opening hook, Tweet 2, ..., Closing takeaway). Creates one `DraftInsert` for the chosen slot.

Each apply creates exactly one `DraftInsert` with `source_role: 'semantic_evidence'`, preserving `previousText` for undo.

### Whole-Draft Strengthen

The "Strengthen draft" button in the pinned evidence section:

1. Collects all pinned evidence snippets as context.
2. For tweet mode: calls `api.assist.improve()` once, creates one `DraftInsert`.
3. For thread mode: iterates non-empty blocks sequentially, calling `api.assist.improve()` per block. Creates one `DraftInsert` per block — each individually undoable.
4. Adds `ProvenanceRef` entries for all pinned evidence used.

Sequential API calls for thread mode ensure each block gets the full evidence context. Partial failure stops iteration but preserves already-completed inserts (each has its own undo).

### Undo Semantics

- **Per-insert undo**: Each `DraftInsert` can be undone individually via `undoInsertById()`. Restores the block to `previousText`.
- **Most-recent undo**: `popInsert()` removes the last insert (used by keyboard shortcut and undo banner).
- **Citation chip undo**: Each evidence chip in the citation strip has an undo button that calls `undoInsertById()`.
- **Strengthen undo**: Each block modified by strengthen has its own insert — undoing one doesn't affect others.

## Provenance Chain

Evidence metadata flows through the system:

```
PinnedEvidence (EvidenceRail)
  → handleApplyEvidence / handleStrengthenDraft (ComposerInspector)
    → buildInsert({ ..., matchReason, similarityScore, chunkId, sourceRole, headingPath, snippet })
      → DraftInsert.provenance: ProvenanceRef { source_role: 'semantic_evidence', match_reason, similarity_score, chunk_id, ... }
        → getDraftInsertState() at submit time
          → buildComposeRequest() includes provenance
            → Backend: approval_queue::enqueue_with_provenance_for()
```

### ProvenanceRef Fields for Evidence

| Field | Source | Purpose |
|-------|--------|---------|
| `node_id` | `PinnedEvidence.node_id` | Source vault note |
| `chunk_id` | `PinnedEvidence.chunk_id` | Specific chunk within note |
| `match_reason` | `PinnedEvidence.match_reason` | How evidence was found (semantic, keyword, graph, hybrid) |
| `similarity_score` | `PinnedEvidence.score` | Relevance score (0-1) |
| `source_role` | `'semantic_evidence'` | Distinguishes from graph neighbor inserts |
| `heading_path` | `PinnedEvidence.heading_path` | Section within the note |
| `snippet` | `PinnedEvidence.snippet` | Text excerpt used |

## Citation Rendering

CitationChips renders three strips:

1. **"Based on:"** (blue) — Vault citations from generation. Expandable chips with heading path and snippet.
2. **"Related notes:"** (purple/graph) — Graph neighbor inserts with undo buttons.
3. **"Evidence used:"** (purple/evidence #a855f7) — Semantic evidence inserts with undo buttons.

The third strip is new in Session 5. It uses `partitionInserts()` to split `DraftInsertState.history` by `provenance.source_role === 'semantic_evidence'`.

## Slot Picker UX

In thread mode, evidence cards show a dropdown chevron on the apply button:

- Clicking opens a menu listing all thread slots by label (from `getSlotLabel()`).
- Selecting a slot applies evidence directly to that block.
- In tweet mode, the apply button fires immediately without a dropdown (single slot).

## Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Slot picker dropdown on evidence cards in thread mode | Lighter than a full panel; matches existing `getSlotLabel()` patterns |
| D2 | "Strengthen draft" as batch operation with per-block inserts | Per-block inserts preserve undo granularity |
| D3 | Purple evidence strip (#a855f7) in CitationChips | Distinguishes from graph (purple #9b59b6) and vault (blue accent) |
| D4 | focusedBlockIndex wired through full component chain | Enables precise auto-query and correct default slot selection |
| D5 | Enriched buildInsert with evidence-specific fields | All new params optional — backward compatible with existing callers |
| D6 | No backend changes — all frontend-only | Existing ProvenanceRef type already supports all needed fields |
