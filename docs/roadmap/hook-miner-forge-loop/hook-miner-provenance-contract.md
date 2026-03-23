# Hook Miner Provenance Contract

**Date:** 2026-03-22
**Session:** 05
**Status:** Active

---

## Overview

Hook Miner provenance extends the existing `vault_provenance_links` table with four additive nullable columns that capture angle attribution metadata. These fields enable downstream systems (Forge sync, analytics aggregation) to trace generated content back to the specific mined angle and supporting evidence.

## New Columns

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `angle_kind` | TEXT | Yes | The angle type that produced this content (e.g., `"story"`, `"listicle"`, `"hot_take"`, `"contrarian_take"`, `"data_driven"`). Set on **all** provenance links for an angle-sourced draft. |
| `signal_kind` | TEXT | Yes | The evidence type extracted from the source note (e.g., `"contradiction"`, `"data_point"`, `"aha_moment"`). Only set on links that correspond to evidence items. |
| `signal_text` | TEXT | Yes | The citation text from the evidence item. Only set on links that have matching evidence. |
| `source_role` | TEXT | Yes | Distinguishes the user's primary selection from accepted neighbor notes. Exactly two values: `"primary_selection"` and `"accepted_neighbor"`. |

## Backward Compatibility

- All four columns are nullable with no defaults. Existing rows retain `NULL` for all new fields.
- The Rust `ProvenanceRef` struct uses `#[serde(default)]` on all new fields. Deserializing old JSON payloads (without the new keys) produces `Option::None`.
- The TypeScript `ProvenanceRef` interface uses optional (`?`) fields. Old API responses missing these fields are handled transparently.
- `copy_links_for` copies all four new columns alongside existing ones. Legacy rows with `NULL` values copy as `NULL`.

## `source_role` Values

| Value | Meaning | Forge Implication |
|-------|---------|-------------------|
| `primary_selection` | The note the user originally selected in Obsidian | Forge writes performance data back to this note |
| `accepted_neighbor` | A related note the user accepted from graph suggestions | Supporting context; does not receive writeback |
| `NULL` | Legacy row or non-vault-sourced content | No Forge writeback target |

## Field Semantics

### `angle_kind`

Set on **every** provenance link for an angle-sourced draft. This is a property of the generation, not of individual source notes. When a user selects a "story" angle, all provenance links (primary + neighbors) get `angle_kind = "story"`.

For hook-based (non-angle) generation, `angle_kind` is `NULL`.

### `signal_kind` and `signal_text`

Set only on provenance links that correspond to evidence items extracted by the angle mining pipeline. The primary selection note and neighbors without extracted evidence keep these `NULL`.

Example: If the "story" angle extracted a `"data_point"` evidence item from neighbor note #7 with citation `"Revenue grew 3x"`, then:
- The provenance link for note #7 gets `signal_kind = "data_point"`, `signal_text = "Revenue grew 3x"`
- The provenance link for the primary selection gets `signal_kind = NULL`, `signal_text = NULL`

## Query Patterns

### Find the angle type for a draft

```sql
SELECT DISTINCT angle_kind
FROM vault_provenance_links
WHERE account_id = ? AND entity_type = 'scheduled_content' AND entity_id = ?
  AND angle_kind IS NOT NULL;
```

### Find the primary note for Forge writeback

```sql
SELECT source_path
FROM vault_provenance_links
WHERE account_id = ? AND entity_type = 'original_tweet' AND entity_id = ?
  AND source_role = 'primary_selection';
```

### Aggregate evidence types by angle performance

```sql
SELECT angle_kind, signal_kind, COUNT(*) as usage_count
FROM vault_provenance_links
WHERE account_id = ? AND angle_kind IS NOT NULL
GROUP BY angle_kind, signal_kind;
```

## Data Flow

```
User selects angle in AngleCards
        │
        ▼
VaultSelectionReview.handleAngleSelected()
  → Sets angle_kind, source_role on all neighborProv
  → Sets signal_kind, signal_text on evidence-matched neighbors
        │
        ▼
ComposerInspector.handleGenerateFromVault()
  → Maps nodeIds to ProvenanceRef[]
  → Primary selection: source_role = 'primary_selection', angle_kind = hookStyle
  → Neighbors: source_role, angle_kind, signal_kind, signal_text from neighborProv
        │
        ▼
POST /api/content/drafts or /api/content/compose
  → body.provenance carries enriched ProvenanceRef[]
        │
        ▼
insert_links_for() writes to vault_provenance_links
  → All 15 columns (11 existing + 4 new) persisted
        │
        ▼
copy_links_for() preserves all fields on entity transitions
  (scheduled_content → approval_queue → original_tweet)
```

## Migration

**File:** `crates/tuitbot-core/migrations/20260322000100_provenance_hook_miner_fields.sql`

Four `ALTER TABLE ADD COLUMN` statements. No data migration needed — existing rows naturally have `NULL` for new columns.
