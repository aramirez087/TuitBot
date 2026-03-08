# Session 04 Handoff — Markdown Fragment Indexing

## What Changed

### New: Chunker Module (`automation/watchtower/chunker.rs`)

- **`extract_fragments(body) -> Vec<Fragment>`** — Pure function that splits
  markdown/plain-text into heading-delimited fragments. Tracks heading hierarchy
  via a stack, ignores headings inside fenced code blocks, skips empty fragments.
- **`chunk_node(pool, account_id, node_id, body_text) -> Vec<i64>`** — Extracts
  fragments, marks existing chunks stale, upserts new chunks (dedup by SHA-256
  hash), marks node as `chunked`. Returns chunk IDs.
- **`chunk_pending_nodes(pool, account_id, limit) -> u32`** — Batch processor
  that fetches pending nodes and chunks each one. Used by the watchtower after
  scan/poll/event cycles.
- **`ChunkerError`** — Wraps `StorageError` for the chunking pipeline.

### Modified: Watchtower Loop (`automation/watchtower/mod.rs`)

- Added `pub mod chunker;` declaration.
- Added `WatchtowerError::Chunker` variant.
- Added `chunk_pending()` helper method on `WatchtowerLoop`.
- Inserted `self.chunk_pending().await` calls after:
  - Initial local directory scan
  - Initial remote source poll
  - Each fallback scan cycle
  - Each remote poll cycle
  - Each filesystem event batch
  - Each remote-only loop poll

### Modified: Chunk Upsert (`storage/watchtower/chunks.rs`)

- `upsert_chunks_for_node` now checks for existing chunks with matching hash
  **regardless of status** (was: only `active`). Stale chunks with matching
  hashes are reactivated with updated `heading_path` and `chunk_index`. This
  preserves chunk IDs and `retrieval_boost` across re-chunking.

## Files Created

- `crates/tuitbot-core/src/automation/watchtower/chunker.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/fragment-indexing.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-04-handoff.md`

## Files Modified

- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/automation/watchtower/tests.rs`
- `crates/tuitbot-core/src/storage/watchtower/chunks.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`

## Test Results

All tests pass:
- `cargo fmt --all --check` — clean
- `RUSTFLAGS="-D warnings" cargo test --workspace` — 1991 passed, 0 failed, 1 ignored
- `cargo clippy --workspace -- -D warnings` — clean

### New Tests Added

**Unit tests (watchtower/tests.rs):**
- `extract_fragments_with_headings` — Multiple heading levels, heading_path hierarchy
- `extract_fragments_no_headings` — Plain text → single root fragment
- `extract_fragments_nested_headings_with_reset` — `# > ## > ### > ##` path reset
- `extract_fragments_empty_body` — Empty string → empty vec
- `extract_fragments_consecutive_headings_no_body` — Headings without body → skipped
- `extract_fragments_preserves_content` — Code blocks preserved exactly
- `extract_fragments_heading_inside_code_block_ignored` — Fenced code block awareness
- `chunk_node_creates_chunks` — DB round-trip: ingest → chunk → verify
- `chunk_node_stale_on_update_preserves_unchanged` — Re-chunk preserves unchanged chunk IDs

**E2E integration tests (source/tests/integration.rs):**
- `e2e_fragment_extraction_from_markdown` — Full pipeline: ingest markdown → verify 4 fragments with correct heading_paths
- `e2e_fragment_update_on_content_change` — Re-ingest with changes → unchanged chunks preserved, changed chunks get new IDs
- `e2e_plain_text_fallback_fragment` — `.txt` file → single root fragment
- `e2e_mixed_source_fragment_isolation` — Local + Drive sources → chunks are per-node, account-scoped
- `e2e_empty_body_no_fragments` — Front-matter-only note → zero chunks

## What Remains

| Session | Scope | Status |
|---------|-------|--------|
| Seed Worker | Query chunked nodes, pass chunk_id to seeds | Next |
| Chunk RAG | `search_chunks_by_keywords` integration in assist | Future |
| Provenance Wiring | Populate `source_node_id`, `source_seed_id` on approval_queue | Future |
| Loop-Back | `update_chunk_retrieval_boost` from analytics | Future |
| Account Isolation Audit | Ensure all callers use `_for` variants | Future |
| Dashboard: Vault Health | Source status UI, sync indicators | Future |
| Dashboard: Source Config | Enable/disable toggle, change_detection picker | Future |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Regex heading parser misses edge cases (HTML blocks, indented code) | Medium | Low | Fenced code blocks handled; document limitations; can add `pulldown-cmark` later without schema changes |
| Large notes produce many small fragments | Low | Low | No hard limit needed now; future session can add min-size thresholds or merging |
| `chunk_pending_nodes` re-processes nodes if called twice before status update | Low | Low | `upsert_chunks_for_node` is idempotent by hash; double-processing is harmless |
| watchtower `mod.rs` at ~950 lines exceeds 500-line limit | Low | Medium | Pre-existing debt; chunker adds only ~10 lines. Extract in a future refactor session |
| Chunking slows ingest for large vaults | Low | Medium | Chunking is O(n) per note body; SHA-256 is fast. No concern until 10k+ notes |

## Decisions Made

1. **Separate-pass chunking over inline chunking** — `chunk_pending_nodes` queries
   pending nodes after scan/poll rather than modifying `ingest_content`'s signature.
   This is additive, doesn't change existing ingest callers, and naturally batches work.

2. **OnceLock over LazyLock for regex** — MSRV is 1.75.0; `LazyLock` requires 1.80.0.
   Used `OnceLock` pattern consistent with the rest of the codebase.

3. **Reactivate stale chunks on hash match** — Modified `upsert_chunks_for_node` to
   find matching chunks regardless of status and reactivate them. Preserves chunk IDs
   and `retrieval_boost` across re-chunking, which is critical for stable citations.

4. **No new crate dependencies** — Used existing `regex`, `sha2`, and standard library.
   No `pulldown-cmark` needed for heading-level splitting.

5. **Code-block awareness** — Fenced code blocks toggle a flag that prevents heading
   detection inside them. This handles the most common false-positive scenario.

## Inputs for Next Session

- `fragment-indexing.md` — extraction rules and identity contract
- `data-model.md` — schema reference (from session 02)
- Key files to modify:
  - `crates/tuitbot-core/src/automation/seed_worker.rs` — integrate with chunked nodes
  - `crates/tuitbot-core/src/storage/watchtower/seeds.rs` — populate `chunk_id` on seeds
- Storage functions ready for use:
  - `get_chunks_for_node(pool, account_id, node_id)` — list active chunks
  - `search_chunks_by_keywords(pool, account_id, keywords, limit)` — keyword retrieval
  - `update_chunk_retrieval_boost(pool, account_id, chunk_id, boost)` — loop-back boost
- The chunker now produces stable fragments with predictable identities. The seed
  worker can be updated to generate seeds per-chunk rather than per-node, populating
  `draft_seeds.chunk_id` for granular provenance.
