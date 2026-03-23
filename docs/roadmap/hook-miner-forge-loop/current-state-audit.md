# Current State Audit — Hook Miner + Forge Loop

**Date:** 2026-03-22
**Author:** Session 01 (automated)
**Scope:** Every subsystem that Hook Miner or Forge touches, with shipped vs. gap classification.

---

## 1. Obsidian Selection Ingress — Fully Shipped

**Plugin** (`plugins/obsidian-tuitbot/src/main.ts`):
- Two commands: `tuitbot:send-selection` (highlighted text + heading context) and `tuitbot:send-block` (block at cursor line).
- Settings: configurable server URL, API token path with home-dir expansion, token caching.
- Sends `GhostwriterPayload` via POST with vault_name, file_path, selected_text, heading_context, selection lines, note_title, frontmatter_tags.

**Server** (`crates/tuitbot-server/src/routes/vault/selections.rs`):
- `POST /api/vault/send-selection`: cloud mode privacy gate, rate limiting (10/min/account), validation (vault_name 1–255 chars, file_path ends with `.md`, selected_text 1–10k chars), identity resolution via `resolve_selection_identity()`, 30-minute expiring session, `SelectionReceived` WebSocket event.
- `GET /api/vault/selection/{session_id}`: returns selection + auto-expands graph neighbors via `resolve_graph_suggestions()` when `resolved_node_id` is present. Cloud mode omits `selected_text`. Returns `graph_state` enum and `frontmatter_tags`.

**No gaps relevant to this epic.** Selection ingress is complete and stable.

---

## 2. Graph-Aware Related Notes — Fully Shipped (Backlink Synthesizer)

**Core** (`crates/tuitbot-core/src/context/graph_expansion.rs`):
- `expand_graph_neighbors()`: 1-hop graph walk — outgoing links, incoming backlinks, shared-tag neighbors.
- Scoring: `3×direct + 2×backlinks + 1×shared_tags + 0.5×chunk_boost`, capped at `max_neighbors` (default 8).
- `SuggestionReason`: LinkedNote, Backlink, MutualLink, SharedTag.
- `SuggestionIntent`: ProTip, Counterpoint, Evidence, Related — heuristic on edge labels.
- Returns `GraphNeighbor` with node metadata, best chunk, 120-char snippet, heading_path.

**Schema** (migration 20260321):
- `note_edges` and `note_tags` tables for graph relationships.
- Provenance `edge_type` and `edge_label` columns on `vault_provenance_links`.

**Dashboard** (`dashboard/src/lib/components/composer/GraphSuggestionCards.svelte`):
- Accept/dismiss/restore UX for graph neighbors.
- Synthesis toggle, slot targeting for thread insertion.

**No gaps relevant to this epic.** Hook Miner builds on top of accepted neighbors as evidence sources.

---

## 3. Hook Generation — Shipped (Generic Path Only)

**Server** (`crates/tuitbot-server/src/routes/assist/hooks.rs`):
- `POST /api/assist/hooks`: accepts `{ topic, selected_node_ids?, session_id? }`, returns `{ hooks, topic, vault_citations }`.

**Core** (`crates/tuitbot-core/src/content/generator/mod.rs`):
- `ContentGenerator::generate_hooks(topic, rag_context)`: selects 5 differentiated hook styles (question, contrarian_take, tip, stat, teaser), formats system prompt with voice/persona/audience, injects RAG context, calls LLM with structured parsing.
- Returns 5 `HookOption { style, text, char_count, confidence }`.
- RAG resolution: if `session_id` → `resolve_selection_rag_context()` (selection text + draft context); if `selected_node_ids` → `resolve_composer_rag_context()` (prompt block from vault); both combined if available.

**Dashboard** (`dashboard/src/lib/components/composer/HookPicker.svelte`):
- Select hook → format toggle → generate content.

**Gap for Hook Miner:** The current hook generation is style-based (5 generic styles), not evidence-based. Hook Miner replaces this first step with 3 mined angles backed by evidence from graph neighbors, preserving the generic path as fallback for sparse notes.

---

## 4. Provenance Propagation — Fully Shipped

**Storage** (`crates/tuitbot-core/src/storage/provenance.rs`):
- `vault_provenance_links` table: polymorphic `entity_type`/`entity_id`.
- Supported entity types: `"approval_queue"`, `"scheduled_content"`, `"original_tweet"`, `"thread"`.
- Fields: `node_id`, `chunk_id`, `seed_id`, `source_path`, `heading_path`, `snippet`, `edge_type`, `edge_label`.
- Functions: `insert_links_for()`, `get_links_for()`, `copy_links_for()`, `delete_links_for()`.

**Lifecycle:**
- Compose endpoint accepts `provenance` parameter.
- Frontend passes neighbor provenance through `ongenerate` callback.
- Approval poster copies provenance from `approval_queue` → `original_tweet` via `copy_links_for()`.

**Gap for Forge:** Provenance propagates to `original_tweet` entity type but the approval poster does not copy provenance to `thread` entity type for thread posts. This blocks Forge from tracing thread performance back to source notes.

---

## 5. Publish Writeback (Loopback) — Shipped (Tweet-Only Effective)

**Core** (`crates/tuitbot-core/src/automation/watchtower/loopback.rs`):
- `execute_loopback(pool, node_id, tweet_id, url, content_type)`: resolves content node → source file → writes `tuitbot:` YAML array to frontmatter.
- `LoopBackEntry` fields: `tweet_id`, `url`, `published_at` (ISO-8601 UTC), `content_type` ("tweet"/"thread"/"reply"), `status` (default "posted"), `thread_url` (optional).
- `write_metadata_to_file()`: idempotent (deduplicates by tweet_id), preserves existing frontmatter.
- Only supports `local_fs` sources.

**Called from** `approval_poster.rs` via `execute_loopback_for_provenance()`:
- Fetches provenance links, deduplicates by node_id, calls `execute_loopback()` per unique node.

**Gap for Forge:**
- `thread_url` is never populated by the approval poster (always `None`).
- No analytics fields exist in `LoopBackEntry` — Forge must add `impressions`, `likes`, `retweets`, `replies`, `engagement_rate`, `performance_score`, `synced_at`.
- No mechanism to update existing entries — loopback is append-only.

---

## 6. Analytics Storage — Shipped (Per-Tweet Snapshot Only)

**Storage** (`crates/tuitbot-core/src/storage/analytics/`):
- `tweet_performance`: `tweet_id` (PK), `likes_received`, `retweets_received`, `replies_received`, `impressions`, `performance_score`, `measured_at`. Upsert via `ON CONFLICT DO UPDATE`.
- `reply_performance`: same shape minus `retweets_received`. Upsert idempotent.
- Additional tables: `engagement_metrics`, `reach_snapshots`, `content_scores`, `best_times`.

**Gaps for Forge:**
- **No thread-level aggregation.** Each tweet in a thread is measured independently with no linkage to the parent thread.
- **Snapshot-only.** Only latest metrics per tweet — no time-series for engagement trends.
- **No link to provenance.** Cannot query "which source notes produced the best-performing tweets."
- Forge needs aggregated thread metrics (sum of child tweet metrics) to write meaningful thread-level performance back to source notes.

---

## 7. Thread Infrastructure — Partially Shipped

**Storage** (`crates/tuitbot-core/src/storage/threads.rs`):
- `threads` table: `id`, `topic`, `tweet_count`, `root_tweet_id` (string, not FK), `status` ("sent"/"partial"/"failed"), `created_at`.
- `thread_tweets` table: `id`, `thread_id` (FK), `position` (0-indexed), `tweet_id`, `content`, `created_at`.
- `original_tweets` table: standalone tweet records with `source_node_id`.
- CRUD: `insert_thread_for()`, `insert_thread_tweets_for()` (transactional batch), `count_threads_this_week_for()`.

**Thread loop generator:** Creates threads via LLM, persists to `threads`/`thread_tweets` correctly.

**Critical gaps:**

1. **Approval poster does not post reply chains.** In `approval_poster.rs`, `thread_tweet` action type routes to `post_tweet()` — each tweet is posted as a standalone tweet, not as a reply to the previous tweet in the chain. There is no `in_reply_to_tweet_id` threading.

2. **Ghostwriter compose → approve → publish does NOT create `thread`/`thread_tweets` records.** When a user composes a thread through the Ghostwriter flow (VaultSelectionReview → HookPicker → compose), the resulting approval queue items have `action_type = "thread_tweet"` but no `thread` record is created. Only the thread loop generator creates proper thread records.

3. **Provenance does not propagate to `thread` entity type.** The approval poster's `propagate_provenance()` creates `original_tweet` records and copies provenance links to `entity_type = "original_tweet"`. It does not create corresponding `thread` entity links even when posting thread tweets.

---

## Summary: Shipped vs. New Scope

| Subsystem | Status | Hook Miner Impact | Forge Impact |
|-----------|--------|-------------------|--------------|
| Obsidian selection ingress | Shipped | None (consumed as-is) | None |
| Graph-aware neighbors | Shipped | Input (accepted neighbors → evidence) | None |
| Hook generation | Shipped (generic) | **Replaced** (3 mined angles vs 5 generic) | None |
| Provenance propagation | Shipped | Extended (angle provenance) | Extended (thread entity) |
| Publish loopback | Shipped (tweet-only) | None | **Extended** (analytics fields, update path) |
| Analytics storage | Shipped (per-tweet) | None | **Extended** (thread aggregation) |
| Thread infrastructure | Partial | None | **Prerequisite fix** (reply-chain posting) |
