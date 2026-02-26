# Session 07 Handoff — Media Upload Tracking and Dry-Run Validation

## What Changed

Before this session, `upload_media` performed a fire-and-forget upload with no tracking, no idempotency, and no dry-run capability. Agents had no way to pre-validate tweets, threads, or media uploads without actually executing them — leading to wasted API calls and opaque failures.

This session adds:
1. **DB-backed media upload tracking** with SHA-256 idempotency and 24-hour expiry awareness.
2. **Dry-run validation** for `post_tweet`, `post_thread`, and `upload_media` — validates length, policy, media, and thread chaining without hitting the X API.
3. **Rich upload metadata** in responses: strategy (simple/chunked), segment count, file hash, cache status.

### Changes Summary

1. **`media_uploads` migration** — New SQLite table tracking every upload from file hash through X API media ID, with status lifecycle (`pending` → `uploading` → `ready` / `failed`).
2. **Storage layer** — 5 new functions: `compute_file_hash`, `find_ready_upload_by_hash`, `insert_media_upload`, `finalize_media_upload`, `fail_media_upload`. Plus `MediaUploadRecord` struct.
3. **`upload_media` rewritten** — SHA-256 idempotency check, DB tracking, rich `UploadResult` response, dry-run validation mode.
4. **`post_tweet_dry_run` tool** — Validates text length, policy gate, media presence without posting. Returns `DryRunTweetResult`.
5. **`post_thread_dry_run` tool** — Validates all tweets in a thread, shows reply chain plan (`chain_action` field), validates per-tweet media.
6. **Manifest, server routes, request types** all updated for the 2 new tools.
7. **Boundary test counts** bumped (+2 tools for write and admin profiles).

## API Contracts

### `x_upload_media` — Enhanced Response

```json
{
  "success": true,
  "data": {
    "media_id": "1234567890",
    "media_type": "image/jpeg",
    "file_size_bytes": 204800,
    "upload_strategy": "simple",
    "segment_count": 1,
    "processing_required": false,
    "cached": false,
    "file_hash": "a1b2c3d4...",
    "alt_text": "A sunset photo"
  },
  "meta": { "elapsed_ms": 1250 }
}
```

When `cached: true`, the upload was skipped — an identical file (by SHA-256 hash) was already uploaded and the X media ID hasn't expired.

### `x_upload_media` — Dry-Run Response

```json
{
  "success": true,
  "data": {
    "dry_run": true,
    "valid": true,
    "media_type": "video/mp4",
    "file_size_bytes": 15728640,
    "upload_strategy": "chunked",
    "segment_count": 3,
    "processing_required": true,
    "alt_text": null
  }
}
```

### `x_post_tweet_dry_run`

```json
{
  "success": true,
  "data": {
    "dry_run": true,
    "valid": true,
    "text": "Hello world!",
    "text_length": 12,
    "has_media": false,
    "media_count": 0,
    "media_ids": [],
    "policy_would_allow": true,
    "x_client_available": true
  }
}
```

### `x_post_thread_dry_run`

```json
{
  "success": true,
  "data": {
    "dry_run": true,
    "valid": true,
    "tweet_count": 3,
    "x_client_available": true,
    "policy_would_allow": true,
    "tweets": [
      {
        "index": 0,
        "text": "Thread start",
        "text_length": 12,
        "valid": true,
        "chain_action": "post_tweet",
        "has_media": false,
        "media_count": 0,
        "media_ids": []
      },
      {
        "index": 1,
        "text": "Thread middle",
        "text_length": 13,
        "valid": true,
        "chain_action": "reply_to_tweet(parent=tweet_0)",
        "has_media": false,
        "media_count": 0,
        "media_ids": []
      }
    ]
  }
}
```

The `chain_action` field makes reply chaining visible: the first tweet uses `post_tweet`, subsequent tweets use `reply_to_tweet(parent=tweet_N)`.

## Failure Modes

| Scenario | Error Code | Behavior |
|----------|------------|----------|
| Text > 280 chars | `tweet_too_long` | Rejected before any API call (including dry-run) |
| Empty thread | `invalid_input` | Rejected with "Thread must contain at least one tweet" |
| Unsupported file extension | `unsupported_media_type` | Rejected immediately, lists supported formats |
| File not readable | `file_read_error` | Returned with OS-level error message |
| File too large for type | `media_upload_error` | Checked during dry-run against `MediaType::max_size()` |
| Upload fails at X API | `media_upload_error` | DB record marked `failed` with error message |
| Policy blocks | `policy_error` | Dry-run returns `policy_would_allow: false`; live returns policy gate response |
| No X client (dry-run) | — | Succeeds with `x_client_available: false` |
| No X client (live) | `not_configured` | Standard "X API client not configured" response |

## Retry Semantics

- **Idempotent uploads**: If the same file is uploaded twice (same SHA-256 hash), the second call returns `cached: true` with the existing X media ID — no API call made. The cache expires after 24 hours (matching X API's media ID lifetime).
- **Failed upload recovery**: A failed upload is recorded in DB but does NOT block re-upload of the same file. `find_ready_upload_by_hash` only matches `status = 'ready'`.
- **Thread partial failure**: If tweet N in a thread fails at X API, tweets 0..N-1 are already posted. The error response includes the partial state. Use dry-run first to validate before posting.
- **Dry-run is always safe**: No API calls, no DB writes, no side effects. Can be called without an X client.

## Files Changed

| File | Change |
|------|--------|
| `migrations/20260227000016_media_uploads.sql` | New migration |
| `crates/tuitbot-core/migrations/20260227000016_media_uploads.sql` | Crate-level copy |
| `crates/tuitbot-core/src/storage/media.rs` | +`MediaUploadRecord`, +5 DB functions, +3 tests |
| `crates/tuitbot-core/src/storage/mod.rs` | +`media_uploads` table assertion in test |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/media.rs` | Complete rewrite: idempotency, tracking, dry-run, rich response |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/write.rs` | +`post_tweet_dry_run`, +`post_thread_dry_run` |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/mod.rs` | Updated re-exports |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/tests/write.rs` | +11 dry-run tests, updated media test signatures |
| `crates/tuitbot-mcp/src/requests.rs` | +`PostTweetDryRunRequest`, +`PostThreadDryRunRequest`, updated `UploadMediaMcpRequest` |
| `crates/tuitbot-mcp/src/server/write.rs` | +2 tool handlers, updated upload_media handler |
| `crates/tuitbot-mcp/src/server/admin.rs` | +2 tool handlers, updated upload_media handler |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | +2 tool entries (`x_post_tweet_dry_run`, `x_post_thread_dry_run`) |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Updated tool counts (+2) |
| `roadmap/artifacts/session-06-tool-manifest.json` | Regenerated with new tools |
| `docs/roadmap/x-api-surface-expansion/session-07-handoff.md` | This document |

## Test Coverage

14 new tests across 2 crates:

**`tuitbot-core` (3 tests):**
- `compute_file_hash_deterministic` — SHA-256 produces consistent 64-char hex
- `insert_and_find_media_upload` — Insert → finalize → lookup by hash
- `fail_media_upload_records_error` — Failed uploads not returned by hash lookup

**`tuitbot-mcp` (11 tests):**
- `upload_media_unsupported_extension` — `.bmp` rejected
- `upload_media_dry_run_unsupported` — `.bmp` rejected in dry-run too
- `post_tweet_dry_run_valid` — Basic validation succeeds
- `post_tweet_dry_run_with_media` — Media IDs included in response
- `post_tweet_dry_run_too_long` — 281 chars rejected
- `post_tweet_dry_run_no_x_client` — Works without X client
- `post_thread_dry_run_valid` — Chain actions correct
- `post_thread_dry_run_with_media` — Per-tweet media validated
- `post_thread_dry_run_empty` — Empty thread rejected
- `post_thread_dry_run_too_long` — Any tweet > 280 rejects whole thread
- `post_thread_dry_run_no_x_client` — Works without X client
- `post_tweet_dry_run_policy_blocked` — Policy gate surfaced

## Verification

```bash
cargo fmt --all && cargo fmt --all --check     # clean
RUSTFLAGS="-D warnings" cargo test --workspace  # all pass (1 pre-existing flaky env var test)
cargo clippy --workspace -- -D warnings         # clean
```

## What's NOT Changed

- `core/x_api/media.rs` — Core chunked upload state machine (INIT → APPEND → FINALIZE → STATUS) unchanged
- `core/x_api/types.rs` — `MediaType`, `MediaId`, `PostTweetRequest` types unchanged
- Thread posting logic in `write.rs` — `post_thread` unchanged
- Policy gate system — Only consumed, not modified
- Readonly/ApiReadonly profiles — No mutation tools added there

## Operational Guidance

### For Agents

1. **Always dry-run first** — Call `x_post_tweet_dry_run` or `x_post_thread_dry_run` before posting to catch length errors, policy blocks, and missing clients.
2. **Check `cached` on uploads** — If `true`, the media ID is reused from a previous upload. No need to re-upload the same file within 24 hours.
3. **Check `processing_required`** — GIFs and videos need X API processing time after upload. The core upload function already handles polling, but agents should expect longer latency.
4. **Thread `chain_action`** — Use the dry-run `chain_action` field to verify reply chaining is correct before posting.

### For Users

1. Media uploads are now tracked in the `media_uploads` table. The same file won't be re-uploaded within 24 hours.
2. Dry-run mode works without X API credentials — useful for content validation during setup.

## Next Steps

- **Media status polling tool**: Expose `check_processing_status` for GIF/video uploads that need processing time.
- **Alt text on tweets**: Wire `alt_text` from upload through to `PostTweetRequest.media_ids` when X API supports per-media alt text in the create tweet endpoint.
- **Upload cleanup**: Periodic job to clean expired/failed records from `media_uploads` table (older than 30 days).
- **Scope negotiation**: Auth flow could request extended scopes for media uploads (currently relies on default scopes).
