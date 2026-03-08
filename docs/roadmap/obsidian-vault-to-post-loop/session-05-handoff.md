# Session 05 Handoff — Retrieval Engine & Context Builder v2

## What Changed

### New: Retrieval Module (`context/retrieval.rs`)

- **`VaultCitation`** — Structured citation record linking prompt fragments
  back to vault sources (chunk_id, node_id, heading_path, source_path,
  source_title, snippet, retrieval_boost).
- **`FragmentContext`** — Intermediate result pairing chunk text with citation
  metadata for prompt formatting.
- **`retrieve_vault_fragments()`** — Account-scoped chunk retrieval with
  optional selected-note bias. Queries selected nodes first, then fills
  remaining slots with keyword search. Deduplicates by chunk_id.
- **`format_fragments_prompt()`** — Formats fragment text with inline heading
  citations, capped at `MAX_FRAGMENT_CHARS` (1000).
- **`build_citations()`** — Extracts `VaultCitation` records from fragments.

### Modified: Ancestor Retrieval (`storage/analytics/ancestors.rs`)

- **`get_scored_ancestors()`** — Added `account_id: &str` parameter. Both
  tweet and reply subqueries now filter by `ot.account_id = ?` and
  `rs.account_id = ?` respectively. Fixes a pre-existing account isolation gap.

### Modified: Chunk Storage (`storage/watchtower/chunks.rs`)

- **`search_chunks_with_context()`** — New function: JOIN query returning
  chunks with parent node's `relative_path` and `title` for citation display.
- **`get_chunks_for_nodes_with_context()`** — New function: retrieves chunks
  for specific node IDs with parent node metadata. Used for selected-note bias.
- **`ChunkWithNodeContext`** — New struct in `storage/watchtower/mod.rs`
  combining a `ContentChunk` with its parent node's path and title.

### Modified: Context Builder (`context/winning_dna.rs`)

- **`build_draft_context()`** — New signature adds `account_id: &str`.
  Implements three-tier model: ancestors + fragments + seeds fallback.
  When both ancestors and fragments exist, they combine into a single prompt
  with split budgets (800 chars ancestors, 1000 chars fragments).
- **`DraftContext`** — Added `vault_citations: Vec<VaultCitation>` field.
- **`retrieve_ancestors()`** — Added `account_id` parameter, passes through
  to `get_scored_ancestors()`.
- **`retrieve_cold_start_seeds()`** — Added `account_id` parameter, calls
  `get_seeds_for_context_for()` instead of `get_seeds_for_context()`.
- **`MAX_ANCESTOR_CHARS`** — New constant (800) for ancestor budget when
  combining with fragments.
- **`format_ancestors_prompt_capped()`** — New internal function for
  budget-limited ancestor formatting.
- **`combine_prompt_blocks()`** — Merges ancestor and fragment blocks,
  truncating at `RAG_MAX_CHARS`.

### Modified: Callers

- **`tuitbot-server/routes/assist.rs`** — `resolve_composer_rag_context()`
  now passes `account_id` to `build_draft_context()`.
- **`tuitbot-core/workflow/draft.rs`** — `DraftInput` gains optional
  `account_id` field. `execute()` threads it to `build_draft_context()`,
  defaulting to `DEFAULT_ACCOUNT_ID`.
- All `DraftInput` construction sites updated with `account_id: None`.

## Files Created

- `crates/tuitbot-core/src/context/retrieval.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/retrieval-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-05-handoff.md`

## Files Modified

- `crates/tuitbot-core/src/context/mod.rs`
- `crates/tuitbot-core/src/context/winning_dna.rs`
- `crates/tuitbot-core/src/storage/analytics/ancestors.rs`
- `crates/tuitbot-core/src/storage/analytics/tests.rs`
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`
- `crates/tuitbot-core/src/storage/watchtower/chunks.rs`
- `crates/tuitbot-core/src/workflow/draft.rs`
- `crates/tuitbot-core/src/workflow/tests.rs`
- `crates/tuitbot-core/src/workflow/orchestrate.rs`
- `crates/tuitbot-core/src/workflow/e2e_tests.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/tests/assist_rag_tests.rs`
- `crates/tuitbot-mcp/src/tools/workflow/composite/draft_replies.rs`

## Test Results

All tests pass:
- `cargo fmt --all --check` — clean
- `RUSTFLAGS="-D warnings" cargo test --workspace` — 1998 passed, 0 failed, 12 ignored
- `cargo clippy --workspace -- -D warnings` — clean

### New Tests Added

**Unit tests (winning_dna.rs):**
- `retrieve_ancestors_account_isolation` — Account A data invisible to account B
- `build_draft_context_with_fragments` — Fragment-only context with citations
- `build_draft_context_mixed_ancestors_and_fragments` — Combined prompt with both
  "Winning patterns" and "Relevant knowledge" sections
- `fragment_citations_populated_correctly` — Citation fields match source data

**Integration tests (assist_rag_tests.rs):**
- `tweet_with_fragment_context` — Chunks seeded, verify "Relevant knowledge" in prompt
- `tweet_with_mixed_ancestor_and_fragment_context` — Both ancestors and chunks
  seeded, verify both headers in prompt
- `fragment_context_account_isolation` — Chunks for account A, query as default
  account, verify no leakage

## What Remains

| Session | Scope | Status |
|---------|-------|--------|
| Seed Worker | Generate seeds per-chunk rather than per-node | Future |
| Provenance Wiring | Populate `source_node_id`, `source_chunk_id` on approval_queue | Future |
| Loop-Back | `update_chunk_retrieval_boost` from analytics feedback | Future |
| API Citations | Expose `vault_citations` in assist response JSON | Future |
| Dashboard: Citation Display | Show source notes in composer UI | Future |
| Dashboard: Vault Health | Source status UI, sync indicators | Future |
| Dashboard: Source Config | Enable/disable toggle, change_detection picker | Future |
| Selected-Note Bias API | Wire `selected_node_ids` through assist endpoints | Future |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| LIKE-based keyword search returns low-quality matches | Medium | Low | Acceptable for MVP. `retrieval_boost` ordering helps. Future: TF-IDF or embedding search |
| Prompt budget split (800+1000) may be tight for rich contexts | Low | Low | Constants are tunable; start conservative |
| `winning_dna.rs` at ~850 lines approaches 500-line limit | Medium | Medium | Extracted retrieval logic to `retrieval.rs`; remaining code is classification + scoring + orchestration |
| `DraftInput.account_id` is `Option<String>` | Low | Low | Defaults to `DEFAULT_ACCOUNT_ID`; callers can set it when account context is available |
| Fragment retrieval adds latency to assist calls | Low | Low | Two additional SQLite queries; negligible vs LLM call latency |

## Decisions Made

1. **Three-tier model over binary (ancestors XOR seeds)** — Ancestors and
   fragments serve different purposes (behavioral vs knowledge signals) and
   should combine rather than compete.

2. **Extracted `retrieval.rs` module** — Keeps `winning_dna.rs` within the
   500-line spirit by separating retrieval/citation logic from classification/
   scoring/orchestration.

3. **`DraftInput.account_id` as `Option<String>`** — Backward compatible with
   existing callers that don't have account context. Defaults to the default
   account ID.

4. **No `floor_char_boundary`** — MSRV is 1.75; `floor_char_boundary` requires
   1.80+. Used manual `is_char_boundary` loop instead.

5. **Account filtering on ancestors was a pre-existing gap** — Fixed as part
   of this session. The `get_scored_ancestors` function now requires and uses
   `account_id`.

6. **Citations in `DraftContext` only, not in API response** — API/UI exposure
   is a future session deliverable. The structured data is ready for it.

## Inputs for Next Session

- `retrieval-contract.md` — ranking and prompt shaping reference
- `fragment-indexing.md` — extraction rules and identity contract (from session 04)
- Key files for seed worker integration:
  - `crates/tuitbot-core/src/automation/seed_worker.rs` — integrate per-chunk seeds
  - `crates/tuitbot-core/src/storage/watchtower/seeds.rs` — populate `chunk_id`
- Key files for provenance wiring:
  - `crates/tuitbot-core/src/workflow/queue.rs` — add `source_chunk_id` to queue items
  - `crates/tuitbot-server/src/routes/approval.rs` — expose provenance in API
- The retrieval engine is ready for use. `build_draft_context` now returns
  `vault_citations` alongside `prompt_block`, enabling API and UI work to
  expose source attribution.
