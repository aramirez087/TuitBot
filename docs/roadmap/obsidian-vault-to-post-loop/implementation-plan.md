# Obsidian Vault to Post Loop — Implementation Plan

Sessions 02–13. Each session has concrete inputs, outputs, defaults, and
verification steps. No deferred decisions.

---

## Session 02: Fragment Chunking Pipeline

**Goal:** Split ingested notes into heading-delimited chunks.

### Files to Create

- `migrations/20260309000021_content_chunks.sql` — `content_chunks` table with
  indexes on `(account_id, node_id)` and `(account_id, status)`.
- `crates/tuitbot-core/src/automation/watchtower/chunker.rs` — heading-based
  splitter module.

### Files to Modify

- `crates/tuitbot-core/src/storage/watchtower/mod.rs` — Add CRUD for
  `content_chunks`: `upsert_chunks_for_node()`, `get_chunks_for_node()`,
  `mark_chunks_stale()`, `delete_stale_chunks()`.
- `crates/tuitbot-core/src/automation/watchtower/mod.rs` — After
  `ingest_content()` upserts a node, call `chunk_note()` to split and store
  fragments. Update node status to `chunked`.
- `crates/tuitbot-core/src/automation/seed_worker.rs` — Change batch query
  from `status = 'pending'` to `status = 'chunked'`. Generate seeds
  per-chunk instead of per-note (iterate `content_chunks` for the node).

### Defaults

- Split regex: `^#{1,3} ` (H1–H3 headings).
- Min chunk size: 50 chars.
- Max chunk size: 2000 chars (split at `\n\n` if exceeded).
- Heading path: slash-delimited hierarchy.
- No-heading fallback: paragraph split at `\n\n`, 500-char max.

### Verification

```bash
cargo test -p tuitbot-core chunker
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 03: Provenance Schema & Storage

**Goal:** Add provenance columns and storage functions.

### Files to Create

- `migrations/20260309000022_draft_provenance.sql` — Adds:
  - `ALTER TABLE approval_queue ADD COLUMN source_node_id INTEGER REFERENCES content_nodes(id);`
  - `ALTER TABLE approval_queue ADD COLUMN source_seed_id INTEGER REFERENCES draft_seeds(id);`
  - `ALTER TABLE approval_queue ADD COLUMN source_chunks_json TEXT DEFAULT '[]';`
  - `ALTER TABLE draft_seeds ADD COLUMN chunk_id INTEGER REFERENCES content_chunks(id);`

### Files to Modify

- `crates/tuitbot-core/src/storage/approval_queue/mod.rs` — Extend
  `ApprovalRow` and `ApprovalItem` with `source_node_id: Option<i64>`,
  `source_seed_id: Option<i64>`, `source_chunks_json: String`.
- `crates/tuitbot-core/src/storage/approval_queue/queries.rs` — Update
  `enqueue_for()` to accept provenance parameters. Update SELECT queries to
  include new columns.
- `crates/tuitbot-core/src/storage/watchtower/mod.rs` — Extend `DraftSeed`
  struct with `chunk_id: Option<i64>`. Update `insert_draft_seed()` to accept
  `chunk_id`. Update `DraftSeedRow` tuple type.

### Defaults

- All new columns nullable (backward compatible).
- `source_chunks_json` defaults to `'[]'`.
- Existing rows unaffected.

### Verification

```bash
cargo test -p tuitbot-core approval
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 04: Chunk-Level RAG Retrieval

**Goal:** Replace seed-only retrieval with chunk-level retrieval.

### Files to Modify

- `crates/tuitbot-core/src/context/winning_dna.rs`:
  - Add `ChunkContext` struct: `chunk_id`, `node_id`, `note_title`,
    `heading_path`, `chunk_text`, `relevance_score`.
  - Add `retrieve_relevant_chunks()`: keyword LIKE-search on `chunk_text`,
    rank by `match_count * retrieval_boost`, return top 3.
  - Extend `DraftContext` with `relevant_chunks: Vec<ChunkContext>`.
  - Update `build_draft_context()` to call `retrieve_relevant_chunks()`.
    Include chunk text in `prompt_block`. Seeds remain as fallback.
  - Add `format_chunk_context()` helper for prompt formatting.
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`:
  - Add `search_chunks_by_keywords()` query: `SELECT ... FROM content_chunks
    JOIN content_nodes ON ... WHERE account_id = ? AND (chunk_text LIKE ? OR
    ...) AND status = 'active' ORDER BY retrieval_boost DESC LIMIT ?`.

### Defaults

- Max chunks: 3.
- Keyword matching: case-insensitive LIKE with `%keyword%` for each keyword.
- Ranking: number of matching keywords multiplied by `retrieval_boost`.
- Prompt format: `[From your notes — "{note_title}" > {heading}]\n{chunk_text}`.
- Cold-start seeds remain when no chunks match.

### Verification

```bash
cargo test -p tuitbot-core winning_dna
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 05: Citation Metadata in Assist Responses

**Goal:** Return citation metadata alongside generated content.

### Files to Modify

- `crates/tuitbot-server/src/routes/assist.rs`:
  - Define `Citation` struct: `note_title`, `heading: Option<String>`,
    `chunk_preview: String`, `obsidian_uri: Option<String>`.
  - Extend `resolve_composer_rag_context()` to return
    `Option<(String, Vec<Citation>)>` instead of `Option<String>`.
  - Extend all assist response structs with `citations: Vec<Citation>`
    (serialized as empty array when no citations).
  - Build `obsidian_uri` when deployment_mode is Desktop and source is
    local_fs.
- `crates/tuitbot-server/src/routes/discovery.rs`:
  - Add `resolve_composer_rag_context()` call in `compose_reply`.
  - Pass context to `generate_reply_with_context()`.
  - Extend `ComposeReplyResponse` with `citations: Vec<Citation>`.

### Defaults

- `chunk_preview`: first 120 characters of chunk text, trimmed at word
  boundary.
- `obsidian_uri`: `obsidian://open?vault={last_dir_component}&file={relative_path}`.
  URL-encoded. Only when Desktop + local_fs.
- Empty `citations` array is always included (not omitted).

### Verification

```bash
cargo test -p tuitbot-server
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 06: Account Isolation Audit

**Goal:** Ensure all watchtower queries enforce per-account isolation.

### Files to Modify

- `crates/tuitbot-core/src/storage/watchtower/mod.rs`:
  - `insert_source_context()` — accept `account_id` parameter, remove
    `DEFAULT_ACCOUNT_ID` binding.
  - `get_source_contexts()` — add `WHERE account_id = ?` filter.
  - `upsert_content_node()` — accept `account_id`, bind in INSERT.
  - `get_pending_nodes()` — add `WHERE account_id = ?`.
  - `get_seeds_for_context()` — add `WHERE account_id = ?`.
  - `search_chunks_by_keywords()` — already takes account_id (from Session 04).
  - All new chunk CRUD functions — already scoped (from Session 02).
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`:
  - Thread `account_id` through `ingest_content()` and all callers.
  - `start_watchtower()` passes account_id from config.
- `crates/tuitbot-core/src/automation/seed_worker.rs`:
  - `process_batch()` queries with account_id filter.
  - Seeds inherit account_id from parent node.
- `crates/tuitbot-core/src/context/winning_dna.rs`:
  - `build_draft_context()` — accept `account_id`, pass to seed and chunk
    retrieval queries.

### Defaults

- All functions gain `account_id: &str` parameter.
- `DEFAULT_ACCOUNT_ID` remains as the fallback value passed by callers when
  only one account exists. It is never hardcoded in queries.
- Tests pass `DEFAULT_ACCOUNT_ID` explicitly.

### Verification

Write a test that creates data for two different account IDs and verifies
queries with account_id A return zero results for account_id B.

```bash
cargo test -p tuitbot-core isolation
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 07: Draft Provenance Wiring

**Goal:** Wire provenance through the draft → approval → post pipeline.

### Files to Modify

- `crates/tuitbot-core/src/workflow/draft.rs`:
  - After `build_draft_context()`, capture `relevant_chunks` from
    `DraftContext`.
  - Build `source_chunks_json` from chunk metadata.
  - Determine `source_node_id` (most-referenced node) and `source_seed_id`
    (if a specific seed was selected).
  - Pass provenance fields to `enqueue_for()`.
- `crates/tuitbot-core/src/workflow/queue.rs`:
  - Thread provenance parameters through to storage layer.
- `crates/tuitbot-core/src/automation/approval_poster.rs`:
  - On successful post: set `draft_seeds.used_at` for the seed referenced
    in `source_seed_id`.
  - Set `original_tweets.source_node_id` from the approval item.

### Defaults

- `source_node_id`: the node_id of the first (highest-relevance) chunk.
- `source_seed_id`: set only when a draft was generated from a specific seed
  (e.g., seed-based generation in composer). NULL for automatic RAG.
- `source_chunks_json`: always populated when chunks were used; `'[]'` when
  no chunks contributed.

### Verification

Integration test: ingest a note → run chunker → generate draft via workflow
→ verify approval_queue row has provenance FKs populated.

```bash
cargo test -p tuitbot-core provenance
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 08: Loop-Back Learning

**Goal:** Feed performance data back to chunk retrieval weights.

### Files to Modify

- `crates/tuitbot-core/src/automation/analytics_loop.rs` (or the adapter that
  records performance — trace from `automation/adapters/storage.rs`):
  - After recording performance for a tweet, look up the corresponding
    `approval_queue` record via `posted_tweet_id`.
  - Read `source_chunks_json` to get chunk IDs.
  - Call `update_chunk_retrieval_boost()` for each chunk.
  - If `source_seed_id` is set, update seed `engagement_weight`.
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`:
  - Add `update_chunk_retrieval_boost(pool, chunk_id, new_boost)`.
  - Add `get_chunk_retrieval_boost(pool, chunk_id) -> f64`.
- `crates/tuitbot-core/src/automation/watchtower/loopback.rs`:
  - Extend `LoopBackEntry` with `performance_score: Option<f64>`.
  - When writing loop-back metadata, include performance score.

### Defaults

- Boost formula: `current_boost * (1 + 0.1 * normalized_engagement)`.
- `normalized_engagement`: post's `performance_score / max_performance_score`
  for the account (0.0–1.0).
- Boost range: [0.1, 5.0].
- No decay applied here (recency decay is handled in retrieval ranking).
- Loop-back entry `performance_score` is the raw score, not normalized.

### Verification

Integration test: create a chunk with default boost → simulate performance
recording → verify boost updated correctly.

```bash
cargo test -p tuitbot-core loopback
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 09: Vault Health API

**Goal:** Backend endpoints for vault health dashboard.

### Files to Create

- `crates/tuitbot-server/src/routes/vault.rs` — Route handlers:
  - `GET /api/vault/health` — Per-source aggregate stats.
  - `GET /api/vault/notes` — Paginated note list with fragment/seed counts.
  - `GET /api/vault/fragments` — Fragments for a specific note.
  - `GET /api/vault/seeds` — Seed browser with status filter.

### Files to Modify

- `crates/tuitbot-server/src/routes/mod.rs` — Register vault routes.
- `crates/tuitbot-core/src/storage/watchtower/mod.rs` — Add aggregate queries:
  - `get_vault_health_stats(pool, account_id)` — returns per-source counts.
  - `get_notes_with_counts(pool, account_id, source_id, status, limit, offset)`.
  - `get_seeds_paginated(pool, account_id, status, limit, offset)`.

### Response Shapes

`GET /api/vault/health`:
```json
{
  "sources": [
    {
      "id": 1,
      "source_type": "local_fs",
      "status": "active",
      "last_sync": "2026-03-08T12:00:00Z",
      "error_message": null,
      "note_count": 12,
      "fragment_count": 47,
      "seed_count": 23
    }
  ]
}
```

`GET /api/vault/notes?source_id=1&limit=50&offset=0`:
```json
{
  "notes": [
    {
      "id": 1,
      "title": "Marketing Playbook",
      "relative_path": "marketing/playbook.md",
      "status": "processed",
      "fragment_count": 8,
      "seed_count": 5,
      "ingested_at": "2026-03-08T10:00:00Z"
    }
  ],
  "total": 12
}
```

`GET /api/vault/fragments?node_id=1`:
```json
{
  "fragments": [
    {
      "id": 1,
      "heading_path": "## Key Insight",
      "chunk_preview": "First 200 chars...",
      "retrieval_boost": 2.3,
      "status": "active",
      "usage_count": 3
    }
  ]
}
```

`GET /api/vault/seeds?status=pending&limit=50`:
```json
{
  "seeds": [
    {
      "id": 1,
      "seed_text": "Most startups underestimate...",
      "archetype_suggestion": "contrarian_take",
      "engagement_weight": 0.5,
      "status": "pending",
      "used_at": null,
      "note_title": "Marketing Playbook"
    }
  ],
  "total": 23
}
```

### Verification

```bash
cargo test -p tuitbot-server vault
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 10: From Vault Composer API

**Goal:** Backend support for explicit fragment selection in composer.

### Files to Modify

- `crates/tuitbot-server/src/routes/assist.rs`:
  - Extend `AssistTweetRequest`, `AssistThreadRequest`, `AssistImproveRequest`
    with `fragment_ids: Option<Vec<i64>>`.
  - When `fragment_ids` is present and non-empty:
    1. Fetch chunk text via `get_chunks_by_ids()`.
    2. Validate chunk count <= 3.
    3. Use fetched chunks as explicit RAG context (skip automatic retrieval).
    4. Build citations from the selected chunks.
  - When `fragment_ids` is absent or empty: continue with automatic RAG.
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`:
  - Add `get_chunks_by_ids(pool, account_id, ids: &[i64])` — returns chunks
    owned by the account. Uses parameterized `WHERE IN (?, ?)`.

### Defaults

- Max 3 fragment IDs per request. Return 400 if exceeded.
- Chunks must belong to the requesting account (account_id filter).
- Chunks must have status `active`. Return 404 if any chunk is stale.

### Verification

```bash
cargo test -p tuitbot-server assist
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Session 11: Dashboard — Vault Health Page

**Goal:** Frontend vault health dashboard.

### Files to Create

- `dashboard/src/routes/(app)/vault/+page.svelte` — Vault health page.
- `dashboard/src/lib/components/vault/SourceCard.svelte` — Source status card.
- `dashboard/src/lib/components/vault/NoteList.svelte` — Note browser with
  expand/collapse.
- `dashboard/src/lib/components/vault/FragmentView.svelte` — Fragment detail
  view.
- `dashboard/src/lib/components/vault/SeedList.svelte` — Seed browser.
- `dashboard/src/lib/api/vault.ts` — Vault API client functions.

### Files to Modify

- `dashboard/src/routes/(app)/+layout.svelte` — Add "Vault" nav item after
  Queue, before Settings.
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` — Add
  inline health summary (note count, fragment count, sync time).

### Requirements

- Svelte 5 runes (`$props()`, `$state()`, `$derived()`, `$effect()`).
- Tabs for indentation.
- TailwindCSS for styling.
- Keyboard accessible (j/k, Enter, Tab, /).
- Mobile responsive (single-column below 768px).
- Color coding: green (synced), yellow (syncing), red (error), gray (disabled).
- Design tokens from `app.css`.

### Verification

```bash
cd dashboard && npm run check && npm run build
```

---

## Session 12: Dashboard — From Vault Composer & Citations

**Goal:** Frontend "From Vault" toggle in composer and citation display.

### Files to Create

- `dashboard/src/lib/components/composer/VaultPicker.svelte` — Note/fragment
  selector with search, expand, and checkbox selection.
- `dashboard/src/lib/components/composer/CitationPill.svelte` — Citation
  display component. Clickable pill that expands to show chunk preview.
  Desktop: click opens `obsidian://` URI.

### Files to Modify

- Composer page (locate exact file in session): Add "Auto" / "From Vault"
  radio toggle. When "From Vault" is selected, show `VaultPicker`. Pass
  `fragment_ids` in assist request.
- Discovery page (locate exact file in session): Show `CitationPill`
  components below generated replies when `citations` array is non-empty.
  Show "Vault" badge indicator.

### Requirements

- "From Vault" off by default.
- Max 3 fragment selections.
- Citation pills are keyboard-focusable.
- `obsidian://` links only on Desktop + local_fs.
- Mobile: pills are tap-to-expand (no hover).

### Verification

```bash
cd dashboard && npm run check && npm run build
```

---

## Session 13: Integration Testing & Documentation

**Goal:** End-to-end validation and final documentation.

### Files to Modify

- `docs/architecture.md` — Update Content Source Pipeline section to describe
  the fragment chunking, provenance chain, and loop-back learning.
- `docs/configuration.md` — Document vault-related config options.

### Files to Create

- `docs/vault-loop.md` — User-facing documentation covering:
  - Setting up a vault (Desktop, SelfHost, Cloud).
  - How notes become posts (lifecycle overview).
  - Using "From Vault" in the composer.
  - Understanding citations.
  - Vault health dashboard.
  - Loop-back learning (how performance improves retrieval).
  - Troubleshooting (sync errors, missing fragments, stale chunks).

### Verification (Full CI)

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check && npm run build
```

### Manual E2E Test Plan

1. Ingest a markdown note with H2 headings.
2. Verify `content_chunks` rows created.
3. Wait for seed worker tick — verify `draft_seeds` with `chunk_id`.
4. Compose a tweet using "From Vault" — select a fragment.
5. Verify response includes citations.
6. Approve and post the tweet.
7. Simulate analytics recording.
8. Verify `retrieval_boost` updated on referenced chunks.
9. Verify loopback entry written to source file with performance score.
10. Verify vault health page shows accurate counts.

---

## Session Dependency Graph

```
Session 02 (Chunks)
    ↓
Session 03 (Provenance Schema)
    ↓
Session 04 (Chunk RAG) ← depends on 02
    ↓
Session 05 (Citations) ← depends on 04
    ↓
Session 06 (Isolation) ← independent, can run after 02
    ↓
Session 07 (Provenance Wiring) ← depends on 03, 04
    ↓
Session 08 (Loop-Back) ← depends on 07
    ↓
Session 09 (Vault Health API) ← depends on 02, 03
    ↓
Session 10 (From Vault API) ← depends on 04, 05
    ↓
Session 11 (Vault Health UI) ← depends on 09
    ↓
Session 12 (Composer & Citations UI) ← depends on 10, 11
    ↓
Session 13 (Integration & Docs) ← depends on all
```

Critical path: 02 → 03 → 04 → 05 → 07 → 08 → 09 → 10 → 11 → 12 → 13.
Session 06 (isolation audit) can be parallelized with 04–05.
