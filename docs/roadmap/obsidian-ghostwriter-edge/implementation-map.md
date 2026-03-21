# Implementation Map: Obsidian Ghostwriter Edge

## Phase Overview

| Phase | Sessions | Focus | Deliverables |
|-------|----------|-------|------------|
| Phase 1 | 2-3 | Hook-first thread drafting | Hooks API, hooks panel, hook→thread generation |
| Phase 2 | 4-5 | Enhanced citations & deep-links | Heading-anchor URIs, highlights cache, citation UX |
| Phase 3 | 6-7 | Selection ingress & handoff | Send-selection API, composer auto-open, plugin spec |
| Phase 4 | 8 | Privacy hardening & integration test | Cloud-mode guards, deployment matrix tests, end-to-end |

---

## Phase 1: Hook-First Thread Drafting (Sessions 2-3)

### Session 2: Backend — Hooks API and Seed Exposure

**Goal**: Expose pre-extracted draft seeds via the vault API so the frontend can display hooks per note.

**File changes**:

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Add `get_seeds_for_node(pool, account_id, node_id) -> Vec<DraftSeed>` query function |
| `crates/tuitbot-server/src/routes/vault.rs` | Add `GET /api/vault/notes/{id}/hooks` handler returning `Vec<HookItem>` |
| `crates/tuitbot-server/src/routes/vault.rs` | Add optional `hooks: Vec<HookSummary>` field to `VaultNoteDetail` response |
| `crates/tuitbot-server/src/routes/assist.rs` | Add optional `hook_id: Option<i64>` to `AssistTweetRequest` and `AssistThreadRequest` |
| `crates/tuitbot-server/src/routes/assist.rs` | When `hook_id` is present, include `seed_id` in generated `ProvenanceRef` |
| `crates/tuitbot-core/src/context/winning_dna/mod.rs` | Add `build_draft_context_with_hook()` variant that biases context toward the hook's parent node |

**API contract for hooks endpoint**:
```
GET /api/vault/notes/{id}/hooks
Response: {
  hooks: [{
    seed_id: i64,
    node_id: i64,
    seed_text: string,       // max 200 chars
    archetype: string | null,
    engagement_weight: f64,
    chunk_id: i64 | null,
    status: string
  }]
}
```

**Tests**:
- Hook endpoint returns empty array when no seeds exist for the note.
- Hook endpoint returns seeds scoped to the correct account.
- Hook endpoint returns 404 for non-existent note ID.
- `hook_id` is optional in assist request deserialization (backward compat).
- When `hook_id` is provided, provenance includes `seed_id`.

**CI gate**: `cargo fmt --all && cargo clippy --workspace -- -D warnings && RUSTFLAGS="-D warnings" cargo test --workspace`

### Session 3: Frontend — Hooks Panel and Hook→Thread Flow

**Goal**: Build the hooks UI in the composer and wire it to the generation API.

**File changes**:

| File | Change |
|------|--------|
| `dashboard/src/lib/api/types.ts` | Add `VaultHookItem` interface |
| `dashboard/src/lib/api/client.ts` | Add `vault.noteHooks(id)` method |
| `dashboard/src/lib/components/composer/VaultHooksPanel.svelte` | New component: hook cards with archetype badges, select→generate flow |
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Add hooks tab toggle; load hooks when note is expanded |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Add `handleGenerateFromHook(hookId, nodeId, seedText, format)` method |
| `dashboard/src/lib/components/composer/VaultNoteList.svelte` | Show hook count badge alongside chunk count |

**UX flow**:
1. User searches vault → expands a note.
2. Below chunks, a "Hooks" section shows 1-3 pre-extracted hooks (if seeds exist).
3. Each hook card shows: seed text, archetype badge (e.g., "contrarian_take"), and "Generate Thread" / "Generate Tweet" buttons.
4. Clicking a generation button calls the assist API with the hook text as topic and the note ID as context.
5. Generated content appears in the composer with citation chips.

**Tests**: `cd dashboard && npx vitest run`
- VaultHooksPanel renders hooks when provided.
- VaultHooksPanel shows empty state when no hooks exist.
- FromVaultPanel loads hooks alongside chunks on note expansion.
- ComposerInspector generates from hook with correct API parameters.

---

## Phase 2: Enhanced Citations & Deep-Links (Sessions 4-5)

### Session 4: Heading-Anchor Deep-Links and Citation UX

**Goal**: Citations link to the exact heading section in Obsidian, not just the file.

**File changes**:

| File | Change |
|------|--------|
| `dashboard/src/lib/utils/obsidianUri.ts` | Add `buildObsidianUriWithHeading(vaultPath, relativePath, headingPath)` |
| `dashboard/src/lib/components/composer/CitationChips.svelte` | Use heading-aware URI when `heading_path` is available |
| `dashboard/src/lib/components/composer/CitationChips.svelte` | Show heading in chip detail without requiring expand click |

**Tests**:
- `buildObsidianUriWithHeading` produces correct URI with `&heading=` parameter.
- `buildObsidianUriWithHeading` falls back to file-level URI when heading is empty.
- `buildObsidianUriWithHeading` extracts deepest heading from multi-level path.
- CitationChips renders heading-anchor link on Desktop.

### Session 5: Pre-Computed Highlights Cache

**Goal**: Eliminate LLM round-trip for repeat highlight extraction.

**File changes**:

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Add `cached_highlights` column to `content_nodes` (nullable TEXT, JSON array) |
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Add `get_cached_highlights(pool, node_id)` and `set_cached_highlights(pool, node_id, highlights)` functions |
| `crates/tuitbot-core/src/automation/seed_worker.rs` | After hook extraction, also extract and cache highlights |
| `crates/tuitbot-server/src/routes/assist.rs` | `assist_highlights` checks cache before LLM call; populates cache on miss |
| Database migration | `ALTER TABLE content_nodes ADD COLUMN cached_highlights TEXT DEFAULT NULL` |

**Cache invalidation**: When `content_hash` changes during re-index (`watchtower::upsert_content_node`), set `cached_highlights = NULL`.

**Tests**:
- Highlights endpoint returns cached data without LLM call when cache exists.
- Cache is invalidated when node content hash changes.
- Seed worker populates highlights cache during node processing.

---

## Phase 3: Selection Ingress & Handoff (Sessions 6-7)

### Session 6: Selection Ingress API

**Goal**: Define and implement the `POST /api/vault/send-selection` endpoint for external clients.

**File changes**:

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Add `vault_selections` table: `id`, `account_id`, `session_id` (UUID), `vault_name`, `file_path`, `selected_text`, `heading_context`, `created_at`, `expires_at` |
| `crates/tuitbot-server/src/routes/vault.rs` | Add `POST /api/vault/send-selection` handler |
| `crates/tuitbot-server/src/routes/vault.rs` | Add `GET /api/vault/selection/{session_id}` handler |
| `crates/tuitbot-server/src/ws.rs` | Add `SelectionReceived { session_id }` WebSocket event |
| Database migration | Create `vault_selections` table with TTL column |

**TTL enforcement**: A cleanup task runs on server start and hourly, deleting rows where `expires_at < now()`.

**Tests**:
- Send-selection stores data and returns session_id.
- Selection expires after 30 minutes.
- Selection is account-scoped.
- Cloud mode: GET selection returns only generated content, not raw `selected_text`.

### Session 7: Composer Auto-Open and Selection Pre-Load

**Goal**: Dashboard receives selection events and pre-populates the composer.

**File changes**:

| File | Change |
|------|--------|
| `dashboard/src/lib/api/client.ts` | Add `vault.getSelection(sessionId)` method |
| `dashboard/src/lib/api/types.ts` | Add `VaultSelection` interface |
| `dashboard/src/lib/stores/websocket.ts` | Handle `SelectionReceived` event |
| `dashboard/src/routes/compose/+page.svelte` | Read `?selection=uuid` query param and pre-load selection |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Add `handleGenerateFromSelection(sessionId)` method |

**UX flow**:
1. External client (Obsidian plugin) POSTs selection to TuitBot.
2. WebSocket event notifies the dashboard.
3. Dashboard navigates to `/compose?selection=uuid`.
4. Composer loads the selection and offers "Generate Tweet" / "Generate Thread".

**Tests**:
- Compose page reads selection query param and loads selection data.
- Selection pre-populates the composer text area (tweet mode) or generation topic (thread mode).

---

## Phase 4: Privacy Hardening & Integration Test (Session 8)

### Session 8: Cloud-Mode Guards and End-to-End Verification

**Goal**: Ensure all Ghostwriter features respect deployment-mode privacy boundaries. Run end-to-end flow verification.

**File changes**:

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/routes/vault.rs` | Add deployment-mode check to `send-selection` and `get-selection` responses |
| `crates/tuitbot-server/src/routes/vault.rs` | Integration tests per deployment mode |
| `dashboard/tests/unit/` | Integration tests for hook→thread→cite→deep-link flow |

**Test matrix**:

| Test | Desktop | Self-host | Cloud |
|------|---------|-----------|-------|
| Hooks API returns seeds | Pass | Pass | Pass |
| Note detail includes hooks | Pass | Pass | Pass |
| Citations include heading-anchor links | Pass (rendered) | Pass (stored, not rendered) | Pass (stored, not rendered) |
| Selection ingress stores text | Pass | Pass | Pass |
| Selection GET returns raw text | Pass | Pass | Filtered (generated content only) |
| Obsidian deep-link opens | Pass | N/A | N/A |
| Provenance includes seed_id | Pass | Pass | Pass |

**CI gate**: Full CI checklist + frontend tests + coverage thresholds.

---

## Dependency Graph

```
Session 2 (Hooks API)
    │
    ├──→ Session 3 (Hooks Panel) ──→ Session 4 (Deep-Links)
    │                                      │
    │                                      ├──→ Session 5 (Highlights Cache)
    │                                      │
    └──→ Session 6 (Selection API) ──→ Session 7 (Composer Auto-Open)
                                              │
                                              ▼
                                    Session 8 (Integration)
```

Sessions 2→3 are strictly sequential (backend before frontend).
Sessions 4-5 can proceed in parallel with Sessions 6-7.
Session 8 depends on all prior sessions.

## Risk Mitigations

| Risk | Mitigation | Fallback |
|------|------------|----------|
| Hook quality varies by note length/quality | Show engagement_weight as quality signal; allow manual hook editing | Users fall back to chunk selection flow |
| Highlights cache grows large | Cache only for notes with >100 chars body_text; prune on re-index | Disable cache; always use LLM extraction |
| Selection ingress abuse (spam) | Rate limit: 10 selections per minute per account; TTL enforcement | Disable endpoint via feature flag |
| Heading-anchor deep-links fail in some Obsidian versions | Graceful degradation: fall back to file-level link | Already the current behavior |
| Cloud-mode filtering misses a new endpoint | Audit checklist in Session 8; test matrix per deployment mode | Add linting rule that flags `chunk_text` or `body_text` in Serialize derives |
