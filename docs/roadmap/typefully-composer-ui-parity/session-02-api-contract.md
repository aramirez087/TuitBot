# Session 02 — API Contract: Thread Blocks Schema

**Date:** 2026-02-27
**Status:** Implemented and tested

---

## ThreadBlock Schema

A `ThreadBlock` represents a single tweet within a structured thread composition.

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | Yes | Client-generated stable UUID. Must be non-empty, unique within the request. Survives reorder/edit roundtrips. |
| `text` | `string` | Yes | Tweet text content. Must be non-empty after trim. Max 280 characters (URL-weighted). |
| `media_paths` | `string[]` | No (default `[]`) | Local file paths for media attached to this block. Max 4 entries. |
| `order` | `u32` | Yes | Zero-based ordering index. All blocks must form a contiguous sequence `0..N`. |

### Example Request

```json
{
  "content_type": "thread",
  "content": "ignored when blocks present",
  "scheduled_for": "2026-03-01T10:00:00Z",
  "blocks": [
    {"id": "550e8400-e29b-41d4-a716-446655440000", "text": "1/ Introducing our new feature...", "media_paths": ["screenshot.png"], "order": 0},
    {"id": "550e8400-e29b-41d4-a716-446655440001", "text": "2/ Here's how it works...", "media_paths": [], "order": 1},
    {"id": "550e8400-e29b-41d4-a716-446655440002", "text": "3/ Try it now at https://example.com", "media_paths": [], "order": 2}
  ]
}
```

### Example Response (blocks)

```json
{
  "status": "scheduled",
  "id": 42,
  "block_ids": ["550e8400-e29b-41d4-a716-446655440000", "550e8400-e29b-41d4-a716-446655440001", "550e8400-e29b-41d4-a716-446655440002"]
}
```

---

## Storage Format

Thread blocks are serialized to a versioned JSON wrapper for database storage in the existing `content` TEXT column:

```json
{
  "version": 1,
  "blocks": [
    {"id": "uuid-1", "text": "First tweet", "media_paths": ["photo.jpg"], "order": 0},
    {"id": "uuid-2", "text": "Second tweet", "media_paths": [], "order": 1}
  ]
}
```

### Format Detection

When reading content from the database, the system distinguishes three formats:

1. **Blocks payload** (new): JSON object with `version` and `blocks` keys
2. **Legacy string array**: JSON array of strings (`["tweet1","tweet2"]`)
3. **Plain text**: Raw string (single tweet content)

Detection is unambiguous: objects with a `blocks` key are new format; arrays of strings are legacy; everything else is plain text.

---

## Endpoint Changes

### `POST /api/content/compose`

**New optional field:** `blocks: ThreadBlock[]`

| Scenario | Behavior |
|----------|----------|
| `content_type: "tweet"` | `blocks` field is ignored. Existing tweet validation applies. |
| `content_type: "thread"`, `blocks` present | Blocks are validated and serialized. `content` field is ignored. Response includes `block_ids`. |
| `content_type: "thread"`, `blocks` absent | Legacy behavior: `content` parsed as JSON string array. |

**Precedence rule:** When both `blocks` and `content` are provided for a thread, `blocks` takes precedence.

### `POST /api/content/drafts`

**New optional field:** `blocks: ThreadBlock[]`

Same precedence rules as compose. When `content_type: "thread"` and `blocks` is present, blocks are validated and serialized for storage.

### `PATCH /api/content/drafts/{id}`

**Changed fields:**
- `content` is now **optional** (`Option<String>`)
- New optional field: `blocks: ThreadBlock[]`

**Validation:**
- Must provide at least one of `content` or `blocks`
- `blocks` takes precedence if both are provided
- Empty request `{}` returns `400 Bad Request`

### `GET /api/content/drafts`

No handler changes. The `content` field in the response contains the raw stored value. Clients use `parseThreadContent()` or `isBlocksPayload()` to detect the format.

---

## Validation Rules

All validation is performed by `validate_thread_blocks()` in `tuitbot-core::content::thread`.

| Rule | Error Message |
|------|--------------|
| Blocks array must not be empty | `thread blocks must not be empty` |
| Thread must have >= 2 blocks | `thread must contain at least 2 blocks` |
| Block IDs must be non-empty | `block at index {i} has an empty ID` |
| Block IDs must be unique | `duplicate block ID: {id}` |
| Order must be contiguous 0..N | `block order must be a contiguous sequence starting at 0` |
| Block text must be non-empty (after trim) | `block {id} has empty text` |
| Block text must be <= 280 chars (URL-weighted) | `block {id}: text exceeds 280 characters (length: {n})` |
| Max 4 media attachments per block | `block {id}: too many media attachments ({n}, max 4)` |

---

## Backwards Compatibility

| Existing Flow | Status |
|--------------|--------|
| `POST /api/content/compose` with `{content_type: "tweet", content: "..."}` | Unchanged |
| `POST /api/content/compose` with `{content_type: "thread", content: "[...]"}` | Unchanged |
| `POST /api/content/tweets` with `{text: "..."}` | Unchanged |
| `POST /api/content/threads` with `{tweets: [...]}` | Unchanged |
| `POST /api/content/drafts` with `{content_type: "tweet", content: "..."}` | Unchanged |
| `PATCH /api/content/drafts/{id}` with `{content: "..."}` | Unchanged (content is now `Option<String>` but `Some("...")` deserialization is identical) |

---

## Frontend Integration Types

### TypeScript Types (in `dashboard/src/lib/api.ts`)

```typescript
interface ThreadBlock {
    id: string;
    text: string;
    media_paths: string[];
    order: number;
}

interface ThreadBlocksPayload {
    version: number;
    blocks: ThreadBlock[];
}

interface ComposeRequest {
    content_type: string;
    content: string;
    scheduled_for?: string;
    media_paths?: string[];
    blocks?: ThreadBlock[];  // NEW
}
```

### Helper Functions

- `parseThreadContent(content: string): ThreadBlock[] | string[]` — Detects storage format and returns typed blocks or legacy string array
- `isBlocksPayload(content: string): boolean` — Quick check whether stored content uses blocks format

### Updated API Methods

- `api.drafts.create(contentType, content, source?, blocks?)` — Now accepts optional blocks
- `api.drafts.edit(id, content?, blocks?)` — Now accepts optional blocks
