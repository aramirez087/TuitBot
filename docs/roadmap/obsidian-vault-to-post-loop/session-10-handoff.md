# Session 10 Handoff — Reply & Automation Vault Integration

## What Changed

### Created: `crates/tuitbot-server/src/routes/rag_helpers.rs`

Extracted `resolve_composer_rag_context()` from `assist.rs` into a shared `pub(crate)` module. Both `assist.rs` and `discovery.rs` now import from this shared helper, eliminating code duplication.

### Modified: `crates/tuitbot-core/src/automation/loop_helpers.rs`

- Added `ReplyOutput` struct: `{ text: String, vault_citations: Vec<VaultCitation> }`
- Added `generate_reply_with_rag()` default method on `ReplyGenerator` trait
- Default implementation delegates to `generate_reply()` with empty citations — backward compatible with all existing mocks
- Added test for `ReplyOutput` struct creation

### Modified: `crates/tuitbot-core/src/automation/adapters/llm.rs`

- Added `VaultAwareLlmReplyAdapter` struct with pre-built RAG prompt and cached citations
- Implements `ReplyGenerator` with vault-aware generation via `generate_reply_with_context()`
- Overrides `generate_reply_with_rag()` to return cached vault citations
- RAG prompt is built once at construction, not per-tweet

### Modified: `crates/tuitbot-core/src/workflow/mod.rs`

- Added `vault_citations: Vec<VaultCitation>` field to `DraftResult::Success`
- Uses `#[serde(default, skip_serializing_if = "Vec::is_empty")]` for backward compatibility
- Imported `VaultCitation` type

### Modified: `crates/tuitbot-core/src/workflow/draft.rs`

- Extracts `vault_citations` from `DraftContext` after building RAG context
- Passes citations through to each `DraftResult::Success` entry
- Citations are shared across all candidates in a batch (same RAG context)

### Modified: `crates/tuitbot-server/src/routes/assist.rs`

- Updated `AssistReplyRequest` to include optional `selected_node_ids`
- Updated `AssistReplyResponse` to include `vault_citations` (skipped when empty)
- Updated `assist_reply()` handler to use `resolve_composer_rag_context` and `generate_reply_with_context`
- Switched to shared RAG helper import
- Added tests for reply request deserialization and response serialization

### Modified: `crates/tuitbot-server/src/routes/discovery.rs`

- Updated `ComposeReplyRequest` to include optional `selected_node_ids`
- Updated `ComposeReplyResponse` to include `vault_citations`
- Updated `compose_reply()` to use vault context via shared RAG helper
- Updated `QueueReplyRequest` to include optional `provenance: Vec<ProvenanceRef>`
- Updated `queue_reply()` to use `enqueue_with_provenance_for` when provenance is present
- Added test module with 4 deserialization/serialization tests

### Modified: `crates/tuitbot-core/src/automation/discovery_loop.rs`

- Updated `process_tweet()` to call `generate_reply_with_rag()` instead of `generate_reply()`
- Extracts `reply_text` from `ReplyOutput`

### Modified: `crates/tuitbot-core/src/automation/target_loop.rs`

- Updated `process_target_tweet()` to call `generate_reply_with_rag()` with `mention_product: false`
- Same pattern as discovery loop

### Modified: `crates/tuitbot-core/src/automation/mentions_loop.rs`

- Updated `process_mention()` to call `generate_reply_with_rag()` with `mention_product: true`
- Same pattern as discovery loop

### Modified: `crates/tuitbot-server/src/routes/mod.rs`

- Added `pub(crate) mod rag_helpers;`

### Created: `docs/roadmap/obsidian-vault-to-post-loop/reply-integration.md`

Documents the complete reply surface behavior matrix, architecture, API changes, and intentional differences between surfaces.

## Files Created

- `crates/tuitbot-server/src/routes/rag_helpers.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/reply-integration.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-10-handoff.md`

## Files Modified

- `crates/tuitbot-core/src/automation/loop_helpers.rs`
- `crates/tuitbot-core/src/automation/adapters/llm.rs`
- `crates/tuitbot-core/src/automation/discovery_loop.rs`
- `crates/tuitbot-core/src/automation/target_loop.rs`
- `crates/tuitbot-core/src/automation/mentions_loop.rs`
- `crates/tuitbot-core/src/workflow/mod.rs`
- `crates/tuitbot-core/src/workflow/draft.rs`
- `crates/tuitbot-server/src/routes/mod.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/discovery.rs`

## Test Results

- `cargo fmt --all --check` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `RUSTFLAGS="-D warnings" cargo test --workspace` — all passed, 0 failed

## What Remains

| Item | Scope | Status |
|------|-------|--------|
| Vault health page (`/vault`) | Source status, sync indicators | Session 11 |
| Settings vault health summary | Inline source status | Session 11 |
| Obsidian URI deep linking from citations | Click chip → open in Obsidian | Session 12 |
| Wire VaultAwareLlmReplyAdapter into watchtower runtime | Server/CLI loop wiring | Future |
| Automation provenance to approval queue | Store citations when loops use approval mode | Future |
| ComposeWorkspace extraction/split | File is 955+ lines (limit 400) | Tech debt |
| Thread-level loop-back | Write all tweet_ids from thread into source note | Future |
| Scheduled content provenance | Store provenance when scheduling | Future |
| Analytics loop-back | Update chunk retrieval boost from tweet performance | Future |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| VaultAwareLlmReplyAdapter not wired into watchtower yet | Expected | Low | Existing `LlmReplyAdapter` still works. Loops use default method (no vault context) until wiring session. The adapter is ready and tested. |
| Automation loops don't persist citations to approval queue | Expected | Low | Loops currently post directly via `PostSender::send_reply()`. When approval mode is wired into loops (future session), `ReplyOutput.vault_citations` is available for queue insertion. |
| `DraftResult::Success` serialization change | Low | Low | `skip_serializing_if = "Vec::is_empty"` ensures no change for empty citations. Existing consumers use `..` pattern matching. |
| RAG helper extraction might miss edge cases | Low | Low | Same function body, just moved. Imports verified. All 3 existing tests pass against the shared helper. |

## Decisions Made

1. **Shared RAG helper over pub function** — Created `routes/rag_helpers.rs` rather than making `assist.rs::resolve_composer_rag_context` public. Cleaner separation, both route modules import from the same place.

2. **Default method on trait** — `generate_reply_with_rag()` is a default method returning `ReplyOutput` with empty citations. This avoids breaking all mock implementations across 40+ tests. Only `VaultAwareLlmReplyAdapter` overrides it.

3. **Pre-built RAG in adapter** — `VaultAwareLlmReplyAdapter` takes the RAG prompt at construction, not per-tweet. This prevents per-tweet DB queries in hot automation loops. The tradeoff is that vault context doesn't refresh mid-iteration, which is acceptable since iterations are short.

4. **Target loop: vault context yes, product mention no** — Vault context grounds replies in domain expertise. Product mention stays false for genuine engagement. Documented as intentional in `reply-integration.md`.

5. **`QueueReplyRequest` accepts `ProvenanceRef` directly** — Rather than requiring the frontend to convert from `VaultCitation` to `ProvenanceRef`, the API accepts `ProvenanceRef` directly. The frontend already has this type from `dashboard/src/lib/api/types.ts`.

6. **Draft phase 3 merged into phase 1** — `DraftResult::Success` vault_citations was needed for compilation. Applied during Phase 1 instead of waiting for Phase 3.

## Inputs for Next Session

- Reply integration doc: `docs/roadmap/obsidian-vault-to-post-loop/reply-integration.md`
- `VaultAwareLlmReplyAdapter` is ready but not yet wired into `watchtower` runtime
- Dashboard `CitationChips` component (from Session 9) is reusable for reply-level citation display
- Discovery compose and queue endpoints now return/accept vault data — frontend integration pending
- Key files for vault health page:
  - `crates/tuitbot-server/src/routes/vault.rs` — existing vault API endpoints
  - `crates/tuitbot-server/src/routes/sources.rs` — content source management
  - `dashboard/src/lib/api/client.ts` — `api.vault.*` methods
