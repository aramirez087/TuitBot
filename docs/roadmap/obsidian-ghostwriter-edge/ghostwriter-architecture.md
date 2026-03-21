# Ghostwriter Architecture

## Design Principles

1. **Extend, don't replace**: Every change builds on existing vault, compose, and provenance primitives.
2. **Hook-first**: Pre-extracted hooks are the primary entry point for content creation, not an afterthought.
3. **Privacy by deployment**: Raw content access is gated by `DeploymentMode`, not by feature flag.
4. **Additive schema**: New columns and tables use defaults; existing data and APIs remain backward-compatible.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                       Obsidian Vault                            │
│  ┌──────┐  ┌──────┐  ┌──────┐                                  │
│  │Note A│  │Note B│  │Note C│  (markdown files)                 │
│  └──┬───┘  └──┬───┘  └──┬───┘                                  │
└─────┼────────┼────────┼────────────────────────────────────────┘
      │        │        │
      ▼        ▼        ▼
┌─────────────────────────────────────────────────────────────────┐
│  Watchtower Ingestion (existing)                                │
│  file watch → content_nodes → content_chunks (heading-level)    │
│                    │                                            │
│                    ▼                                            │
│  Seed Worker → draft_seeds (1-3 hooks per note)                 │
└─────────────────────────────────────────────────────────────────┘
      │                    │
      ▼                    ▼
┌──────────────────┐ ┌──────────────────────────────────────────┐
│ Vault API        │ │ NEW: Hooks API                           │
│ /vault/notes     │ │ GET /vault/notes/{id}/hooks              │
│ /vault/notes/{id}│ │ Returns pre-extracted draft seeds        │
│ /vault/search    │ │ with archetype and engagement weight     │
└──────────────────┘ └──────────────────────────────────────────┘
      │                    │
      ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│  Dashboard Composer                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │ Vault Panel  │  │ NEW: Hooks   │  │ Voice Cue / Notes     │ │
│  │ (chunks)     │  │ Panel (seeds)│  │ (existing)            │ │
│  └──────┬──────┘  └──────┬───────┘  └────────────────────────┘ │
│         │                │                                      │
│         ▼                ▼                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Generation Engine (existing assist endpoints)            │   │
│  │ assist/tweet, assist/thread, assist/improve              │   │
│  │ + NEW: hook_id parameter for seed-originated generation  │   │
│  └──────────────────────────────┬───────────────────────────┘   │
│                                 │                               │
│                                 ▼                               │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Draft Studio / Approval Queue (existing)                 │   │
│  │ + provenance with seed_id populated for hook-originated  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                 │                               │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Citation Chips (enhanced)                                │   │
│  │ + heading-anchor deep-links: obsidian://...#heading      │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Component Design

### 1. Hooks API Endpoint

**New endpoint**: `GET /api/vault/notes/{id}/hooks`

**Location**: `crates/tuitbot-server/src/routes/vault.rs`

**Response**:
```json
{
  "hooks": [
    {
      "seed_id": 42,
      "node_id": 100,
      "seed_text": "Most people think async is hard. It's not—the tooling is.",
      "archetype": "contrarian_take",
      "engagement_weight": 0.5,
      "chunk_id": null,
      "status": "active"
    }
  ]
}
```

**Implementation**: Query `draft_seeds` where `node_id = {id}` and `status IN ('active', 'unused')`, account-scoped. Returns only seed metadata, not full chunk text. Privacy-safe: seed_text is a short hook (max 200 chars), not raw note content.

**Backward compatibility**: Additive endpoint. No changes to existing vault routes.

### 2. Hook-First Composer Panel

**New component**: `dashboard/src/lib/components/composer/VaultHooksPanel.svelte`

**Behavior**:
1. When user expands a note in the vault panel, hooks are loaded alongside chunks.
2. Hooks appear as selectable cards with archetype badge and seed text.
3. Selecting a hook sets it as the generation topic (replaces the current "extract highlights" step).
4. "Generate Thread" button: calls `api.assist.thread()` with the hook text as topic and the note's `node_id` as `selected_node_ids`.
5. "Generate Tweet" button: calls `api.assist.tweet()` with the hook text as topic.

**Integration with existing flow**: Hooks panel is an alternative to the chunk-selection flow, not a replacement. Users can still select chunks manually. The `notesPanelMode` in `ComposerInspector.svelte` gains a third value: `'hooks'`.

### 3. Enhanced Note Detail Response

**Modified endpoint**: `GET /api/vault/notes/{id}` (existing)

**Change**: Add optional `hooks` field to the response when seeds exist for the note.

```json
{
  "node_id": 100,
  "chunks": [...],
  "hooks": [
    {
      "seed_id": 42,
      "seed_text": "Most people think async is hard...",
      "archetype": "contrarian_take"
    }
  ]
}
```

**Backward compatibility**: `hooks` field is additive (`#[serde(skip_serializing_if = "Vec::is_empty")]`). Existing clients that don't read `hooks` are unaffected.

### 4. Heading-Anchor Deep-Links

**Modified file**: `dashboard/src/lib/utils/obsidianUri.ts`

**Change**: New function `buildObsidianUriWithHeading(vaultPath, relativePath, headingPath)` that appends `#heading` to the URI.

```typescript
export function buildObsidianUriWithHeading(
  vaultPath: string,
  relativePath: string,
  headingPath: string
): string | null {
  const base = buildObsidianUri(vaultPath, relativePath);
  if (!base) return null;
  // Extract the deepest heading from the path (e.g., "# Title > ## Setup" → "Setup")
  const heading = headingPath.split(' > ').pop()?.replace(/^#+\s*/, '');
  if (!heading) return base;
  return `${base}&heading=${encodeURIComponent(heading)}`;
}
```

**Obsidian URI spec**: `obsidian://open?vault=V&file=F&heading=H` navigates to the heading within the file.

**Integration**: `CitationChips.svelte` uses the new function when `heading_path` is available, falling back to file-level links when it is not.

### 5. Hook-Originated Provenance

**Modified behavior in**: `crates/tuitbot-server/src/routes/content/compose/mod.rs`, `crates/tuitbot-server/src/routes/assist.rs`

**Change**: When content is generated from a hook, the `ProvenanceRef` includes `seed_id`:

```json
{
  "node_id": 100,
  "chunk_id": null,
  "seed_id": 42,
  "source_path": "notes/async-rust.md",
  "heading_path": null,
  "snippet": "Most people think async is hard..."
}
```

**Implementation**: The assist endpoints accept an optional `hook_id` (seed ID). When present, the generated content's provenance includes `seed_id`. This requires a minor change to the `AssistTweetRequest` and `AssistThreadRequest` structs.

### 6. Selection Ingress API (Future Obsidian Plugin)

**New endpoint**: `POST /api/vault/send-selection`

**Request**:
```json
{
  "vault_name": "marketing",
  "file_path": "notes/async-rust.md",
  "selected_text": "The key insight is that async in Rust...",
  "heading_context": "## The Async Story",
  "selection_start_line": 45,
  "selection_end_line": 52
}
```

**Response**:
```json
{
  "status": "received",
  "session_id": "uuid",
  "composer_url": "/compose?selection=uuid"
}
```

**Behavior**: Stores the selection in a transient `vault_selections` table (TTL: 30 minutes). The dashboard polls or receives a WebSocket event to auto-open the composer with the selection pre-loaded.

**Privacy**: The selected text is stored temporarily and scoped to the authenticated account. In Cloud mode, the text is processed by the LLM and discarded — never persisted beyond the TTL.

**Plugin contract**: Any client that can POST to this endpoint can act as a selection source. An Obsidian community plugin, a browser extension, or a CLI tool can all use the same API.

### 7. Pre-Computed Highlights Cache

**Schema change**: New `cached_highlights` column on `content_nodes` table (nullable TEXT, JSON array of strings).

**Behavior**: When `seed_worker.rs` processes a node and extracts hooks, it also caches the LLM-extracted highlights. The `assist/highlights` endpoint checks the cache before making an LLM call. Cache is invalidated when the node's `content_hash` changes (re-index).

**Benefit**: Eliminates the LLM round-trip for repeat highlight extraction. First extraction still hits the LLM; subsequent views are instant.

## Data Flow: Hook-First Thread Generation

```
User opens vault panel
  → GET /api/vault/notes?q=async     (search notes)
  → User expands "Async Patterns" note
  → GET /api/vault/notes/100          (note detail + hooks)
  → Dashboard shows: 3 chunks + 2 hooks
  → User clicks hook: "Most people think async is hard..."
  → POST /api/assist/thread
      body: { topic: "Most people think async is hard...",
              selected_node_ids: [100],
              hook_id: 42 }
  → Server resolves RAG context (winning_dna + vault fragments for node 100)
  → LLM generates 4-tweet thread grounded in the note's content
  → Response includes vault_citations
  → User reviews thread in composer
  → POST /api/content/compose
      body: { content_type: "thread",
              blocks: [...],
              provenance: [{ node_id: 100, seed_id: 42, ... }] }
  → Content enters approval queue with seed provenance
  → Citation chips show "Based on: Async Patterns › ## The Async Story"
      with heading-level Obsidian deep-link
```

## File Change Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `crates/tuitbot-server/src/routes/vault.rs` | Modified | Add `GET /vault/notes/{id}/hooks` endpoint; add `hooks` field to note detail |
| `crates/tuitbot-server/src/routes/assist.rs` | Modified | Add optional `hook_id` to tweet/thread request structs |
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Modified | Add query for seeds by node_id; add `cached_highlights` support |
| `crates/tuitbot-core/src/automation/seed_worker.rs` | Modified | Cache highlights during seed extraction |
| `dashboard/src/lib/utils/obsidianUri.ts` | Modified | Add `buildObsidianUriWithHeading()` function |
| `dashboard/src/lib/components/composer/CitationChips.svelte` | Modified | Use heading-anchor deep-links |
| `dashboard/src/lib/components/composer/VaultHooksPanel.svelte` | New | Hook selection UI component |
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Modified | Integrate hooks tab alongside chunk selection |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Modified | Add hooks panel mode and generation handler |
| `dashboard/src/lib/api/client.ts` | Modified | Add `vault.noteHooks()` API method |
| `dashboard/src/lib/api/types.ts` | Modified | Add `VaultHookItem` type |
