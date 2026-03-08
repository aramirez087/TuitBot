# Fragment Indexing — Extraction Rules & Identity

## Overview

Ingested content nodes are split into heading-delimited **fragments** (stored
as `content_chunks`) so that retrieval can cite specific sections rather than
entire notes. This document specifies the extraction algorithm, identity
rules, fallback behavior, and update semantics.

## Fragment Extraction Rules

### Heading-based splitting

The chunker scans the note body line by line and splits on ATX headings
(`# ` through `###### `). Each heading starts a new fragment.

- Regex: `^(#{1,6})\s+(.+)$`
- Headings inside fenced code blocks (` ``` `) are ignored.
- Text before the first heading becomes a **root fragment** with
  `heading_path = ""`.
- Empty or whitespace-only fragments are discarded.

### Heading path construction

A stack of `(level, label)` tuples tracks the heading hierarchy. When a
heading is encountered:

1. Pop all stack entries with `level >= current_level`.
2. Push `(current_level, "## Heading Text")`.
3. The `heading_path` is the stack joined with `/`.

**Examples:**

| Heading sequence | heading_path |
|------------------|-------------|
| `## Intro` | `## Intro` |
| `## Intro` then `### Details` | `## Intro/### Details` |
| `## Intro` then `### Details` then `## Next` | `## Next` (stack resets) |
| `# Title` then `## A` then `### B` | `# Title/## A/### B` |

### Code-block awareness

Lines starting with ` ``` ` toggle a code-block flag. While inside a code
block, no heading detection occurs. This prevents false positives from
markdown inside code examples.

## Fragment Identity Rules

A fragment is identified by:

- **`node_id`** — the parent content node
- **`chunk_hash`** — SHA-256 of the fragment's `chunk_text`

On re-chunking, if a fragment with the same `chunk_hash` already exists for
the node (active or stale), the existing row is **reactivated** with updated
`heading_path` and `chunk_index`. This preserves the chunk ID and any
accumulated `retrieval_boost`.

If no matching hash is found, a new chunk row is inserted.

## Update Semantics (Re-ingest)

When a note is modified and re-ingested:

1. `ingest_content` detects the content hash change → `UpsertResult::Updated`.
2. Node status resets to `pending`.
3. `chunk_pending_nodes` picks up the node.
4. `mark_chunks_stale` sets all existing chunks for the node to `status = 'stale'`.
5. `upsert_chunks_for_node` processes each new fragment:
   - **Hash match** → reactivate existing chunk (preserves ID and retrieval_boost).
   - **No match** → insert new chunk.
6. Chunks that remain `stale` after upsert are old fragments that no longer
   exist in the updated note. They are preserved for FK integrity but excluded
   from retrieval queries (which filter `status = 'active'`).
7. Node status transitions to `chunked`.

## Fallback Behavior

### Plain-text files (`.txt`)

Plain-text files have no headings. The entire body becomes a single root
fragment with `heading_path = ""`. This is identical to a markdown file with
no headings.

### Empty body

If the body is empty or whitespace-only after front-matter stripping, no
fragments are created. The node is still marked `chunked` (with zero chunks).

### Front-matter handling

Front-matter is stripped by the ingest pipeline before the body reaches the
chunker. Title and tags are stored on the parent `ContentNode`, not
duplicated into chunk text. Retrieval queries join with `content_nodes` for
context.

## Chunk Lifecycle

```
                 ┌──────────┐
  ingest ──────→ │  pending  │
                 └────┬─────┘
                      │ chunk_pending_nodes()
                      ▼
                 ┌──────────┐
                 │  chunked  │
                 └────┬─────┘
                      │ seed worker (future)
                      ▼
                 ┌──────────┐
                 │ processed │
                 └──────────┘
```

Chunk status within a node:

```
  active ──→ stale  (on re-chunk)
  stale  ──→ active (if hash matches during re-chunk)
```

## Integration Points

- **Watchtower** calls `chunk_pending_nodes()` after every scan, poll, or
  event batch.
- **Retrieval** (future) queries `content_chunks WHERE status = 'active'`
  with keyword matching and `retrieval_boost` ordering.
- **Loop-back** (future) can update `retrieval_boost` based on engagement
  analytics.

## Known Limitations

- The heading parser uses regex, not a full CommonMark parser. Headings
  inside HTML blocks or indented code blocks may be misdetected. Fenced
  code blocks are handled correctly.
- No minimum fragment size threshold. Very short sections (e.g., a heading
  with one word of body) produce small chunks. A future session can add
  merging heuristics if needed.
- Fragment ordering (`chunk_index`) is sequential within a single note.
  Cross-note ordering is by `node_id` + `chunk_index`.
