# Obsidian Plugin Transport Contract

This document defines the exact contract between the Obsidian TuitBot Ghostwriter plugin (sender) and the TuitBot server (receiver). Session 3+ implements the receiving side using this specification.

## Transport

**Method**: HTTP POST to the local TuitBot server.

**Endpoint**: `POST /api/vault/send-selection`

**Base URL**: `http://127.0.0.1:3001` (configurable; default for Desktop mode).

### Why HTTP POST

| Alternative | Reason Rejected |
|---|---|
| Clipboard bridge | Fragile, platform-specific, no structured payload, user loses clipboard content, no acknowledgment of receipt. |
| File-system watcher | Operator rules prohibit background daemons and sync engines. A watched drop-file is a sync engine in disguise. |
| Custom IPC / Unix socket | New transport layer. TuitBot already exposes a full HTTP API on `127.0.0.1:3001`. Zero new infrastructure needed. |
| `obsidian://` URI callback | Obsidian's URI scheme is one-way (opens files). Cannot receive structured data back. |
| WebSocket push | Adds connection lifecycle management. HTTP request-response is simpler for a command-driven, one-shot action. |

**Chosen**: HTTP POST is the existing integration surface used by the Tauri frontend, MCP server, and CLI. One request, one response, no persistent connection. Aligns with Session 01 Decision 4.

## Authentication

**Header**: `Authorization: Bearer <api_token>`

**Token source**: File at `~/.tuitbot/api_token` (macOS/Linux) or `%USERPROFILE%\.tuitbot\api_token` (Windows).

The token file is created by the TuitBot server on first start. The plugin reads it once at load time and caches it in memory.

**Auth flow**:
1. Plugin loads → reads `~/.tuitbot/api_token` from disk.
2. If file missing → shows Notice: "TuitBot API token not found. Start TuitBot once to generate it."
3. On each request → sends `Authorization: Bearer <token>` header.
4. If 401 response → shows Notice: "Authentication failed. Check your API token."

## Request Schema

### `POST /api/vault/send-selection`

**Content-Type**: `application/json`

**Body** (JSON):

```json
{
  "vault_name": "string",
  "file_path": "string",
  "selected_text": "string",
  "heading_context": "string | null",
  "selection_start_line": "integer",
  "selection_end_line": "integer",
  "note_title": "string | null",
  "frontmatter_tags": "string[] | null"
}
```

### Field Descriptions

| Field | Type | Required | Constraints | Description |
|---|---|---|---|---|
| `vault_name` | string | yes | 1-255 chars | Obsidian vault name from `app.vault.getName()`. |
| `file_path` | string | yes | Relative to vault root, ends in `.md` | Note path within the vault (e.g., `notes/async-rust.md`). |
| `selected_text` | string | yes | 1-10000 chars, trimmed | The user's selection or extracted block text. |
| `heading_context` | string \| null | no | Heading path with ` > ` separator | Heading hierarchy at the selection. Example: `"# Title > ## Setup > ### Config"`. Null when the selection is before any heading. |
| `selection_start_line` | integer | yes | >= 0 | Zero-indexed start line of the selection in the editor. |
| `selection_end_line` | integer | yes | >= `selection_start_line` | Zero-indexed end line of the selection in the editor. |
| `note_title` | string \| null | no | Extracted from frontmatter `title` or filename | Human-readable note title. Falls back to filename (without `.md`). |
| `frontmatter_tags` | string[] \| null | no | Array of strings | Tags from the note's YAML frontmatter. Null if no frontmatter or no tags. |

### JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["vault_name", "file_path", "selected_text", "selection_start_line", "selection_end_line"],
  "properties": {
    "vault_name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 255
    },
    "file_path": {
      "type": "string",
      "minLength": 1,
      "pattern": "\\.md$"
    },
    "selected_text": {
      "type": "string",
      "minLength": 1,
      "maxLength": 10000
    },
    "heading_context": {
      "type": ["string", "null"]
    },
    "selection_start_line": {
      "type": "integer",
      "minimum": 0
    },
    "selection_end_line": {
      "type": "integer",
      "minimum": 0
    },
    "note_title": {
      "type": ["string", "null"]
    },
    "frontmatter_tags": {
      "type": ["array", "null"],
      "items": { "type": "string" }
    }
  },
  "additionalProperties": false
}
```

## Response Schema

### Success: `200 OK`

```json
{
  "status": "received",
  "session_id": "<uuid>",
  "composer_url": "/compose?selection=<uuid>"
}
```

| Field | Type | Description |
|---|---|---|
| `status` | string | Always `"received"` on success. |
| `session_id` | string (UUID v4) | Unique identifier for this selection. Used to retrieve the selection later. |
| `composer_url` | string | Relative URL to open the composer pre-loaded with this selection. |

### Success JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["status", "session_id", "composer_url"],
  "properties": {
    "status": { "type": "string", "const": "received" },
    "session_id": { "type": "string", "format": "uuid" },
    "composer_url": { "type": "string" }
  },
  "additionalProperties": false
}
```

## Error Responses

| HTTP Status | Error Code | When | Example Response |
|---|---|---|---|
| `401 Unauthorized` | `invalid_token` | Missing or invalid `Authorization` header. | `{"error": "invalid_token", "message": "Missing or invalid Authorization header"}` |
| `422 Unprocessable Entity` | `validation_error` | Request body fails schema validation. | `{"error": "validation_error", "message": "selected_text must not be empty"}` |
| `413 Payload Too Large` | `payload_too_large` | `selected_text` exceeds 10000 characters. | `{"error": "payload_too_large", "message": "selected_text exceeds 10000 character limit"}` |
| `429 Too Many Requests` | `rate_limited` | More than 10 selections per minute per account. | `{"error": "rate_limited", "message": "Rate limit exceeded. Try again in 6 seconds."}` |
| `500 Internal Server Error` | `internal_error` | Server-side failure. | `{"error": "internal_error", "message": "Failed to store selection"}` |

### Error Response Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["error", "message"],
  "properties": {
    "error": { "type": "string" },
    "message": { "type": "string" }
  },
  "additionalProperties": false
}
```

## Concrete Examples

### Example 1: Selection Send (heading context present)

User selects two paragraphs under a nested heading in `notes/async-rust.md`.

**Request**:

```http
POST /api/vault/send-selection HTTP/1.1
Host: 127.0.0.1:3001
Content-Type: application/json
Authorization: Bearer tb_k7x9m2p4q8r1w5y3

{
  "vault_name": "marketing",
  "file_path": "notes/async-rust.md",
  "selected_text": "The key insight is that async in Rust gives you zero-cost abstractions for concurrent code. Unlike Go's goroutines, Rust's async model compiles down to state machines with no runtime overhead.\n\nThis is a powerful differentiator when pitching Rust for backend services.",
  "heading_context": "# Async Patterns in Rust > ## The Async Story > ### Zero-Cost Abstractions",
  "selection_start_line": 45,
  "selection_end_line": 52,
  "note_title": "Async Patterns in Rust",
  "frontmatter_tags": ["rust", "async", "programming"]
}
```

**Response**:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "received",
  "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "composer_url": "/compose?selection=a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

### Example 2: Block Send (no selection, no heading)

User's cursor is in a paragraph at the top of a note with no headings.

**Request**:

```http
POST /api/vault/send-selection HTTP/1.1
Host: 127.0.0.1:3001
Content-Type: application/json
Authorization: Bearer tb_k7x9m2p4q8r1w5y3

{
  "vault_name": "marketing",
  "file_path": "daily/2026-03-21.md",
  "selected_text": "Had an interesting conversation about why most SaaS onboarding flows fail. The core issue isn't complexity — it's that the first screen asks for information instead of delivering value.",
  "heading_context": null,
  "selection_start_line": 3,
  "selection_end_line": 5,
  "note_title": "2026-03-21",
  "frontmatter_tags": null
}
```

**Response**:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "received",
  "session_id": "f9e8d7c6-b5a4-3210-fedc-ba0987654321",
  "composer_url": "/compose?selection=f9e8d7c6-b5a4-3210-fedc-ba0987654321"
}
```

### Example 3: Validation Error

Empty selection (plugin should prevent this, but the server validates too).

**Request**:

```http
POST /api/vault/send-selection HTTP/1.1
Host: 127.0.0.1:3001
Content-Type: application/json
Authorization: Bearer tb_k7x9m2p4q8r1w5y3

{
  "vault_name": "marketing",
  "file_path": "notes/empty.md",
  "selected_text": "",
  "heading_context": null,
  "selection_start_line": 0,
  "selection_end_line": 0,
  "note_title": "empty",
  "frontmatter_tags": null
}
```

**Response**:

```http
HTTP/1.1 422 Unprocessable Entity
Content-Type: application/json

{
  "error": "validation_error",
  "message": "selected_text must not be empty"
}
```

## Trigger Flow

```
┌──────────────────────────────────────────────────────────────────┐
│ Obsidian Editor                                                  │
│                                                                  │
│  1. User selects text (or places cursor in a paragraph)          │
│  2. Opens Command Palette → "Send selection to TuitBot"          │
│     (or "Send current block to TuitBot")                         │
└───────────────────────────┬──────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ TuitBot Obsidian Plugin                                          │
│                                                                  │
│  3. Reads editor state:                                          │
│     - editor.getSelection() or extractBlock(lines, cursorLine)   │
│     - editor.getCursor('from').line / getCursor('to').line       │
│  4. Resolves heading context via app.metadataCache               │
│  5. Reads note title from frontmatter or filename                │
│  6. Reads frontmatter tags                                       │
│  7. Assembles GhostwriterPayload                                 │
│  8. POST /api/vault/send-selection with Bearer token             │
└───────────────────────────┬──────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ TuitBot Server (127.0.0.1:3001)                                  │
│                                                                  │
│  9. Validates token → 401 if invalid                             │
│ 10. Validates payload → 422 if schema violation                  │
│ 11. Stores selection in vault_selections table (30-min TTL)      │
│ 12. Returns { status, session_id, composer_url }                 │
│ 13. Emits WebSocket event: SelectionReceived { session_id }      │
└───────────────────────────┬──────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│ TuitBot Dashboard                                                │
│                                                                  │
│ 14. Receives WebSocket event → navigates to composer_url         │
│ 15. Loads selection → pre-populates composer                     │
│ 16. User generates thread/tweet from the selection               │
└──────────────────────────────────────────────────────────────────┘
```

Steps 9-16 are implemented in later sessions. The plugin (steps 1-8) is complete as of Session 2.

## Backend Requirements Checklist

The session that implements `POST /api/vault/send-selection` must:

- [ ] Create `vault_selections` table: `id` (PK), `account_id`, `session_id` (UUID, unique index), `vault_name`, `file_path`, `selected_text`, `heading_context`, `selection_start_line`, `selection_end_line`, `note_title`, `frontmatter_tags` (JSON), `created_at`, `expires_at`
- [ ] Set `expires_at` = `created_at` + 30 minutes
- [ ] Validate request body against the JSON schema above
- [ ] Enforce `selected_text` max length of 10000 characters (return 413)
- [ ] Rate limit: 10 requests per minute per account (return 429)
- [ ] Return the response schema above on success
- [ ] Scope all queries by `account_id` from the Bearer token
- [ ] Add hourly cleanup: `DELETE FROM vault_selections WHERE expires_at < NOW()`
- [ ] Emit `SelectionReceived { session_id }` WebSocket event to the account's connected dashboard clients
- [ ] Add `GET /api/vault/selection/{session_id}` to retrieve a stored selection
- [ ] In Cloud mode: `GET /api/vault/selection/{session_id}` must not return raw `selected_text` (privacy invariant)
- [ ] Add integration tests for success, 401, 422, 413, 429, and TTL expiry
- [ ] All new code passes `cargo fmt --all && cargo clippy --workspace -- -D warnings && RUSTFLAGS="-D warnings" cargo test --workspace`
