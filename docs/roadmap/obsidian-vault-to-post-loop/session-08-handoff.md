# Session 08 Handoff — Vault Query and Generation APIs

## What Changed

### Created: `crates/tuitbot-server/src/routes/vault.rs`

New route module with five account-scoped endpoints:

- **`GET /api/vault/sources`** — Returns all source contexts for the account with per-source node counts.
- **`GET /api/vault/notes?q=&source_id=&limit=`** — Search notes by title/path (LIKE-based). Returns note summaries without body text.
- **`GET /api/vault/notes/{id}`** — Note detail with chunk heading summaries. Only snippets (≤120 chars), no raw body.
- **`GET /api/vault/search?q=&limit=`** — Fragment search across chunks. Returns `VaultCitation[]` (same shape as assist responses).
- **`POST /api/vault/resolve-refs`** — Resolve selected node IDs to their chunk citations.

Five contract tests included in the module.

### Modified: `crates/tuitbot-server/src/routes/assist.rs`

- Added `selected_node_ids: Option<Vec<i64>>` (with `#[serde(default)]`) to `AssistTweetRequest`, `AssistThreadRequest`, and `AssistImproveRequest`.
- Updated `resolve_composer_rag_context` to accept and pass through `selected_node_ids`.
- Updated all three handler bodies to extract and forward `body.selected_node_ids`.
- Added `selected_node_ids_is_optional` test for backward compatibility.

### Modified: `crates/tuitbot-core/src/context/winning_dna.rs`

- Added `build_draft_context_with_selection()` — new entry point accepting `selected_node_ids: Option<&[i64]>`.
- Existing `build_draft_context()` delegates to it with `None` (fully backward compatible).

### Modified: `crates/tuitbot-core/src/storage/watchtower/nodes.rs`

Added five account-scoped query functions:
- `search_nodes_for()` — LIKE search on title + relative_path
- `get_nodes_for_source_for()` — list nodes by source
- `get_content_node_for()` — get node by ID, scoped to account
- `count_chunks_for_node()` — count active chunks
- `count_nodes_for_source()` — count nodes per source

### Modified: `crates/tuitbot-core/src/storage/watchtower/sources.rs`

Added `get_all_source_contexts_for()` — returns all source contexts for an account regardless of status.

### Modified: `crates/tuitbot-server/src/routes/mod.rs`

Added `pub mod vault;`.

### Modified: `crates/tuitbot-server/src/lib.rs`

Registered five vault routes after assist routes.

### Modified: `dashboard/src/lib/api/types.ts`

Added vault types:
- `VaultCitation`, `ProvenanceRef`, `VaultSourceStatus`, `VaultNoteItem`, `VaultChunkSummary`, `VaultNoteDetail`
- Added `provenance?: ProvenanceRef[]` to `ComposeRequest`

### Modified: `dashboard/src/lib/api/client.ts`

- Added `api.vault.*` namespace with 5 methods (sources, searchNotes, noteDetail, searchFragments, resolveRefs).
- Updated `api.assist.tweet`, `api.assist.thread`, `api.assist.improve` to accept optional `selectedNodeIds` and return `vault_citations`.
- Updated `api.drafts.create` to accept optional `provenance` parameter.

### Created: `docs/roadmap/obsidian-vault-to-post-loop/vault-api-contract.md`

Documents all five vault endpoints with request/response shapes, error codes, privacy guarantees, and TypeScript type reference.

## Files Modified

- `crates/tuitbot-core/src/context/winning_dna.rs`
- `crates/tuitbot-core/src/storage/watchtower/nodes.rs`
- `crates/tuitbot-core/src/storage/watchtower/sources.rs`
- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/mod.rs`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`

## Files Created

- `crates/tuitbot-server/src/routes/vault.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/vault-api-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-08-handoff.md`

## Test Results

- `cargo fmt --all --check` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `RUSTFLAGS="-D warnings" cargo test --workspace` — all passed, 0 failed
- `npm --prefix dashboard run check` — 0 errors, 7 warnings (all pre-existing)

### New Tests Added

**Vault route contract tests (vault.rs):**
- `vault_sources_returns_empty_when_no_sources`
- `search_notes_returns_empty_for_no_matches`
- `note_detail_returns_404_for_missing_node`
- `search_fragments_returns_empty_for_no_chunks`
- `resolve_refs_returns_empty_for_empty_ids`

**Assist backward compatibility (assist.rs):**
- `selected_node_ids_is_optional` — verifies all three request structs deserialize without the new field

## What Remains

| Item | Scope | Status |
|------|-------|--------|
| Dashboard: Vault Search UI | Composer search panel using `api.vault.*` | Future |
| Dashboard: Citation Display | Show source notes in composer results | Future |
| Dashboard: Vault Health | Source status page, sync indicators | Future |
| Dashboard: Source Config | Enable/disable toggle, change_detection picker | Future |
| Seed Worker | Generate seeds per-chunk rather than per-node | Future |
| Analytics Loop-Back | `update_chunk_retrieval_boost` from tweet performance | Future |
| Scheduled Content Provenance | Store provenance links when scheduling | Future |
| Multi-Account Poster | Propagate provenance with correct account_id | Future |
| Thread-Level Loop-Back | Write all tweet_ids from a thread into source note | Future |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Note search LIKE queries slow on large vaults | Low | Low | Default limit 20, max limit 100. LIKE on indexed columns. |
| Full chunk text leaked through API | Low | High | Only snippets (120 chars) returned. `body_text` never serialized to API. |
| `build_draft_context_with_selection` signature change breaks callers | None | None | Original `build_draft_context` preserved as wrapper. All tests pass. |
| Assist request shape change breaks clients | Low | Medium | `selected_node_ids` uses `#[serde(default)]`. Omitting the field is identical to old behavior. |
| `vault.rs` approaches 500-line limit | Low | Low | Currently ~310 lines including tests. Room for growth before split needed. |
| `DEFAULT_ACCOUNT_ID` used for test state | Known | None | Test pattern matches existing codebase. Account isolation tested via AccountContext. |

## Decisions Made

1. **Dedicated `vault.rs` route module** — Separate from `sources.rs` (which is admin/watchtower-focused) and `assist.rs` (which is generation-focused). Vault endpoints serve the dashboard's content browsing needs.

2. **`build_draft_context_with_selection` as new entry point** — Rather than modifying the widely-used `build_draft_context` signature, added a new function and made the old one a thin wrapper. Zero breakage.

3. **Privacy-safe by default** — Note search never returns `body_text`. Note detail returns only chunk heading paths + 120-char snippets. Source responses omit `config_json`.

4. **Fragment search reuses `retrieve_vault_fragments`** — The vault search endpoint uses the same retrieval pipeline as assist, ensuring consistent results and ranking.

5. **`count_nodes_for_source` in core, not server** — Moved the SQL count query to `tuitbot-core::storage::watchtower::nodes` since `tuitbot-server` doesn't depend on `sqlx` directly. Consistent with the "server owns zero business logic" constraint.

## Inputs for Next Session

- `vault-api-contract.md` — full API reference for vault endpoints
- Key files for dashboard vault UI:
  - `dashboard/src/lib/api/client.ts` — `api.vault.*` methods
  - `dashboard/src/lib/api/types.ts` — `VaultCitation`, `VaultNoteItem`, `VaultNoteDetail`, etc.
- Key files for citation display in composer:
  - `api.assist.tweet(topic, selectedNodeIds?)` returns `vault_citations`
  - `api.drafts.create(contentType, content, source, blocks, provenance?)` accepts provenance refs
- The vault API surface is stable and ready for the dashboard to consume.
