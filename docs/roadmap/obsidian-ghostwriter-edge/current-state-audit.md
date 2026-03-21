# Current State Audit: Obsidian Ghostwriter Edge

## 1. Vault Ingestion Pipeline

### Storage Schema

**`content_nodes` table** (via `crates/tuitbot-core/src/storage/watchtower/mod.rs`):
- Stores ingested Obsidian/markdown notes: `id`, `account_id`, `source_id`, `title`, `body_text`, `relative_path`, `tags`, `front_matter_json`, `content_hash`, `status`, timestamps.
- Status lifecycle: `pending` → `processed` (after seed extraction).

**`content_chunks` table** (via `crates/tuitbot-core/src/storage/watchtower/mod.rs:189`):
- `ContentChunk` struct: `id`, `account_id`, `node_id`, `heading_path`, `chunk_text`, `chunk_hash`, `chunk_index`, `retrieval_boost`, `status`, timestamps.
- Chunks are addressed by heading hierarchy (e.g., `"# Title > ## Section"`).
- No line-range, byte-offset, or paragraph-level addressing exists.
- `chunk_index` is sequential within a node but represents heading order, not paragraph position.

**`draft_seeds` table** (via `crates/tuitbot-core/src/storage/watchtower/mod.rs:157`):
- `DraftSeed` struct: `id`, `account_id`, `node_id`, `seed_text`, `archetype_suggestion`, `engagement_weight`, `status`, `created_at`, `used_at`, `chunk_id`.
- Seeds are LLM-extracted hooks (max 200 chars each, 1-3 per note).
- `chunk_id` is optional — seeds are linked to nodes, not always to specific chunks.
- `engagement_weight` defaults to `COLD_START_WEIGHT` and can be updated based on performance.

### Seed Worker (`crates/tuitbot-core/src/automation/seed_worker.rs`)

- Background worker processes nodes with `status='pending'` in batches of 5.
- Extracts 1-3 tweetable hooks per note via LLM with format suggestions (list, tip, question, contrarian_take, storytelling, before_after).
- Parses `HOOK:` / `FORMAT:` response format.
- Runs on 5-minute interval; low-priority (yields between nodes).
- Seeds are stored with `COLD_START_WEIGHT` and marked `processed` on success.

**Gap**: Seeds exist in the database but are only consumed by the autopilot context builder (`winning_dna`). They are never exposed to the user in the composer UI.

## 2. Retrieval & RAG

### Fragment Retrieval (`crates/tuitbot-core/src/context/retrieval.rs`)

- `retrieve_vault_fragments()`: Two-step retrieval — selected-note chunks first, then keyword-matched chunks.
- `FragmentContext`: Pairs `chunk_text` (full text) with `VaultCitation` metadata.
- `VaultCitation` struct: `chunk_id`, `node_id`, `heading_path`, `source_path`, `source_title`, `snippet` (120 chars), `retrieval_boost`.
- `format_fragments_prompt()`: Builds a numbered prompt block capped at `MAX_FRAGMENT_CHARS` (1000 chars).
- `build_citations()`: Extracts citation metadata from fragments.
- `citations_to_provenance_refs()`: Converts citations to `ProvenanceRef` for persistence.

**Constants**: `MAX_FRAGMENT_CHARS = 1000`, `MAX_FRAGMENTS = 5`, `CITATION_SNIPPET_LEN = 120`.

### Winning DNA Context (`crates/tuitbot-core/src/context/winning_dna/mod.rs`)

- `DraftContext` struct: `winning_ancestors`, `content_seeds`, `vault_citations`, `prompt_block`.
- Three-tier context: winning historical content + vault fragments + cold-start seeds.
- `build_draft_context_with_selection()`: Accepts optional `selected_node_ids` for biased retrieval.

### RAG Helpers (`crates/tuitbot-server/src/routes/rag_helpers.rs`)

- `resolve_composer_rag_context()`: Server-side resolver that loads config keywords, builds `DraftContext`.
- Fail-open design: returns `None` on any error, missing config, or empty keywords.
- Used by all assist endpoints (tweet, thread, improve, reply, highlights).

**Gap**: No mechanism to expose seed hooks to the user before generation. The RAG pipeline treats seeds as background context, not as selectable entry points.

## 3. Composer Flow (Frontend)

### FromVaultPanel (`dashboard/src/lib/components/composer/FromVaultPanel.svelte`)

- Entry point for vault-backed generation.
- Flow: Search notes → Expand note → Select chunks (max 3 via `MAX_SELECTIONS`) → Extract highlights (LLM round-trip) → Generate.
- State: `searchQuery`, `notes`, `expandedNote`, `selectedChunks` (Map of chunk_id → {nodeId, heading}), `highlightsStep`.
- Calls `api.vault.searchNotes()`, `api.vault.noteDetail()`, `api.assist.highlights()`.
- Output format toggle: `tweet` or `thread`.

### VaultNoteList (`dashboard/src/lib/components/composer/VaultNoteList.svelte`)

- Displays notes with expand/collapse, chunk checkboxes.
- Shows `heading_path` and truncated `snippet` per chunk.
- Selection is chunk-level (heading sections), not paragraph or block-level.

### VaultHighlights (`dashboard/src/lib/components/composer/VaultHighlights.svelte`)

- Intermediate step: shows LLM-extracted highlights with enable/disable toggles.
- Highlights are computed per-request, not cached.

**Gap**: No "Hooks" tab or hook-first entry point. Users must go through the full search → expand → select → extract flow even when pre-computed hooks exist.

### ComposerInspector (`dashboard/src/lib/components/composer/ComposerInspector.svelte`)

- Orchestrates AI assist, vault generation, voice cue, notes panel, and scheduling.
- `handleGenerateFromVault()`: Accepts `selectedNodeIds`, `outputFormat`, optional `highlights`. Calls `api.assist.thread()` or `api.assist.improve()` depending on format and highlights presence.
- `handleAiAssist()`: Standalone generation without vault context.
- Exposes `notesPanelMode: 'notes' | 'vault' | null` for panel switching.

**Gap**: No "hooks" panel mode. The inspector only knows about notes and vault panels.

## 4. Citation Display

### CitationChips (`dashboard/src/lib/components/composer/CitationChips.svelte`)

- Displays vault citations as expandable chips with "Based on:" label.
- Each chip shows: title + heading (or just title), expandable detail with `heading_path` and `snippet`.
- Desktop-only: "Open in Obsidian" button via `buildObsidianUri()` + `openExternalUrl()`.
- Remove button to detach citations.

**Gap**: Deep-links open the note file but do not navigate to the specific heading section. Obsidian supports `#heading` anchors in URIs but `buildObsidianUri()` does not use them.

## 5. Obsidian Deep-Link & Desktop Integration

### obsidianUri.ts (`dashboard/src/lib/utils/obsidianUri.ts`)

- `buildObsidianUri(vaultPath, relativePath)`: Constructs `obsidian://open?vault=<name>&file=<path>` URIs.
- Vault name derived from last path component of `vaultPath`.
- Strips `.md` extension from file path.
- Does **not** include heading anchors (`#heading`).

### Tauri Integration (`dashboard/src-tauri/src/lib.rs`)

- `open_external_url` command: Restricted to `obsidian://` and `file://` schemes.
- Cross-platform: uses `open` (macOS), `cmd /C start` (Windows), `xdg-open` (Linux).
- `DeploymentMode::Desktop` set explicitly in Tauri app state.
- Clipboard plugin available (`tauri_plugin_clipboard_manager`).

**Gap**: No mechanism to receive data from Obsidian. The current flow is one-way (TuitBot → Obsidian). There is no `POST /api/vault/send-selection` or URI-based push from Obsidian to TuitBot.

## 6. Provenance System

### Storage (`crates/tuitbot-core/src/storage/provenance.rs`)

- `vault_provenance_links` table: polymorphic via `entity_type` + `entity_id`.
- `ProvenanceLink` struct: `id`, `account_id`, `entity_type`, `entity_id`, `node_id`, `chunk_id`, `seed_id`, `source_path`, `heading_path`, `snippet`, `created_at`.
- `ProvenanceRef` struct: carried through API layer, all fields optional with `#[serde(default)]`.
- CRUD operations: `insert_links_for`, `get_links_for`, `copy_links_for`, `delete_links_for`.
- Provenance survives source note deletion (snapshot fields).

### API Integration

- `ComposeRequest` in `crates/tuitbot-server/src/routes/content/compose/mod.rs` accepts `provenance: Option<Vec<ProvenanceRef>>`.
- Frontend `ComposeRequest` type in `dashboard/src/lib/api/types.ts` includes `provenance?: ProvenanceRef[]`.
- `api.drafts.create()` accepts `provenance` parameter.
- Citations are converted to provenance refs via `citations_to_provenance_refs()`.

**Gap**: Provenance currently links to `node_id` and `chunk_id` but not to `seed_id` in practice (the field exists but is never populated from the composer flow). Hook-first drafting would populate `seed_id`.

## 7. Compose & Scheduling

### Compose Endpoint (`crates/tuitbot-server/src/routes/content/compose/mod.rs`)

- Unified `POST /api/content/compose`: Accepts `content_type`, `content`, `scheduled_for`, `media_paths`, `blocks`, `provenance`.
- Routes to `compose_tweet_flow` or `compose_thread_blocks_flow`/`compose_thread_legacy_flow`.
- Approval mode check: queues to approval queue if enabled, otherwise accepts directly.
- Thread blocks: structured `ThreadBlockRequest` with `id`, `text`, `media_paths`, `order`.

### Draft Studio

- Full CRUD via `api.draftStudio.*`: list, get, create, autosave, schedule, revisions, activity log, tags.
- `DraftSummary`: `id`, `title`, `content_type`, `content_preview`, `status`, `scheduled_for`, `archived_at`, timestamps, `source`.
- Revisions: manual and auto-triggered snapshots.
- Activity log: tracks create, edit, schedule, publish events.

### Assist Endpoints (`crates/tuitbot-server/src/routes/assist.rs`)

- `POST /api/assist/tweet`: Topic + optional `selected_node_ids` → generated tweet + citations.
- `POST /api/assist/thread`: Topic + optional `selected_node_ids` → generated thread tweets + citations.
- `POST /api/assist/improve`: Draft + context + optional `selected_node_ids` → improved content + citations.
- `POST /api/assist/highlights`: `selected_node_ids` → extracted highlights + citations (LLM round-trip).
- `GET /api/assist/topics`: Top-performing topics from analytics.

**Gap**: No endpoint to retrieve draft seeds/hooks for a specific note. Seeds exist in the DB but are only accessible through the `winning_dna` context builder.

## 8. Vault API Endpoints (`crates/tuitbot-server/src/routes/vault.rs`)

- `GET /api/vault/sources`: Returns configured sources with status, node count, and local path (for deep-links).
- `GET /api/vault/notes?q=&source_id=&limit=`: Search/list notes. Returns `node_id`, `title`, `relative_path`, `tags`, `chunk_count`.
- `GET /api/vault/notes/{id}`: Note detail with chunks. Chunks include `chunk_id`, `heading_path`, truncated `snippet` (120 chars), `retrieval_boost`.
- `GET /api/vault/search?q=&limit=`: Fragment search returning `VaultCitation` records.
- `POST /api/vault/resolve-refs`: Resolve node IDs to citation records.

**Privacy enforcement**: `SNIPPET_MAX_LEN = 120`. No raw `chunk_text` or `body_text` in any response. All endpoints are account-scoped via `AccountContext`.

## 9. Deployment Modes

### Types (`dashboard/src/lib/api/types.ts:49`)

```typescript
type DeploymentModeValue = 'desktop' | 'self_host' | 'cloud';
```

### Capabilities (`dashboard/src/lib/api/types.ts:51-58`)

```typescript
interface DeploymentCapabilities {
  local_folder: boolean;
  manual_local_path: boolean;
  google_drive: boolean;
  inline_ingest: boolean;
  file_picker_native: boolean;
  preferred_source_default: string;
}
```

### Runtime Detection

- Desktop: `DeploymentMode::Desktop` set in `dashboard/src-tauri/src/lib.rs:209`.
- Self-host/Cloud: determined by server configuration.
- Frontend reads `deployment_mode` from `RuntimeStatus` response.
- `isDesktop` prop used by `CitationChips.svelte` to conditionally show Obsidian deep-links.

## Summary of Gaps

| # | Gap | Impact | Relevant Files |
|---|-----|--------|----------------|
| 1 | No block-level send (paragraph granularity) | Users select whole heading sections, not specific paragraphs | `VaultNoteList.svelte`, `ContentChunk` struct |
| 2 | No hook-first thread drafting | Draft seeds exist but are hidden from the composer | `seed_worker.rs`, `FromVaultPanel.svelte` |
| 3 | No heading-anchor deep-links | Obsidian opens the file but not the specific section | `obsidianUri.ts` |
| 4 | Highlight extraction is not cached | Every highlight request is an LLM round-trip | `assist.rs` highlights endpoint |
| 5 | No Obsidian → TuitBot selection handoff | One-way deep-links only (TuitBot → Obsidian) | `lib.rs` (Tauri), no ingress endpoint |
| 6 | No hook/seed retrieval endpoint | Seeds are DB-only, not exposed via API | `vault.rs`, no seeds endpoint |
| 7 | Provenance `seed_id` never populated from composer | Hook-originated content has no seed provenance | `provenance.rs`, `compose/mod.rs` |
