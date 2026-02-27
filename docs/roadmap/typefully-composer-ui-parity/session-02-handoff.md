# Session 02 Handoff

**Date:** 2026-02-27
**Session:** 02 — Data Model & API Contract
**Status:** Complete
**Next Session:** 03 — Thread Composer Component

---

## What Changed

### New Files

| File | Purpose |
|------|---------|
| `crates/tuitbot-core/src/content/thread.rs` | `ThreadBlock` struct, `ThreadBlocksPayload`, `ThreadBlockError` enum, `validate_thread_blocks()`, `serialize_blocks_for_storage()`, `deserialize_blocks_from_content()`. 18 unit tests. |
| `crates/tuitbot-server/tests/compose_contract_tests.rs` | 24 integration tests covering legacy compatibility, blocks compose, blocks validation rejections, draft blocks, and edge cases. |
| `docs/roadmap/typefully-composer-ui-parity/session-02-api-contract.md` | API contract documentation with schema, storage format, endpoint changes, validation rules. |
| `docs/roadmap/typefully-composer-ui-parity/session-02-handoff.md` | This file. |

### Modified Files

| File | Change Summary |
|------|----------------|
| `crates/tuitbot-core/src/content/mod.rs` | Added `pub mod thread;` and re-exports for all thread types and functions. |
| `crates/tuitbot-server/src/routes/content/compose.rs` | Added `ThreadBlockRequest` struct. Refactored `compose()` handler into three flow functions: `compose_tweet_flow`, `compose_thread_legacy_flow`, `compose_thread_blocks_flow`. Added `persist_content` shared helper. `ComposeRequest` gains optional `blocks` field. |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | `CreateDraftRequest` gains optional `blocks` field. `EditDraftRequest.content` becomes `Option<String>`, adds `blocks` field. Both handlers validate blocks via core when present. Added `validate_draft_content()` helper. |
| `crates/tuitbot-server/src/routes/content/mod.rs` | Added `ThreadBlockRequest` to re-exports. |
| `dashboard/src/lib/api.ts` | Added `ThreadBlock`, `ThreadBlocksPayload` interfaces. Added `parseThreadContent()`, `isBlocksPayload()` helpers. Updated `ComposeRequest` with optional `blocks`. Updated `api.drafts.create()` and `api.drafts.edit()` signatures. |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D-1 | ThreadBlock struct lives in `tuitbot-core`, not `tuitbot-server` | Per three-layer architecture: domain types in core, reusable by CLI/MCP/server. Server defines `ThreadBlockRequest` as API boundary type. |
| D-2 | Blocks serialized to versioned JSON wrapper `{"version":1,"blocks":[...]}` | Unambiguously distinguishable from legacy `["tweet1","tweet2"]` format. Object vs array detection is trivial. No DB migration needed. |
| D-3 | Validation in core, request parsing in server | `validate_thread_blocks()` in core is framework-agnostic. Server handles Axum deserialization and maps `ThreadBlockError` to `ApiError::BadRequest`. |
| D-4 | Per-block media validation is count-only for now | Validates max 4 per block. Actual file type/existence is a media upload concern, not a compose concern. |
| D-5 | `EditDraftRequest.content` becomes `Option<String>` | Backwards compatible: existing `{"content": "..."}` deserializes identically as `Some("...")`. Handler requires at least one of `content` or `blocks`. |
| D-6 | `compose()` refactored into three flow functions | `compose_tweet_flow`, `compose_thread_legacy_flow`, `compose_thread_blocks_flow` keep the match arms clean. `persist_content` shared helper avoids duplication for tweet/legacy-thread storage. |
| D-7 | Per-block media flattened for approval queue | Approval queue stores a single `media_paths` JSON array. When blocks have per-block media, they are collected into a flat list ordered by block order. |

---

## Open Risks

| # | Risk | Mitigation |
|---|------|------------|
| R-1 | Thread publishing (autopilot) does not yet understand blocks format | The `publish` workflow in `tuitbot-core::workflow::publish.rs` will need updating when Session 03+ integrates the full compose UI. Currently, publishing reads `content` as either plain text or `["t1","t2"]`. The new blocks format won't cause errors (it'll be treated as an opaque string), but won't be properly split into individual tweets for posting. This is acceptable since blocks are only used via the compose modal which goes through the approval queue. |
| R-2 | CalendarItem returns raw content, frontend must parse | The `CalendarItem` serialization includes the raw `content` field. Existing frontend code that displays thread content as-is will show the JSON wrapper. Session 03 should use `parseThreadContent()` in calendar/draft list views. |

---

## Test Coverage

| Suite | Tests | Status |
|-------|-------|--------|
| `tuitbot-core::content::thread` (unit) | 18 | Pass |
| `compose_contract_tests` (integration) | 24 | Pass |
| Full workspace (`cargo test --workspace`) | All existing | Pass |

---

## Exact Inputs for Session 03

### Documents to Read First

| File | Section | Purpose |
|------|---------|---------|
| `docs/roadmap/typefully-composer-ui-parity/charter.md` | A-1, A-4 | ThreadComposer component design, side-panel preview |
| `docs/roadmap/typefully-composer-ui-parity/session-02-api-contract.md` | Full | API schema and integration types |
| `docs/roadmap/typefully-composer-ui-parity/session-02-handoff.md` | This file | Context and risks |
| `docs/roadmap/typefully-composer-ui-parity/session-execution-map.md` | Session 03 section | File targets and exit criteria |

### Source Files to Read

| File | Purpose |
|------|---------|
| `dashboard/src/lib/components/ComposeModal.svelte` | Current compose modal (787 lines). Session 03 extracts thread mode to ThreadComposer. |
| `dashboard/src/lib/api.ts` | `ThreadBlock` interface, `parseThreadContent()`, `isBlocksPayload()`, updated draft methods. |
| `dashboard/src/routes/(app)/drafts/+page.svelte` | Draft list page — needs to handle blocks payload display. |

### Integration Points for Session 03

1. **`ThreadBlock` TypeScript type** — `dashboard/src/lib/api.ts` exports `ThreadBlock` for use by `ThreadComposer.svelte`.
2. **`parseThreadContent()`** — Detect and parse blocks from draft content when pre-populating the thread editor.
3. **`blocks` field on `ComposeRequest`** — ThreadComposer constructs `ThreadBlock[]` with client-generated UUIDs and sends via `api.content.compose()`.
4. **Draft editing with blocks** — ThreadComposer saves/loads drafts with block structure preserved via `api.drafts.create(type, content, source, blocks)` and `api.drafts.edit(id, undefined, blocks)`.
5. **`isBlocksPayload()`** — ComposeModal detects whether a draft uses blocks format when opening for editing, to route to ThreadComposer vs legacy editor.

### Key Constraints for Session 03

1. **ComposeModal stays under 400 lines** — Extract thread composition to `ThreadComposer.svelte`.
2. **Preserve tweet compose behavior** — All existing tweet flows (open, type, attach media, schedule, submit) must work unchanged.
3. **Client-generated block IDs** — Use `crypto.randomUUID()` for block IDs. Stable across edits.
4. **Minimum 2 blocks for thread** — UI should enforce >= 2 blocks before submit (matching API validation).
5. **Side-panel preview** — Per charter A-4, use a side panel for preview rather than inline WYSIWYG.
