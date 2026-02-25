# Session 02 — Refactor Notes

## What Changed

### New Modules (7 files)

1. **`contract/mod.rs`** (10 lines) — Re-exports envelope and error types
2. **`contract/envelope.rs`** (237 lines) — `ToolResponse`, `ToolError`, `ToolMeta` + 15 unit tests. Moved verbatim from `tools/response.rs`; no behavioral changes.
3. **`contract/error.rs`** (120 lines) — `ProviderError` enum (7 variants), `provider_error_to_response()` helper, 4 unit tests. Error taxonomy extracted from `x_actions/mod.rs::x_error_to_response()` and made provider-agnostic.
4. **`provider/mod.rs`** (32 lines) — `SocialReadProvider` trait with 3 read methods.
5. **`provider/x_api.rs`** (57 lines) — `XApiProvider` adapter: wraps `&dyn XApiClient`, maps `XApiError` → `ProviderError`.
6. **`kernel/mod.rs`** (8 lines) — Module declarations.
7. **`kernel/read.rs`** (60 lines) — 3 provider-agnostic tool functions: `get_tweet`, `get_user_by_username`, `search_tweets`.
8. **`kernel/tests.rs`** (176 lines) — `MockProvider` + `ErrorProvider`, 9 tests covering success, errors, and envelope structure.

### Modified Files (3 files)

9. **`tools/response.rs`** — Replaced 300 lines with 7-line re-export shim from `contract::envelope`. All downstream imports resolve unchanged.
10. **`tools/x_actions/read.rs`** — 3 functions (`get_tweet_by_id`, `get_user_by_username`, `search_tweets`) rewritten to create `XApiProvider` from `state.x_client` and delegate to `kernel::read::*`. 4 functions unchanged (`get_user_mentions`, `get_user_tweets`, `get_home_timeline`, `get_x_usage`).
11. **`lib.rs`** — Added `pub mod contract; mod kernel; mod provider;` declarations.

### Unchanged

- `server.rs` — All tool handler dispatches unchanged; still calls `tools::x_actions::*`.
- `state.rs` — `AppState` struct unchanged.
- `requests.rs` — All request structs unchanged.
- `tools/x_actions/mod.rs` — `x_error_to_response`, `error_response`, `not_configured_response`, `no_user_id_response` retained for non-migrated tools. These will be removed once all X tools route through kernel.
- All existing tests — Every original test passes without modification.

## Design Decisions

### Why keep `x_actions/mod.rs` error helpers?
The 4 non-migrated read tools (`get_user_mentions`, `get_user_tweets`, `get_home_timeline`, `get_x_usage`) and all write/engage tools still use `error_response()` directly. Removing it now would require migrating all tools at once. The helpers will be deprecated when the remaining tools move to kernel.

### Why `&dyn XApiClient` in XApiProvider, not `Arc<dyn XApiClient>`?
The provider is created per-call from the `SharedState` reference — no need for shared ownership. The borrow is valid for the duration of the async tool call. This avoids unnecessary `Arc` cloning.

### Why not introduce `SocialWriteProvider` yet?
Write tools are policy-gated (require `AppState` for policy evaluation + mutation recording). The write provider trait needs to account for these concerns. Deferred to Session 03.

## Test Coverage

| Layer | Tests | Description |
|-------|-------|-------------|
| `contract::envelope` | 15 | All envelope builder methods, roundtrip, edge cases |
| `contract::error` | 4 | Error taxonomy mapping + JSON output |
| `kernel::tests` | 9 | Mock provider success (4), error paths (3), envelope structure (2) |
| `x_actions::tests::read` | 12 | Original tests all pass (unchanged) |
| **Total new** | **28** | Added by session 02 |
