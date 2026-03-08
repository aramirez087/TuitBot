# Session 06 Handoff — End-to-End Provenance Wiring

## What Changed

### New: `vault_provenance_links` Table

Polymorphic link table mapping any content entity (approval_queue, scheduled_content, original_tweet, thread) back to vault source material. Stores snapshot values (source_path, heading_path, snippet) so provenance survives even if source notes are deleted. Migration applied to both `migrations/` and `crates/tuitbot-core/migrations/`.

### New: `storage/provenance.rs` Module

CRUD operations for provenance links:
- `insert_links_for()` — batch insert for an entity
- `get_links_for()` — query by entity type + ID
- `copy_links_for()` — copy links between entities (used when approval_queue → original_tweet)
- `delete_links_for()` — cleanup

### New: `ProvenanceRef` Struct

API-layer provenance reference in `storage/provenance.rs`. All fields optional. Serializable via serde for JSON transport.

### New: Converter Functions (`context/retrieval.rs`)

- `citations_to_provenance_refs()` — `VaultCitation[]` → `ProvenanceRef[]`
- `citations_to_chunks_json()` — `VaultCitation[]` → legacy JSON for `source_chunks_json` column

### Modified: `storage/approval_queue/queries.rs`

- Added `ProvenanceInput` struct bundling `source_node_id`, `source_seed_id`, `source_chunks_json`, and `refs: Vec<ProvenanceRef>`
- Added `enqueue_with_provenance_for()` — populates inline provenance columns AND inserts link table rows
- Existing `enqueue_for()` / `enqueue_with_context_for()` unchanged

### Modified: `storage/scheduled_content.rs`

- Added `insert_draft_with_provenance_for()` — creates draft row then inserts provenance link rows

### Modified: `storage/threads.rs`

- Added `set_original_tweet_source_node_for()` — sets `source_node_id` on existing original_tweet
- Added `insert_original_tweet_with_provenance_for()` — creates tweet row with provenance links

### Modified: `routes/assist.rs`

- `resolve_composer_rag_context()` now returns full `DraftContext` (not just `String`)
- `AssistTweetResponse`, `AssistThreadResponse`, `AssistImproveResponse` now include `vault_citations: Vec<VaultCitation>` (skipped when empty)
- Handlers extract `prompt_block` and `vault_citations` from the context

### Modified: `routes/content/drafts.rs`

- `CreateDraftRequest` now accepts optional `provenance: Vec<ProvenanceRef>`
- `create_draft()` uses `insert_draft_with_provenance_for()` when provenance provided
- `publish_draft()` loads provenance links from `scheduled_content`, builds `ProvenanceInput`, and passes to `enqueue_with_provenance_for()`

### Modified: `routes/content/compose.rs`

- `ComposeTweetRequest` and `ComposeRequest` now accept optional `provenance: Vec<ProvenanceRef>`
- `compose_tweet()`, `persist_content()`, `compose_thread_blocks_flow()` use `enqueue_with_provenance_for()` when provenance provided
- Added `build_provenance_input()` helper to convert `Option<&[ProvenanceRef]>` to `Option<ProvenanceInput>`

### Modified: `automation/approval_poster.rs`

- After successful post, calls `propagate_provenance()` which:
  1. Creates `original_tweets` record if `source_node_id` or `source_seed_id` is set
  2. Sets `source_node_id` on the original_tweet
  3. Copies provenance links from `approval_queue` to `original_tweet`

### Modified: Test Infrastructure

- `storage/reset.rs` — Added `vault_provenance_links` to `TABLES_TO_CLEAR` (children-first position), updated table count assertions from 32 to 33
- `storage/mod.rs` — Added `vault_provenance_links` to table existence test
- `tuitbot-server/tests/factory_reset.rs` — Updated table count assertion

## Files Created

- `migrations/20260308020000_vault_provenance_links.sql`
- `crates/tuitbot-core/migrations/20260308020000_vault_provenance_links.sql`
- `crates/tuitbot-core/src/storage/provenance.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/provenance-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-06-handoff.md`

## Files Modified

- `crates/tuitbot-core/src/storage/mod.rs`
- `crates/tuitbot-core/src/context/retrieval.rs`
- `crates/tuitbot-core/src/storage/approval_queue/queries.rs`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`
- `crates/tuitbot-core/src/storage/threads.rs`
- `crates/tuitbot-core/src/storage/reset.rs`
- `crates/tuitbot-core/src/automation/approval_poster.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `crates/tuitbot-server/tests/factory_reset.rs`

## Test Results

All tests pass:
- `cargo fmt --all --check` — clean
- `RUSTFLAGS="-D warnings" cargo test --workspace` — 2002 passed, 0 failed, 12 ignored
- `cargo clippy --workspace -- -D warnings` — clean

### New Tests Added

**Unit tests (provenance.rs):**
- `insert_and_get_provenance_links` — round-trip create + query
- `copy_links_between_entities` — copy from approval_queue to original_tweet
- `delete_links` — cleanup works
- `empty_provenance_is_noop` — inserting empty refs doesn't error

## What Remains

| Session | Scope | Status |
|---------|-------|--------|
| Seed Worker | Generate seeds per-chunk rather than per-node | Future |
| Loop-Back | `update_chunk_retrieval_boost` from analytics feedback | Future |
| API Citations | Frontend reads `vault_citations` and passes back as `provenance` | Future |
| Dashboard: Citation Display | Show source notes in composer UI | Future |
| Dashboard: Vault Health | Source status UI, sync indicators | Future |
| Dashboard: Source Config | Enable/disable toggle, change_detection picker | Future |
| Selected-Note Bias API | Wire `selected_node_ids` through assist endpoints | Future |
| Scheduled Content Provenance | Store provenance links when scheduling (non-approval path) | Future |
| Multi-Account Poster | Propagate provenance with correct account_id | Future |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Polymorphic FK can't use real constraints | Known | Low | Indexed; convention documented. Acceptable for SQLite. |
| Provenance links accumulate without cleanup | Low | Low | Small rows; add cleanup query if needed later |
| Approval poster uses DEFAULT_ACCOUNT_ID | Medium | Medium | Current single-account setup. Multi-account poster work will address |
| `enqueue_with_provenance_for` has many params | Low | Low | Bundled into `ProvenanceInput` struct |
| Snapshot values in provenance may become stale | Known | Low | By design — provenance should survive source deletion |

## Decisions Made

1. **Separate link table over more inline columns** — Multiple content tables need provenance. A single `vault_provenance_links` table avoids duplicating columns on every table and supports multiple source refs per item.

2. **Keep inline columns on `approval_queue`** — `source_node_id`, `source_seed_id`, `source_chunks_json` are populated alongside the link table for backward compatibility and quick lookups without JOINs.

3. **Snapshot values in provenance links** — `source_path`, `heading_path`, `snippet` are stored at creation time so provenance survives source note deletion.

4. **`ProvenanceRef` in `storage::provenance` module** — It's a persistence concept used across storage layers, not just retrieval.

5. **Provenance is always optional** — All API fields use `#[serde(default)]`. Legacy callers work unchanged. This is critical for backward compatibility.

6. **Approval poster creates `original_tweets` for provenance tracking** — When an approval item with provenance is posted, the poster creates an original_tweets record to maintain the chain.

## Inputs for Next Session

- `provenance-contract.md` — full API and storage reference
- Key files for frontend citation display:
  - `crates/tuitbot-server/src/routes/assist.rs` — `vault_citations` in responses
  - `crates/tuitbot-server/src/routes/content/drafts.rs` — `provenance` in request
  - `crates/tuitbot-server/src/routes/content/compose.rs` — `provenance` in request
- Key files for loop-back:
  - `crates/tuitbot-core/src/storage/provenance.rs` — query links by node_id
  - `crates/tuitbot-core/src/storage/watchtower/chunks.rs` — `update_chunk_retrieval_boost`
- The provenance chain is now: vault note → VaultCitation → ProvenanceRef → vault_provenance_links → approval_queue → original_tweets. Future loop-back can query original_tweets performance → trace back to chunks via provenance links → update retrieval_boost.
