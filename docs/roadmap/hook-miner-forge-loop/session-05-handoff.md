# Session 05 Handoff — Hook Miner Provenance & Draft Semantics

**Date:** 2026-03-22
**Session:** 05 of 11
**Status:** Complete

---

## What Changed

Additive provenance fields that snapshot Hook Miner attribution through the full draft lifecycle: creation, revision, publish, and later Forge sync.

| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-core/migrations/20260322000100_provenance_hook_miner_fields.sql` | Created | Migration adding `angle_kind`, `signal_kind`, `signal_text`, `source_role` to `vault_provenance_links` |
| `crates/tuitbot-core/src/storage/provenance.rs` | Modified | Extended `ProvenanceRef` and `ProvenanceLink` structs; updated `insert_links_for` and `copy_links_for` SQL; added 5 new tests |
| `crates/tuitbot-core/src/context/retrieval.rs` | Modified | Updated `citations_to_provenance_refs` to include new fields (all `None`) |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | Modified | Updated `publish_draft` `ProvenanceRef` reconstruction to include 4 new fields |
| `crates/tuitbot-server/src/routes/content/compose/tests/types.rs` | Modified | Added new fields to all `ProvenanceRef` test fixtures |
| `crates/tuitbot-server/src/routes/content/compose/tests/routing.rs` | Modified | Added new fields to `ProvenanceRef` test fixture |
| `crates/tuitbot-core/src/storage/scheduled_content/tests/provenance.rs` | Modified | Added new fields to `sample_provenance_refs` |
| `crates/tuitbot-core/src/storage/scheduled_content/tests/scheduling.rs` | Modified | Added new fields to `ProvenanceRef` test fixture |
| `dashboard/src/lib/api/types.ts` | Modified | Added 4 optional fields to `ProvenanceRef`; added 6 fields to `ProvenanceLink` (`edge_type`, `edge_label` + 4 new) |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Modified | Enriched `vaultProvenance` with `source_role`, `angle_kind`, `signal_kind`, `signal_text` from neighbor provenance |
| `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` | Modified | Enriched `handleAngleSelected` neighbor provenance with angle/evidence metadata; added `source_role` to `handleHookSelected` |
| `dashboard/tests/unit/DraftStudioDetailsPane.test.ts` | Modified | Added new fields to `ProvenanceLink` test fixture |
| `docs/roadmap/hook-miner-forge-loop/hook-miner-provenance-contract.md` | Created | Additive provenance contract documentation |
| `docs/roadmap/hook-miner-forge-loop/session-05-handoff.md` | Created | This file |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D29 | 4 nullable columns via `ALTER TABLE ADD COLUMN` | Backward compatible. No defaults needed. Existing rows get `NULL`. |
| D30 | `source_role` values: `primary_selection` and `accepted_neighbor` | Forge sync needs to know which note to write back to. Primary selection receives writeback; neighbors are supporting context. |
| D31 | `angle_kind` set on ALL provenance refs for a given draft | Simpler querying — `SELECT DISTINCT angle_kind` gives the angle without joins. |
| D32 | `signal_kind` and `signal_text` only on evidence-bearing refs | These fields describe the evidence relationship, which only exists for items the LLM extracted evidence from. |
| D33 | Frontend pre-builds enriched provenance during angle selection | Avoids server-side re-derivation. VaultSelectionReview constructs the full provenance before calling `ongenerate`. |
| D34 | `copy_links_for` explicitly copies new columns | Ensures entity transitions (draft → approval → tweet) preserve attribution. |
| D35 | `#[serde(default)]` on all new fields | Old clients without the new fields deserialize cleanly to `None`. |
| D36 | `ProvenanceLink` TypeScript type also gains `edge_type` and `edge_label` | These were missing from the TS type despite existing in the DB. Fixed alongside the 4 new fields for completeness. |

---

## Test Coverage

| Module | Tests | Status |
|--------|-------|--------|
| `provenance_ref_serde_with_hook_miner_fields` | JSON roundtrip with all 4 new fields populated | Pass |
| `provenance_ref_backward_compat_no_new_fields` | Deserialize old JSON without new fields → all None | Pass |
| `insert_and_get_with_hook_miner_fields` | DB insert + SELECT preserves new fields (primary + neighbor) | Pass |
| `copy_links_preserves_hook_miner_fields` | `copy_links_for` copies all 4 new columns | Pass |
| `hook_miner_fields_null_for_legacy_rows` | Legacy-style refs get NULL for new columns | Pass |
| **Total Rust tests** | **3472** | **All pass** |
| **Total frontend tests** | **971** | **All pass** |

---

## Quality Gates

```
cargo fmt --all && cargo fmt --all --check     ✅
RUSTFLAGS="-D warnings" cargo test --workspace ✅ (3472 passed)
cargo clippy --workspace -- -D warnings        ✅
npm --prefix dashboard run check               ✅ (0 errors, 0 warnings)
npx vitest run                                 ✅ (971 passed)
```

---

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `ProvenanceLink.edge_type`/`edge_label` were missing from TS type | Fixed | Added alongside new fields. No runtime impact since old code didn't access them. |
| Evidence `source_node_id` may not match any `acceptedNeighbors` entry | Low | Frontend uses `.find()` which returns `undefined` — evidence fields stay unset. |
| VaultSelectionReview at ~330 lines script (unchanged from session 04) | Low | Well within 400-line Svelte limit. New logic adds ~10 lines per handler. |

---

## Required Inputs for Session 06

Session 06 implements the Forge Data Contract — defining how performance data flows back to vault notes.

**Must read:**
- `docs/roadmap/hook-miner-forge-loop/hook-miner-provenance-contract.md` (this session's contract)
- `crates/tuitbot-core/src/storage/provenance.rs` (updated provenance module)
- `dashboard/src/lib/api/types.ts` (updated `ProvenanceRef` and `ProvenanceLink`)

**Must preserve:**
- All backward compatibility guarantees (nullable fields, serde defaults)
- Existing provenance consumers (approval poster, analytics, draft lifecycle)
- `source_role` semantics for Forge writeback targeting

**Must create:**
- Forge data contract defining performance-to-note writeback schema
- Query patterns for resolving primary selection notes
- Writeback format specification for Obsidian vault notes

---

## Architecture Summary

```
Angle Selection (AngleCards)
        │
        ▼
VaultSelectionReview.handleAngleSelected()
  → angle_kind = angle.angle_type (on ALL provenance refs)
  → source_role = 'accepted_neighbor' (on neighbor refs)
  → signal_kind = evidence.evidence_type (on evidence-matched refs)
  → signal_text = evidence.citation_text (on evidence-matched refs)
        │
        ▼
ComposerInspector.handleGenerateFromVault()
  → source_role = 'primary_selection' (on primary node)
  → angle_kind = hookStyle (on primary node)
  → Propagates neighbor fields as-is
        │
        ▼
POST /api/content/drafts { provenance: ProvenanceRef[] }
        │
        ▼
insert_links_for() → vault_provenance_links (15 columns)
        │
        ├─ schedule_draft → copy_links_for (preserves all fields)
        ├─ publish_draft → ProvenanceRef reconstruction (12 fields)
        │                  → approval_queue (source_chunks_json)
        │                  → copy_links_for → original_tweet
        │
        ▼
Forge sync (Session 08): SELECT WHERE source_role = 'primary_selection'
Analytics (Session 10): GROUP BY angle_kind, signal_kind
```
