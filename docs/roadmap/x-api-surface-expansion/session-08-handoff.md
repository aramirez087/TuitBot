# Session 08 Handoff — Idempotency & Auditable Mutation History

## What Changed

Before this session, mutation tools had basic in-memory dedup (30s window) but no persistent idempotency, no audit trail, no correlation IDs, and no rollback guidance. Agent retry storms or restarts could cause duplicate tweets/likes/follows with no way to trace or undo them.

This session adds:
1. **DB-backed idempotency** — 5-minute persistent dedup window using SHA-256 fingerprints, layered on top of the existing 30s in-memory store.
2. **Mutation audit trail** — Every mutation attempt (13 tools) creates a `mutation_audit` record with correlation ID, params hash, status lifecycle, and timing.
3. **Rollback guidance** — Advisory metadata on success responses mapping each mutation to its undo action (e.g., `post_tweet` → `delete_tweet`, `like_tweet` → `unlike_tweet`).
4. **Query tools** — `get_recent_mutations` and `get_mutation_detail` for agents to review what writes were performed and look up individual mutations by correlation ID.

### Design Decisions

- **Dual-layer idempotency**: In-memory (30s, fast hash) catches immediate retries; DB-backed (5min, SHA-256) catches restarts and longer retry storms. Both must pass before a mutation proceeds.
- **Audit gate ordering**: Validation checks (length, scraper guard, policy gate, client availability) run BEFORE the audit gate. This ensures audit records are only created for genuine mutation attempts, not rejected requests.
- **Rollback is advisory**: The `rollback` metadata in responses tells agents HOW to undo an action but does not auto-execute. This avoids safety issues with automated rollback.
- **Correlation ID on errors too**: Failed mutations get correlation IDs in both the DB record and the error response metadata, enabling full traceability.

## API Contracts

### Mutation Success Response (all 13 tools)

```json
{
  "success": true,
  "data": { "liked": true, "tweet_id": "123" },
  "meta": {
    "tool_version": "1.0",
    "elapsed_ms": 342,
    "correlation_id": "a1b2c3d4-...",
    "rollback": {
      "reversible": true,
      "undo_tool": "x_unlike_tweet",
      "undo_params": { "tweet_id": "123" },
      "note": "Unlike to reverse"
    }
  }
}
```

### Mutation Error Response

```json
{
  "success": false,
  "error": { "code": "x_rate_limited", "message": "...", "retryable": true },
  "meta": {
    "elapsed_ms": 150,
    "correlation_id": "e5f6g7h8-..."
  }
}
```

### Duplicate Detection Response

```json
{
  "success": true,
  "data": {
    "duplicate": true,
    "original_correlation_id": "a1b2c3d4-...",
    "cached_result": { "liked": true, "tweet_id": "123" },
    "message": "Identical like_tweet was already executed successfully. Returning cached result."
  }
}
```

### `get_recent_mutations`

```json
{
  "success": true,
  "data": {
    "mutations": [
      {
        "correlation_id": "a1b2c3d4-...",
        "tool_name": "post_tweet",
        "status": "success",
        "params_summary": "{\"text\":\"Hello world!\"}",
        "result_summary": "{\"id\":\"999\",\"text\":\"Hello world!\"}",
        "error_message": null,
        "elapsed_ms": 342,
        "created_at": "2026-02-27T10:30:00.000Z",
        "completed_at": "2026-02-27T10:30:00.342Z"
      }
    ],
    "count": 1
  }
}
```

### `get_mutation_detail`

Returns the full `MutationAuditEntry` including `rollback_action`, `params_hash`, `account_id`.

## Rollback Guidance Matrix

| Tool | Reversible | Undo Tool | Note |
|------|-----------|-----------|------|
| `post_tweet` | Yes | `delete_tweet` | Delete to reverse |
| `reply_to_tweet` | Yes | `delete_tweet` | Delete to reverse |
| `quote_tweet` | Yes | `delete_tweet` | Delete to reverse |
| `delete_tweet` | No | — | Deletion is permanent |
| `post_thread` | Yes | `delete_tweet` (per ID) | Delete each tweet individually |
| `like_tweet` | Yes | `unlike_tweet` | Unlike to reverse |
| `unlike_tweet` | Yes | `like_tweet` | Like again to reverse |
| `follow_user` | Yes | `unfollow_user` | Unfollow to reverse |
| `unfollow_user` | Yes | `follow_user` | Follow again to reverse |
| `retweet` | Yes | `unretweet` | Unretweet to reverse |
| `unretweet` | Yes | `retweet` | Retweet again to reverse |
| `bookmark_tweet` | Yes | `unbookmark_tweet` | Unbookmark to reverse |
| `unbookmark_tweet` | Yes | `bookmark_tweet` | Bookmark again to reverse |
| `upload_media` | No | — | Media upload is permanent |

## Failure Modes

| Scenario | Behavior |
|----------|----------|
| Duplicate within 30s (in-memory) | Rejected immediately, no DB record |
| Duplicate within 5min (DB) | Returns cached result with `duplicate: true` |
| DB failure creating audit record | Returns `db_error`, mutation not attempted |
| API failure after audit record | Record marked `failure` with error, correlation ID in response |
| Thread partial failure | Record marked `failure` with posted IDs, partial data in response |
| Scraper guard / policy gate rejection | No audit record created (rejected before gate) |

## Files Changed

| File | Change |
|------|--------|
| `migrations/20260227000017_mutation_audit.sql` | New migration: `mutation_audit` table + 5 indexes |
| `crates/tuitbot-core/migrations/20260227000017_mutation_audit.sql` | Crate-level copy |
| `crates/tuitbot-core/src/storage/mutation_audit.rs` | New: `MutationAuditEntry`, 10 DB functions, 10 tests |
| `crates/tuitbot-core/src/storage/mod.rs` | +`mutation_audit` module, table assertion |
| `crates/tuitbot-mcp/src/tools/idempotency.rs` | +`MutationGuard`, +`begin_mutation`, +`uuid_v4`, 5 new tests |
| `crates/tuitbot-mcp/src/tools/rollback.rs` | New: rollback guidance matrix (14 tools), 7 tests |
| `crates/tuitbot-mcp/src/tools/mod.rs` | +`rollback` module |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/audit.rs` | New: `begin_audited_mutation`, `complete_audited_success/failure`, `audited_x_error_response` |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/mod.rs` | +`audit` module, `#[allow(dead_code)]` on legacy `error_response` |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/write.rs` | All 5 mutation handlers wired to audit gate |
| `crates/tuitbot-mcp/src/tools/workflow/x_actions/engage.rs` | All 8 mutation handlers wired to audit gate |
| `crates/tuitbot-mcp/src/tools/workflow/mutation_audit.rs` | New: `get_recent_mutations`, `get_mutation_detail` query tools |
| `crates/tuitbot-mcp/src/tools/workflow/mod.rs` | +`mutation_audit` module |
| `crates/tuitbot-mcp/src/contract/envelope.rs` | +`correlation_id`, +`rollback` fields on `ToolMeta` |
| `crates/tuitbot-mcp/src/contract/error.rs` | +`provider_error_to_audited_response` |
| `crates/tuitbot-mcp/src/requests.rs` | +`GetRecentMutationsRequest`, +`GetMutationDetailRequest` |
| `crates/tuitbot-mcp/src/tools/manifest.rs` | +2 tool entries |
| `crates/tuitbot-mcp/src/server/write.rs` | +2 tool handlers |
| `crates/tuitbot-mcp/src/server/admin.rs` | +2 tool handlers |
| `crates/tuitbot-mcp/src/tools/boundary_tests.rs` | Updated tool counts (+2 for write and admin) |

## Test Coverage

22 new tests across 2 crates:

**`tuitbot-core` (10 tests):**
- `insert_and_complete_success` — Full lifecycle: pending → success
- `insert_and_complete_failure` — Failure recording with error message
- `find_recent_duplicate_within_window` — SHA-256 fingerprint dedup
- `no_duplicate_for_different_tool` — Cross-tool independence
- `idempotency_key_lookup` — Caller-provided key lookup
- `get_recent_with_filters` — Tool name and status filters
- `mark_duplicate_records_original` — Duplicate chain tracking
- `status_counts_aggregation` — Status counts grouping
- `params_hash_deterministic` — SHA-256 consistency
- (table assertion updated)

**`tuitbot-mcp` (12 tests):**
- `begin_mutation_creates_audit_record` — DB record with correlation ID
- `begin_mutation_detects_db_duplicate` — 5min dedup window
- `begin_mutation_allows_after_failure` — Retry after failed attempt
- `uuid_v4_format` — UUID v4 format validation
- `uuid_v4_unique` — 100 unique UUIDs
- `rollback_post_tweet_reversible` — Rollback guidance for post
- `rollback_delete_tweet_irreversible` — Delete is permanent
- `rollback_like_tweet_reversible` — Like → unlike mapping
- `rollback_follow_user_reversible` — Follow → unfollow mapping
- `rollback_post_thread_multiple_ids` — Thread → per-ID deletes
- `rollback_upload_media_irreversible` — Upload is permanent
- `rollback_unknown_tool_irreversible` — Default for unknown tools

## Verification

```bash
cargo fmt --all && cargo fmt --all --check     # clean
RUSTFLAGS="-D warnings" cargo test --workspace  # all pass (1 pre-existing flaky env var test)
cargo clippy --workspace -- -D warnings         # clean
```

## What's NOT Changed

- In-memory `IdempotencyStore` — Still runs as a fast-path first layer, unchanged behavior
- Policy gate system — Only consumed, not modified
- Action log system (`action_log` table) — Separate system, not modified
- X API usage tracking (`x_api_usage` table) — Separate system, not modified
- Readonly/ApiReadonly profiles — No tools added there
- Dry-run tools — No audit (they're read-only validation)
- Universal request tools (`x_get`/`x_post`/`x_put`/`x_delete`) — Not wired to audit (admin-only escape hatch)

## Operational Guidance

### For Agents

1. **Correlation IDs are your receipt** — Every mutation response includes a `correlation_id` in `meta`. Use `get_mutation_detail` to look up any past mutation.
2. **Check rollback guidance** — The `rollback` field in success responses tells you exactly how to undo an action. Use it before asking the user.
3. **Respect duplicate detection** — If you get `duplicate: true`, the mutation already succeeded. Use the `cached_result` instead of retrying.
4. **Use `get_recent_mutations`** — Before performing related mutations, check what was recently done to avoid conflicts.

### For Users

1. All mutations are now tracked in the `mutation_audit` table with full before/after state.
2. The `correlation_id` in responses can be used to trace any mutation through the system.
3. Rollback guidance is advisory — agents will suggest undo actions but won't auto-execute them.

## Next Steps

- **Caller-provided idempotency keys**: The `idempotency_key` column exists but isn't yet exposed in request types. Agents could provide their own dedup keys.
- **Audit cleanup job**: Periodic cleanup of old audit records (>90 days) to prevent unbounded table growth.
- **Universal request audit**: Wire audit logging into `x_post`/`x_put`/`x_delete` universal request tools.
- **Account-scoped audit**: Wire account ID from multi-account support into audit records (currently defaults to `"default"`).
