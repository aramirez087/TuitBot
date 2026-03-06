# Session 03 Handoff

## What Was Done

Implemented per-account credential storage and X access backend flows. OAuth tokens and scraper sessions are now resolved per-account instead of using global singleton files. Two non-default accounts can hold different credentials with independent `can_post` state.

### New Files

- **`crates/tuitbot-server/src/routes/x_auth.rs`** — X API credential-linking endpoints: `start_link` (PKCE OAuth start), `complete_link` (code exchange + token save), `link_status` (credential status check). Converts `StoredTokens` -> `auth::Tokens`, saves to account-specific path, evicts stale TokenManagers.

- **`docs/roadmap/dashboard-multi-account/credential-isolation-contract.md`** — Contract documenting per-account file layout, token format, `can_post` computation, X auth linking flow, PKCE state management, and effective config resolution.

### Modified Files

- **`crates/tuitbot-server/src/state.rs`** — Added `account_id: String` to `PendingOAuth` so OAuth callbacks can verify which account initiated the flow.

- **`crates/tuitbot-server/src/routes/connectors.rs`** — Set `account_id: String::new()` in PendingOAuth creation for connector flows (not account-scoped).

- **`crates/tuitbot-server/src/routes/content/mod.rs`** — Converted sync helpers to async account-aware versions:
  - `read_config` -> `read_effective_config(state, account_id)` — loads base config, merges with account overrides for non-default accounts.
  - `read_approval_mode` -> `read_approval_mode(state, account_id)` — uses effective config.
  - `require_post_capable` -> `require_post_capable(state, account_id)` — checks account-specific credential files.
  - Added `can_post_for(state, account_id)` — non-error bool version for status endpoints.

- **`crates/tuitbot-server/src/routes/content/compose.rs`** — Updated all callers to pass `account_id`. `try_post_now` uses `account_data_dir` for scraper backend and `account_token_path` for X API backend. Removed `PathBuf` and `accounts` imports that became unused.

- **`crates/tuitbot-server/src/routes/content/drafts.rs`** — Updated `publish_draft` to use `require_post_capable(state, account_id).await`.

- **`crates/tuitbot-server/src/routes/content/calendar.rs`** — Updated `schedule` to use `read_effective_config(state, account_id).await`.

- **`crates/tuitbot-server/src/routes/runtime.rs`** — `status` endpoint now computes `can_post` via `can_post_for(state, account_id)` instead of checking global `provider_backend` and `data_dir`.

- **`crates/tuitbot-server/src/routes/discovery.rs`** — `queue_reply` now uses `require_post_capable(state, account_id).await` instead of inline global config check. Removed unused `Config` import.

- **`crates/tuitbot-server/src/routes/mod.rs`** — Added `pub mod x_auth`.

- **`crates/tuitbot-server/src/lib.rs`** — Registered x_auth routes (`/accounts/{id}/x-auth/start`, `/accounts/{id}/x-auth/callback`, `/accounts/{id}/x-auth/status`) before the `/accounts/{id}` catch-all.

- **`crates/tuitbot-server/tests/api_tests.rs`** — Added 5 integration tests:
  - `credential_isolation_two_accounts` — two accounts have isolated scraper sessions
  - `can_post_isolated_per_account` — `can_post` is true only for the credentialed account
  - `x_auth_status_no_credentials` — fresh account shows no credentials
  - `x_auth_start_returns_auth_url` — start endpoint returns valid auth URL
  - `x_auth_status_reflects_scraper_session` — status endpoint shows scraper session

## Key Decisions Made

| Decision | Rationale |
|----------|-----------|
| Convert sync helpers to async | All callers are async handlers; async enables DB queries for non-default account config without `block_in_place` |
| Use `auth::Tokens` (not `StoredTokens`) for saved files | `load_tokens` and `TokenManager` expect `auth::Tokens` format with required fields |
| Fill defaults for Optional `StoredTokens` fields during conversion | `refresh_token` defaults to empty string, `expires_at` defaults to 2 hours from now |
| Evict `TokenManager` on credential link | Forces fresh token load on next API call instead of using stale cached tokens |
| `can_post` checks token file existence (not just `true` for x_api) | Account without linked tokens should not report `can_post = true` |
| `PendingOAuth.account_id` empty string for connectors | Connectors are not account-scoped; empty string distinguishes from account flows |

## Quality Gates Passed

- `cargo fmt --all && cargo fmt --all --check` — clean
- `RUSTFLAGS="-D warnings" cargo test --workspace` — all tests pass (including 5 new)
- `cargo clippy --workspace -- -D warnings` — clean

## Open Issues for Session 4

1. **Automation loop scoping.** The automation runtimes (`discovery_loop`, `content_loop`, `mention_loop`) still read the global config. Each runtime should load the effective config for its account. The credential path helpers are now available; the runtimes just need to use them.

2. **WebSocket account scoping.** WebSocket events are broadcast globally. Events should be filtered or tagged by account_id so the frontend only sees events for the active account.

3. **Frontend account switcher and credential linking UI.** The x-auth endpoints are ready for the frontend. The dashboard needs a UI to: (a) switch between accounts, (b) initiate X credential linking via `/x-auth/start`, (c) show credential status per account.

4. **Config reload after PATCH.** When `PATCH /api/settings` modifies `config.toml` (default account), running runtimes don't pick up the change. Pre-existing issue, now more relevant with per-account overrides.

5. **Token refresh for non-default accounts.** The `TokenManager` auto-refreshes tokens but saves to the path it was initialized with. This should work correctly since `get_x_access_token` creates managers with account-specific paths, but should be validated end-to-end.

## Exact Inputs for Session 4

### Files to Modify

- `crates/tuitbot-core/src/automation/runtime.rs` — Accept account_id, load effective config per account
- `crates/tuitbot-core/src/automation/discovery_loop.rs` — Use account-scoped config and credential paths
- `crates/tuitbot-core/src/automation/content_loop.rs` — Use account-scoped config and credential paths
- `crates/tuitbot-core/src/automation/mention_loop.rs` — Use account-scoped config and credential paths
- `crates/tuitbot-server/src/ws.rs` — Add account_id to WsEvent variants or filter on broadcast

### Key Contracts to Respect

- `read_effective_config` lives in `tuitbot-server`, not `tuitbot-core` (it needs DB access)
- For core automation loops, pass the resolved `Config` at runtime startup (don't query DB from core)
- Default account file paths remain at root level (backward compat)
- Runtimes keyed by `account_id` in `state.runtimes` (already in AppState)
