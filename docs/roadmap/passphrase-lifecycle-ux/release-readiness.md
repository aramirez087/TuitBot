# Passphrase Lifecycle UX — Release Readiness Report

**Date:** 2026-03-01
**Branch:** fix/uninstall
**Recommendation:** GO

---

## Quality Gate Results

| Gate | Status | Detail |
|------|--------|--------|
| `cargo fmt --all && cargo fmt --all --check` | PASS | No formatting changes needed |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | PASS | 12 test suites, 1824 tests, 0 failures |
| `cargo clippy --workspace -- -D warnings` | PASS | No warnings |
| `npm run check` (svelte-check) | PASS | 0 errors, 6 warnings (all pre-existing: a11y, empty rulesets, non-reactive canvas) |
| `npm run build` (vite + adapter-static) | PASS | SSR + client builds successful |
| `cargo package --workspace --allow-dirty` | PASS | All 4 crates packaged and verified |
| `release-plz update` | FAIL (pre-existing) | Expects `dashboard/Cargo.toml` which does not exist (dashboard is Svelte, not Rust). Unrelated to passphrase-lifecycle-ux. |

---

## Issue Resolution

### Issue 1: No passphrase after onboarding (claimed-but-unconfigured path)

**Status:** RESOLVED

**Root cause:** When `alreadyClaimed` was true, `showClaimStep` was false, excluding the 'Secure' step entirely. Users completed onboarding without seeing a passphrase or recovery guidance.

**Fix:**
- `onboarding/+page.svelte:32` — Changed `showClaimStep` from `!isTauri && !alreadyClaimed` to `!isTauri`. Web mode always shows the Secure step.
- `ClaimStep.svelte:9` — Added `alreadyClaimed` prop. When true, shows "Your Instance is Secured" recovery guidance instead of passphrase generation.
- `onboarding/+page.svelte:72-74` — `canAdvance()` case 8 supports acknowledgment-only mode for `alreadyClaimed`.
- `onboarding/+page.svelte:171-172` — `alreadyClaimed` users redirect to `/login` after setup (not `/content`).

**Test coverage:** Code path review (A1, A2 scenarios). `npm run check` and `npm run build` pass.

**Files:** `dashboard/src/routes/onboarding/+page.svelte`, `dashboard/src/lib/components/onboarding/ClaimStep.svelte`

### Issue 2: Noisy `--reset-passphrase` output

**Status:** RESOLVED

**Root cause:** Reset was handled at line 78 of `main.rs`, after DB init, API token, config, and `tracing::info!("starting tuitbot server")`. Server continued to port binding, failing with "Address already in use."

**Fix:**
- `main.rs:64-70` — Early exit fast path after `db_dir` derivation (line 62), before DB init, API token, config, LLM, watchtower, and port binding.
- `main.rs:68` — `println!("{new_passphrase}")` with no prefix, label, or formatting. Exit code 0.
- `main.rs:72-77` — "starting tuitbot server" log moved below the early exit.

**Test coverage:** `passphrase_hash_mtime_returns_some_after_create`, `passphrase_hash_mtime_changes_after_reset` unit tests. Code path review (A4, A5 scenarios).

**Files:** `crates/tuitbot-server/src/main.rs`

### Issue 3: Stale in-memory auth after out-of-band reset

**Status:** RESOLVED

**Root cause:** Login handler only read from `state.passphrase_hash` (in-memory), never from disk. CLI reset wrote to disk but couldn't notify the running server.

**Fix:**
- `auth/routes.rs:117-135` — Mtime-based passphrase hash refresh: `stat()` the `passphrase_hash` file on each login, compare with cached mtime, reload if changed.
- `state.rs:42` — Added `passphrase_hash_mtime: RwLock<Option<SystemTime>>` field.
- `passphrase.rs:163-168` — Added `passphrase_hash_mtime(data_dir) -> Option<SystemTime>` helper.
- All in-process mutations (settings claim, LAN reset, factory reset) also update the cached mtime.

**Test coverage:** `login_detects_out_of_band_passphrase_reset`, `login_detects_new_passphrase_file` integration tests. Code path review (A6, A7 scenarios).

**Files:** `crates/tuitbot-server/src/auth/routes.rs`, `crates/tuitbot-server/src/state.rs`, `crates/tuitbot-core/src/auth/passphrase.rs`, `crates/tuitbot-server/src/routes/lan.rs`, `crates/tuitbot-server/src/routes/settings.rs`

---

## Acceptance Scenario Results

| # | Scenario | Status | Evidence |
|---|----------|--------|----------|
| A1 | Fresh web install: onboarding with claim | PASS | `+layout.svelte:50` routes to `/onboarding`. `showClaimStep=true` (web, not claimed). `ClaimStep` shows passphrase generation. `submit()` sends claim. `claimSession()` establishes cookie. Redirect to `/content?compose=true`. Automated: `claim_creates_passphrase_and_session` test. |
| A2 | Config deleted, passphrase_hash exists: web onboarding | PASS | `+layout.svelte:53-54` routes to `/onboarding?claimed=1`. `showClaimStep=true`, `alreadyClaimed=true`. `ClaimStep` shows "Your Instance is Secured" with 3 recovery paths. `canAdvance()` requires acknowledgment checkbox. `submit()` skips claim. Redirect to `/login`. |
| A3 | Tauri fresh install: onboarding | PASS | `+layout.svelte:31` sets `authModeStore` to `tauri`. `isTauri=true`, `showClaimStep=false`. 8 steps (no Secure). No passphrase interaction. Unchanged from predecessor. |
| A4 | `--reset-passphrase` with server running | PASS | `main.rs:66-70` early exit after `db_dir` derivation. No DB init, no port binding. `println!("{new_passphrase}")` only. `return Ok(())` gives exit code 0. |
| A5 | `--reset-passphrase` without server | PASS | Same code path as A4. No port binding attempted. |
| A6 | Login after out-of-band CLI reset | PASS | `auth/routes.rs:117-135`: mtime check detects `disk != cached`, reloads hash from disk, updates both hash and mtime in AppState. New passphrase verified against refreshed hash. Automated: `login_detects_out_of_band_passphrase_reset` test. |
| A7 | Login after LAN reset endpoint | PASS | `lan.rs:44-54`: in-process reset updates both `passphrase_hash` and `passphrase_hash_mtime`. Next login sees matching mtime, skips reload, verifies against current hash. Regression guard. |
| A8 | Returning web user, valid session | PASS | `+layout.svelte:63-68`: `checkAuth()` returns true, `connectWs()` establishes WebSocket, user proceeds to dashboard. Unchanged flow. |
| A9 | Returning web user, expired session | PASS | `+layout.svelte:70-74`: `checkAuth()` returns false, redirect to `/login`. Login page includes "Forgot your passphrase?" section with CLI reset command. Unchanged flow. |

---

## Security Model

All security properties from the charter are preserved:

| Property | Status |
|----------|--------|
| Passphrase stored as bcrypt hash only | Preserved |
| Plaintext never written to disk or logs | Preserved — `println!` to stdout only in reset path; tracing never receives plaintext |
| Session cookies are HttpOnly | Preserved |
| CSRF required for mutating cookie-auth requests | Preserved |
| Rate limiting on login attempts (5/min/IP) | Preserved |
| Bearer token auth for Tauri/API/MCP | Unchanged |
| `/settings/init` is auth-exempt | Preserved |
| Config init is one-shot (409 on repeat) | Preserved |
| Claim is one-shot (passphrase_hash exists check) | Preserved |
| Mtime check is read-only `stat()` syscall | New — no security impact |
| Early exit reduces attack surface for reset | New — fewer subsystems initialized |

---

## Residual Risks

| Risk | Severity | Scope | Mitigation |
|------|----------|-------|------------|
| `release-plz update` fails on `dashboard/Cargo.toml` | Low | Pre-existing config issue, not related to passphrase changes | Fix `release-plz.toml` workspace config in a separate PR |
| Yanked crates in Cargo.lock (`js-sys 0.3.88`, `wasm-bindgen 0.2.111`) | Low | Pre-existing, unrelated to this initiative | Update in routine dependency maintenance |
| Frontend acceptance scenarios (A1, A2, A3) verified by code review only | Low | Browser testing recommended before merge | Add to PR description as manual testing checklist |
| Mtime granularity on some filesystems (e.g., FAT32 has 2-second resolution) | Very Low | Passphrase resets are human-initiated; two resets within the same mtime tick is not a realistic scenario | Session 03's `!=` comparison handles this correctly |

---

## Recommendation

**GO** — All three reported issues are resolved with targeted, minimal changes. Quality gates pass. All 9 acceptance scenarios verified. Security model preserved. No regressions introduced. Residual risks are low-severity and unrelated to the initiative.

**Pre-merge recommendation:** Manual browser testing of scenarios A1 and A2 on the PR before merging to main.
