# Loop-Back Contract

Loop-back is the mechanism that writes publishing metadata back into the source note's YAML front-matter after content is posted to X. This closes the note-to-post loop, giving vault users traceability from idea to published tweet.

## Front-Matter Format

Loop-back entries are stored under a `tuitbot` key as a YAML array:

```yaml
---
title: My Launch Plan
tags: [launch, strategy]
tuitbot:
  - tweet_id: "1234567890"
    url: "https://x.com/i/status/1234567890"
    published_at: "2026-03-08T14:30:00Z"
    type: tweet
    status: posted
  - tweet_id: "9876543210"
    url: "https://x.com/i/status/9876543210"
    published_at: "2026-03-09T10:00:00Z"
    type: thread
    status: posted
    thread_url: "https://x.com/user/status/9876543210"
---
Content of the note...
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tweet_id` | string | yes | The X platform tweet ID |
| `url` | string | yes | Full URL to the posted tweet |
| `published_at` | string | yes | ISO 8601 timestamp of when it was posted |
| `type` | string | yes | Content type: `tweet`, `thread`, `reply`, `thread_tweet` |
| `status` | string | no | Post status: `posted`, `deleted`. Added in Session 7. |
| `thread_url` | string | no | Thread URL when entry is part of a thread. Added in Session 7. |

New fields use `#[serde(default, skip_serializing_if = "Option::is_none")]` so they are backward-compatible with entries written by earlier versions.

## Source-Type Support Matrix

| Source Type | Read | Write (Loop-Back) | Notes |
|-------------|------|--------------------|-------|
| `local_fs` | yes | yes | Full read-write. Base path resolved from `config_json.path`. |
| `google_drive` | yes | no | Read-only. `execute_loopback()` returns `SourceNotWritable("google_drive")`. |
| `manual` | yes | no | Read-only. No backing file. Returns `SourceNotWritable("manual")`. |

## Idempotency

Loop-back writes are idempotent, keyed on `tweet_id`:

1. Before writing, `write_metadata_to_file()` parses existing front-matter
2. If any entry already has the same `tweet_id`, the write is skipped (returns `false`)
3. Multiple calls with the same `tweet_id` produce exactly one entry

## API: `execute_loopback()`

```rust
pub async fn execute_loopback(
    pool: &DbPool,
    node_id: i64,
    tweet_id: &str,
    url: &str,
    content_type: &str,
) -> LoopBackResult
```

### Resolution Flow

1. Look up `ContentNode` by `node_id` → get `source_id`, `relative_path`
2. Look up `SourceContext` by `source_id` → get `source_type`, `config_json`
3. If `source_type != "local_fs"` → return `SourceNotWritable`
4. Parse `config_json` to extract `path` field, expand tilde, join with `relative_path`
5. Build `LoopBackEntry` and call `write_metadata_to_file()`

### `LoopBackResult` Variants

| Variant | Meaning |
|---------|---------|
| `Written` | Metadata was written to the source file |
| `AlreadyPresent` | The `tweet_id` was already in the file — no modification |
| `SourceNotWritable(reason)` | Source type doesn't support writes (google_drive, manual, etc.) |
| `NodeNotFound` | The content node was not found in the database |
| `FileNotFound` | The resolved file path does not exist on disk |

## Re-Ingest Behavior

After a loop-back write, the file's content hash changes (front-matter was modified). When the watchtower detects this:

1. The watchtower's 5-second `CooldownSet` suppresses the immediate filesystem event from the write
2. On the next poll/scan cycle, the file is re-ingested as `Updated` (hash differs)
3. The content node's status resets to `pending`
4. Chunks are re-extracted (they may or may not differ since front-matter is stripped before chunking)

This is harmless and expected — the note's body text hasn't changed, so chunks will be deduplicated.

## Integration with Approval Poster

After a successful post, the approval poster:

1. Calls `propagate_provenance()` to copy provenance links from `approval_queue` to `original_tweet`
2. Calls `execute_loopback_for_provenance()` which:
   - Queries provenance links for the approval queue item
   - Deduplicates by `node_id` (multiple chunks from the same note)
   - Calls `execute_loopback()` for each unique node
   - Logs results at appropriate levels (info for writes, debug for skips/no-ops)
