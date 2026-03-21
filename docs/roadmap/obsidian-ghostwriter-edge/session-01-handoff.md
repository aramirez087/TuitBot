# Session 01 Handoff: Obsidian Ghostwriter Edge

## What Changed

Six roadmap artifacts created under `docs/roadmap/obsidian-ghostwriter-edge/`. Zero source code changes.

### Artifacts Created

| File | Purpose |
|------|---------|
| `epic-charter.md` | Mission, competitive positioning vs Typefully, success metrics, non-goals, constraints |
| `current-state-audit.md` | Detailed audit of all 15 repository anchor files with gap analysis |
| `ghostwriter-architecture.md` | Target architecture: hooks API, hooks panel, heading-anchor deep-links, selection ingress, provenance enhancement |
| `privacy-and-deployment-matrix.md` | Per-mode behavior matrix (Desktop/Self-host/Cloud) with privacy invariants |
| `implementation-map.md` | Phased plan: Sessions 2-8, file-level change lists, dependency graph, test requirements |
| `session-01-handoff.md` | This file |

## Decisions Made

### Decision 1: Extend existing `chunk_id` + `heading_path` model

**Chose**: Enrich the existing heading-level chunk model with hook exposure and heading-anchor deep-links.
**Rejected**: Byte-offset block addressing, paragraph-level sub-chunking, line-range addressing.
**Rationale**: The current `heading_path` model captures the semantic units users think in (markdown sections). Adding byte offsets would break the retrieval pipeline and existing provenance links without proportional user value. If paragraph-level granularity proves necessary after Phase 1, it can be added as an additive `paragraph_index` field without re-IDing existing chunks.

### Decision 2: Hook-first thread drafting via existing seed infrastructure

**Chose**: Expose `draft_seeds` table contents through a new `GET /api/vault/notes/{id}/hooks` endpoint and a hooks panel in the composer.
**Rejected**: New hook extraction pipeline, real-time hook generation on note expansion.
**Rationale**: `seed_worker.rs` already extracts 1-3 tweetable hooks per note with format suggestions. The infrastructure exists — it just needs an API surface and UI. Pre-computed hooks provide instant display (no LLM wait).

### Decision 3: Privacy boundary — seed text is derived content

**Chose**: Treat seed text (max 200 chars, LLM-generated) as derived content safe for all deployment modes.
**Rejected**: Mode-gating seed text (would cripple the hook-first flow in Cloud mode).
**Rationale**: Seeds are LLM transformations of source material, analogous to titles or summaries. They do not contain verbatim excerpts. The existing privacy invariant (no raw `body_text` or `chunk_text` in API responses) is preserved.

### Decision 4: Selection handoff via API, not sync daemon

**Chose**: `POST /api/vault/send-selection` with 30-minute TTL transient storage.
**Rejected**: Background sync daemon, clipboard bridge, file-system watcher.
**Rationale**: Per operator constraints — "minimal: command-driven selection handoff, no sync engine, no background daemon." The API-first approach allows any client (Obsidian plugin, browser extension, CLI) to push selections.

### Decision 5: Heading-anchor deep-links via Obsidian URI spec

**Chose**: Append `&heading=<heading>` to existing `obsidian://open` URIs.
**Rejected**: Custom URI scheme, Obsidian protocol handler registration.
**Rationale**: Obsidian natively supports the `heading` parameter in its URI scheme. This is a one-line change to `buildObsidianUri()` that provides section-level navigation for free.

## Open Risks

| # | Risk | Severity | Mitigation Plan |
|---|------|----------|-----------------|
| 1 | **Hook quality depends on seed_worker LLM quality** | Medium | Session 3 allows manual hook editing before generation; engagement_weight surfaces quality signal; users can fall back to chunk selection |
| 2 | **Obsidian heading URI behavior varies across versions** | Low | Graceful degradation to file-level link; test with Obsidian v1.4+ |
| 3 | **Cloud-mode privacy filtering requires vigilance on new endpoints** | Medium | Session 8 audit checklist; test matrix per deployment mode; linting rule for `chunk_text`/`body_text` in Serialize |
| 4 | **Selection ingress TTL cleanup under load** | Low | Hourly cleanup cron + on-insert check; rate limit 10 selections/min/account |
| 5 | **Highlights cache staleness if re-index doesn't clear it** | Low | Cache invalidation is tied to `content_hash` change in `upsert_content_node`; tested in Session 5 |

## Session 2 Inputs

Session 2 must consume the following artifacts to begin implementation:

1. **`ghostwriter-architecture.md` § "1. Hooks API Endpoint"**: Full API contract for `GET /api/vault/notes/{id}/hooks`.
2. **`ghostwriter-architecture.md` § "3. Enhanced Note Detail Response"**: Schema for adding `hooks` field to existing note detail response.
3. **`ghostwriter-architecture.md` § "5. Hook-Originated Provenance"**: `hook_id` parameter addition to assist request structs.
4. **`implementation-map.md` § "Session 2"**: Exact file change list and test requirements.
5. **`privacy-and-deployment-matrix.md` § "Hook/Seed Access"**: Confirmation that seed text is safe for all deployment modes.
6. **`current-state-audit.md` § "1. Vault Ingestion Pipeline"**: `DraftSeed` struct definition and seed worker behavior for context.

### Session 2 Scope (Summary)

- Add `get_seeds_for_node()` query to `watchtower/mod.rs`.
- Add `GET /api/vault/notes/{id}/hooks` handler to `vault.rs`.
- Add optional `hooks` field to `VaultNoteDetail` response.
- Add optional `hook_id` to `AssistTweetRequest` and `AssistThreadRequest`.
- When `hook_id` is present, populate `seed_id` in generated `ProvenanceRef`.
- All changes pass the CI checklist: `cargo fmt --all && cargo clippy --workspace -- -D warnings && RUSTFLAGS="-D warnings" cargo test --workspace`.

### Session 2 Exit Criteria

- `GET /api/vault/notes/{id}/hooks` returns seeds for a note (tested).
- `VaultNoteDetail` includes hooks when seeds exist (tested).
- Assist endpoints accept `hook_id` without breaking existing callers (backward compat tested).
- Provenance includes `seed_id` when `hook_id` is provided (tested).
- Zero compilation warnings. All tests pass on macOS, Linux, Windows.
