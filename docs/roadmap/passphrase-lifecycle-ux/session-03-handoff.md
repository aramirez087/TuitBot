# Session 03 Handoff — Reset Command Fast Path & Live Reload

**Date:** 2026-03-01
**Status:** Complete
**Branch:** fix/uninstall

---

## Completed

- [x] `passphrase_hash_mtime()` helper added to `tuitbot-core::auth::passphrase`
- [x] `passphrase_hash_mtime` field added to `AppState` in `tuitbot-server::state`
- [x] `--reset-passphrase` moved to early fast path in `main.rs` — skips DB init, API token, config, LLM, watchtower, and port binding
- [x] Reset prints only the bare passphrase to stdout and exits with code 0
- [x] `tracing::info!("starting tuitbot server")` moved below early exit
- [x] Mtime-based passphrase reload in login handler detects out-of-band resets
- [x] LAN reset endpoint (`/api/settings/lan/reset-passphrase`) updates mtime after in-process reset
- [x] Settings init/claim handler updates mtime after creating passphrase hash
- [x] Factory reset clears mtime alongside passphrase hash
- [x] All `AppState` constructions updated (server, tests, Tauri)
- [x] Two regression tests: `login_detects_out_of_band_passphrase_reset`, `login_detects_new_passphrase_file`
- [x] Two unit tests: `passphrase_hash_mtime_returns_some_after_create`, `passphrase_hash_mtime_changes_after_reset`
- [x] `docs/lan-mode.md` updated with new reset semantics
- [x] `cargo fmt --all && cargo fmt --all --check` — passes
- [x] `RUSTFLAGS="-D warnings" cargo test --workspace` — all tests pass (12 test suites, 0 failures)
- [x] `cargo clippy --workspace -- -D warnings` — passes

## Decisions Made

1. **Bare stdout output for `--reset-passphrase`.** The command prints only the passphrase — no `tracing::info!` prefix, no "Web login passphrase (reset):" label, no padding newlines. This makes it scriptable: `NEW_PASS=$(tuitbot-server --reset-passphrase)`.

2. **Mtime comparison uses `!=` not `>=`.** The charter recommended `>=` for filesystem granularity. However, `>=` is equivalent to `>` OR `==`, and `==` would cause a reload on every login since `disk == cached` after startup. Using `!=` correctly detects changes while avoiding unnecessary reloads. The edge case where two writes in the same filesystem tick produce identical mtimes is not realistic for passphrase reset operations.

3. **Reload scope is login-only.** The mtime check runs only in the `login` handler, not in the auth middleware or status endpoint. Login is the only path that verifies the passphrase against the hash. Auth middleware checks session cookies and bearer tokens — it never touches the passphrase hash.

4. **All in-process mutations update mtime too.** The LAN reset endpoint, settings init/claim handler, and factory reset all update the cached mtime alongside the passphrase hash. This prevents redundant file reads and confusing "reloaded from disk" log lines on the next login.

5. **Tauri AppState brought into sync.** The Tauri `lib.rs` was missing `connector_config` and `pending_oauth` fields (pre-existing gap). Added those alongside the new `passphrase_hash_mtime` field.

6. **Tracing subscriber stays above early exit.** If `reset_passphrase()` returns `Err`, anyhow will format the error through the tracing subscriber. Keeping tracing init above the early exit ensures clean error output.

## Files Modified

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/auth/passphrase.rs` | Added `passphrase_hash_mtime()` helper + 2 unit tests |
| `crates/tuitbot-server/src/state.rs` | Added `passphrase_hash_mtime: RwLock<Option<SystemTime>>` field |
| `crates/tuitbot-server/src/main.rs` | Early-exit fast path for `--reset-passphrase`; moved startup log below exit; initialized mtime; added field to AppState construction |
| `crates/tuitbot-server/src/auth/routes.rs` | Mtime check before passphrase verify in login handler |
| `crates/tuitbot-server/src/routes/lan.rs` | Update mtime after in-process reset |
| `crates/tuitbot-server/src/routes/settings.rs` | Update mtime after claim; clear mtime on factory reset |
| `crates/tuitbot-server/tests/fresh_install_auth.rs` | Added `passphrase_hash_mtime` field to AppState constructions; added 2 regression tests |
| `crates/tuitbot-server/tests/api_tests.rs` | Added `passphrase_hash_mtime` field to all AppState constructions (19 instances) |
| `crates/tuitbot-server/tests/compose_contract_tests.rs` | Added `passphrase_hash_mtime` field to AppState construction |
| `crates/tuitbot-server/tests/factory_reset.rs` | Added `passphrase_hash_mtime` field to AppState construction |
| `dashboard/src-tauri/src/lib.rs` | Added `passphrase_hash_mtime`, `connector_config`, `pending_oauth` fields to AppState construction |
| `docs/lan-mode.md` | Updated reset command docs: bare output, no restart needed, scriptable |
| `docs/roadmap/passphrase-lifecycle-ux/session-03-handoff.md` | This document |

## What Did NOT Change

- **Frontend code** — no modifications to any Svelte component or store
- **Tauri bearer-token auth** — unchanged
- **Session management** — existing sessions continue working after reset
- **Database schema** — no migrations
- **Config file format** — no changes to `config.toml` structure
- **MCP server** — `tuitbot-mcp` AppState is separate and unaffected

## Manual Verification Steps

### A4 — `--reset-passphrase` while server IS running
1. Start server: `cargo run -p tuitbot-server`
2. In another terminal: `cargo run -p tuitbot-server -- --reset-passphrase`
3. Verify: only the passphrase is printed, exit code 0, no "Address already in use"

### A5 — `--reset-passphrase` while server is NOT running
1. Stop the server
2. Run: `cargo run -p tuitbot-server -- --reset-passphrase`
3. Verify: same clean output as A4

### A6 — Login after out-of-band CLI reset
1. Start server, log in with original passphrase
2. Run `--reset-passphrase` in another terminal, note new passphrase
3. Try logging in with old passphrase → should fail
4. Try logging in with new passphrase → should succeed (no restart)

### A7 — Login after LAN reset endpoint
1. Start server, log in
2. Call `POST /api/settings/lan/reset-passphrase`
3. Use the returned passphrase to log in → should succeed

## Open Issues

None blocking Session 04.

## Inputs for Session 04

**Scope:** Final polish, release validation, and initiative wrap-up.

**Remaining items from charter:**
- Validate `cargo package --workspace --allow-dirty` and `release-plz update` pass
- End-to-end smoke test covering the full lifecycle (install → onboard → use → reset → re-login)
- Review all acceptance scenarios (A1–A7) holistically
- Close out the initiative with a summary document

**Files to review:**
- `docs/roadmap/passphrase-lifecycle-ux/charter.md` — verify all acceptance criteria met
- All session handoffs (01, 02, 03) for completeness
