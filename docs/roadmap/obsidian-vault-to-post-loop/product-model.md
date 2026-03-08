# Obsidian Vault to Post Loop — Product Model

## Entities

### 1. Source

A registered content source (row in `source_contexts`).

| Field | Type | Description |
|-------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `account_id` | TEXT | Owner account; enforces isolation |
| `source_type` | TEXT | `local_fs` (Obsidian vault), `google_drive`, `manual` |
| `config_json` | TEXT | Source-specific config (path, folder_id, etc.) |
| `sync_cursor` | TEXT | Last sync position (timestamp or page token) |
| `status` | TEXT | `active`, `syncing`, `error`, `disabled` |
| `error_message` | TEXT | Last error description |

Existing table: `source_contexts` (migration `20260228000019`).

### 2. Note

An ingested file (row in `content_nodes`).

| Field | Type | Description |
|-------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `account_id` | TEXT | Owner account |
| `source_id` | INTEGER FK | References `source_contexts.id` |
| `relative_path` | TEXT | File path relative to source root |
| `content_hash` | TEXT | SHA-256 of file content; dedup key |
| `title` | TEXT | Extracted from YAML front-matter |
| `body_text` | TEXT | Full note body (after front-matter) |
| `front_matter_json` | TEXT | Raw YAML as JSON |
| `tags` | TEXT | Comma-separated tags from front-matter |
| `status` | TEXT | `pending` → `chunked` → `processed` |

Existing table: `content_nodes` (migration `20260228000019`).

Status lifecycle change: currently `pending` → `processed` (after seed
extraction). This epic adds the `chunked` intermediate state, set after
fragments are created but before seed extraction runs.

### 3. Fragment (NEW)

A heading-delimited section of a Note (row in `content_chunks`).

| Field | Type | Description |
|-------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `account_id` | TEXT | Owner account |
| `node_id` | INTEGER FK | References `content_nodes.id` |
| `heading_path` | TEXT | Slash-delimited heading hierarchy (e.g. `## Intro/### Background`) |
| `chunk_text` | TEXT | Section body text |
| `chunk_hash` | TEXT | SHA-256 of chunk_text; dedup on re-chunk |
| `chunk_index` | INTEGER | Ordering within the note (0-based) |
| `retrieval_boost` | REAL | Multiplicative relevance factor (default 1.0, range 0.1–5.0) |
| `status` | TEXT | `active`, `stale` (after note re-chunk) |

New table: `content_chunks` (Session 02 migration).

**Chunking algorithm:**
1. Split note body on `^#{1,3} ` (H1–H3 headings).
2. Each heading starts a new chunk; text before the first heading is chunk 0
   with `heading_path = ""`.
3. `heading_path` tracks nested hierarchy: if an H3 follows an H2, the path
   is `## Parent/### Child`.
4. Chunks smaller than 50 characters are merged with the previous chunk.
5. Chunks larger than 2000 characters are split further at double-newline
   paragraph boundaries.
6. Notes without any headings fall back to paragraph splitting with 500-char
   max per chunk.

### 4. Seed

An LLM-extracted tweetable hook from a Note or Fragment (row in `draft_seeds`).

| Field | Type | Description |
|-------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `account_id` | TEXT | Owner account |
| `node_id` | INTEGER FK | References `content_nodes.id` |
| `chunk_id` | INTEGER FK | References `content_chunks.id` (NEW, nullable) |
| `seed_text` | TEXT | The extracted hook |
| `archetype_suggestion` | TEXT | Suggested tweet format |
| `engagement_weight` | REAL | Cold-start default 0.5; updated by loop-back |
| `status` | TEXT | `pending`, `used`, `expired` |
| `used_at` | TEXT | Set when seed contributes to a queued draft |

Existing table: `draft_seeds` (migration `20260228000019`).
New column: `chunk_id` (Session 03 migration).

After this epic, seed extraction runs per-fragment instead of per-note. Each
seed links to the specific chunk it was extracted from, enabling fragment-level
provenance.

### 5. Selected Reference

A fragment chosen by the RAG retriever for a specific generation call.
Not persisted as a separate table; captured as a JSON array in the
approval queue record.

Format:
```json
[
  {
    "chunk_id": 42,
    "note_title": "Marketing Playbook",
    "heading_path": "## Key Insight",
    "relevance_score": 0.85
  }
]
```

Stored in `approval_queue.source_chunks_json`.

### 6. Citation

User-facing attribution in assist responses. Returned alongside generated
content in API responses.

Format:
```json
{
  "note_title": "Marketing Playbook",
  "heading": "## Key Insight",
  "chunk_preview": "First 120 characters of the chunk text...",
  "obsidian_uri": "obsidian://open?vault=notes&file=Marketing%20Playbook"
}
```

- `obsidian_uri` is populated only when `deployment_mode === Desktop` and
  source type is `local_fs`.
- All fields are optional (backward compatible).
- Returned as an array in `citations` field of assist responses.

### 7. Draft Provenance

Link from a draft to its source material. Tracked via new nullable columns
on `approval_queue`:

| Column | Type | Description |
|--------|------|-------------|
| `source_node_id` | INTEGER FK | Primary note that influenced the draft |
| `source_seed_id` | INTEGER FK | Specific seed used (if applicable) |
| `source_chunks_json` | TEXT | JSON array of selected references |

Existing table: `approval_queue` (needs migration in Session 03).

### 8. Post Provenance

Link from a posted tweet to its source material. Chain:

```
Note (content_nodes)
  → Fragment (content_chunks)
    → Seed (draft_seeds, optional)
      → Draft (approval_queue, with provenance FKs)
        → Post (original_tweets.source_node_id already exists)
```

`original_tweets.source_node_id` was added in migration `20260228000019`.
The approval queue's `posted_tweet_id` column links draft to post.

### 9. Loop-Back State

Performance data flowing back to the vault to improve future retrieval.

**Trigger:** When the analytics loop records a post's performance score.

**Process:**
1. Look up the post's approval queue record.
2. Read `source_chunks_json` to find referenced chunk IDs.
3. For each chunk: `new_boost = current_boost * (1 + 0.1 * normalized_engagement)`.
4. Clamp `retrieval_boost` to range [0.1, 5.0].
5. If `source_seed_id` is set, update `draft_seeds.engagement_weight`.
6. `loopback.rs` writes performance score to source file YAML front-matter.

**Normalized engagement:** The post's `performance_score` divided by the
account's max `performance_score` (same normalization as `winning_dna.rs`).

## Entity Lifecycle

```
Note File (Obsidian vault)
  │
  ▼ [Watchtower ingest — file watch or poll]
content_nodes (status: pending)
  │
  ▼ [Chunker — heading-based split]
content_chunks (status: active)
content_nodes (status: chunked)
  │
  ▼ [Seed Worker — LLM extract per chunk]
draft_seeds (status: pending, chunk_id set)
content_nodes (status: processed)
  │
  ▼ [RAG Retriever — keyword match + retrieval_boost ranking]
Selected references (chunks ranked by relevance × boost)
  │
  ▼ [Generator — LLM with RAG context]
Draft text + citation metadata
  │
  ▼ [Approval Queue — provenance FKs populated]
approval_queue (source_node_id, source_seed_id, source_chunks_json)
  │
  ▼ [Poster — approved or auto-approved]
Posted tweet (original_tweets.source_node_id set)
  │
  ▼ [Analytics Loop — engagement measured]
Performance score recorded
  │
  ▼ [Loop-Back — update retrieval weights]
content_chunks.retrieval_boost updated
draft_seeds.engagement_weight updated
Source file YAML front-matter updated with performance score
  │
  ▼ (cycle repeats — updated boost improves next retrieval)
```

## Re-Chunking on Note Update

When a note is updated (content hash changes):

1. Watchtower detects the change via file watch or poll.
2. `upsert_content_node` updates the node, resets status to `pending`.
3. Chunker runs: marks existing chunks as `stale`, creates new chunks.
4. Seed worker generates new seeds for new/changed chunks.
5. Old seeds remain (their `engagement_weight` preserves historical signal).
6. Stale chunks with non-default `retrieval_boost` (> 1.0) transfer their
   boost to the closest-matching new chunk by heading path.

## Account Isolation Model

Every entity carries an `account_id`. Every storage query filters by
`account_id`. The `DEFAULT_ACCOUNT_ID` constant
(`00000000-0000-0000-0000-000000000000`) serves as fallback for single-account
installs but is never hardcoded in queries — it is passed as a parameter.

Isolation guarantees:
- Source listing: filtered by account_id.
- Note listing: filtered by account_id.
- Chunk retrieval: filtered by account_id.
- Seed retrieval: filtered by account_id.
- RAG context building: all underlying queries filter by account_id.
- Vault health stats: scoped to account_id.
