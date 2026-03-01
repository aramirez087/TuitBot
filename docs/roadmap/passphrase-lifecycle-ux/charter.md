# Passphrase Lifecycle UX Charter

**Status:** Approved
**Date:** 2026-03-01
**Epic:** passphrase-lifecycle-ux
**Predecessor:** fresh-install-auth-ux (merged, GO)
**Sessions:** 4 (charter, onboarding UX, backend fixes, validation)

---

## Problem Statement

The `fresh-install-auth-ux` epic successfully resolved the auth-before-onboarding inversion. Three latent passphrase lifecycle issues remain that surface once users operate within the new flow:

1. **No passphrase after onboarding (claimed-but-unconfigured path):** A user who deletes `config.toml` but still has `passphrase_hash` on disk completes onboarding without ever seeing a passphrase or recovery guidance. If their session expires, they are locked out.

2. **Noisy `--reset-passphrase` output:** Running `tuitbot-server --reset-passphrase` produces full startup output (DB init, API token, config loading, LLM initialization, Watchtower) and then fails with "Address already in use" if the server is already running. The new passphrase is buried in noise.

3. **Stale in-memory auth after out-of-band reset:** After running `--reset-passphrase` as a separate process while the server is running, the new passphrase does not work until the server is restarted. The running server still holds the old bcrypt hash in memory.

These are not regressions from `fresh-install-auth-ux`. They are pre-existing gaps now exposed by the fact that the claim flow works correctly and users actually encounter these edge paths.

---

## Root-Cause Analysis

### Failure 1: No passphrase after onboarding

**Symptom:** User completes the onboarding wizard via the `alreadyClaimed` path and lands in the dashboard authenticated. When their session expires, they cannot log in because they were never shown a passphrase.

**Code trace:**

1. **Layout gate** (`dashboard/src/routes/+layout.svelte:50-56`): When `configStatus()` returns `{ configured: false, claimed: true }`, the layout redirects to `/onboarding?claimed=1`.

2. **Step exclusion** (`dashboard/src/routes/onboarding/+page.svelte:31-33`):
   ```typescript
   let alreadyClaimed = $derived($page.url.searchParams.get('claimed') === '1');
   let showClaimStep = $derived(!isTauri && !alreadyClaimed);
   let steps = $derived(showClaimStep ? [...BASE_STEPS, 'Secure'] : BASE_STEPS);
   ```
   When `alreadyClaimed` is true, `showClaimStep` is false, and the 'Secure' step is excluded. The wizard shows 8 steps instead of 9.

3. **Submit skips claim** (`dashboard/src/routes/onboarding/+page.svelte:152-154`):
   ```typescript
   if (showClaimStep && claimPassphrase.trim()) {
       config.claim = { passphrase: claimPassphrase.trim() };
   }
   ```
   No claim is sent. The user gets a config-only setup with no passphrase interaction.

4. **Result:** After onboarding, the user is redirected to `/content?compose=true`. Their existing session (from a prior claim or from a session cookie established during a previous onboarding) carries them through. But they have no passphrase knowledge. When the session expires, they hit the login page with no way to authenticate.

**Secondary scenario (Tauri to LAN transition):** In Tauri mode, `showClaimStep` is false because `isTauri` is true (line 30). This is intentional per Design Decision 5 of the predecessor charter. However, if the user later enables LAN mode (`--host 0.0.0.0`), they need a passphrase. The LAN settings page allows generating one, but this is not communicated during onboarding.

### Failure 2: Noisy `--reset-passphrase` output

**Symptom:** Running `tuitbot-server --reset-passphrase` produces server startup noise and fails with "Address already in use" if port 3001 is occupied.

**Code trace in `crates/tuitbot-server/src/main.rs`:**

The `--reset-passphrase` flag is handled WITHIN the normal server startup flow, not as an early exit:

```
Line 51-53:  tracing init (before reset)
Line 55:     CLI parse
Line 58-62:  db_dir derivation (needed for reset)
Line 64-69:  tracing::info "starting tuitbot server" (noise)
Line 71:     storage::init_db() — SQLite pool + migration (unnecessary for reset)
Line 74:     auth::ensure_api_token() — file I/O (unnecessary for reset)
Line 78-83:  --reset-passphrase block — passphrase reset + print
Line 84-109: else branches for passphrase handling
Line 112:    broadcast channel creation (unnecessary)
Line 117:    Config::load() (unnecessary)
Line 138-153: LLM provider init (unnecessary)
Line 179-213: Watchtower startup (unnecessary)
Line 215-233: AppState construction (unnecessary)
Line 245:    TcpListener::bind() — FAILS if port in use
```

After the reset prints at line 81, the server continues through 150+ lines of initialization and eventually fails at port binding. The user sees the new passphrase buried between log lines and gets a non-zero exit code.

**The fix is straightforward:** Move the reset check immediately after `db_dir` derivation (after line 62), print the passphrase, and return `Ok(())`.

### Failure 3: Stale in-memory auth after out-of-band reset

**Symptom:** After running `--reset-passphrase` as a separate process while the server is running, the old passphrase still works and the new one is rejected.

**Code trace:**

1. **Hash loaded once at startup** (`crates/tuitbot-server/src/main.rs:78-109`): The passphrase hash is read from disk and stored in `AppState.passphrase_hash: RwLock<Option<String>>` (line 221).

2. **Login checks in-memory hash only** (`crates/tuitbot-server/src/auth/routes.rs:117`):
   ```rust
   let hash = state.passphrase_hash.read().await;
   ```
   The login handler reads from the in-memory `RwLock`, never from disk.

3. **CLI reset writes to disk only** (`crates/tuitbot-core/src/auth/passphrase.rs:163-179`):
   ```rust
   pub fn reset_passphrase(data_dir: &Path) -> Result<String, AuthError> {
       let hash_path = data_dir.join("passphrase_hash");
       let passphrase = generate_passphrase();
       let hash = hash_passphrase(&passphrase)?;
       std::fs::write(&hash_path, &hash).map_err(|e| AuthError::Storage { ... })?;
       Ok(passphrase)
   }
   ```
   No mechanism exists to notify the running server process.

4. **LAN reset works correctly** (`crates/tuitbot-server/src/routes/lan.rs:43-55`): The in-process endpoint calls `passphrase::reset_passphrase()` and then updates `state.passphrase_hash.write().await` with the new hash. This works because it runs within the server process.

**The gap:** Out-of-band (CLI) resets modify the disk file but cannot update the in-memory state of a separate process. There is no filesystem watcher, periodic refresh, or IPC mechanism.

---

## Comparison with `fresh-install-auth-ux` Charter

| fresh-install-auth-ux Decision | Status | Gap in This Epic |
|-------------------------------|--------|------------------|
| 1. Instance claiming via `/settings/init` | Complete | None |
| 2. Frontend gate reordering | Complete | Claimed-but-unconfigured path lacks passphrase visibility |
| 3. Session bootstrap at claim time | Complete | None |
| 4. Client-side passphrase generation | Complete | None |
| 5. Deferred passphrase for Tauri | Complete | No onboarding guidance about future LAN access |
| 6. Conditional startup passphrase | Complete | Reset command still noisy; no live reload |

**What was out of scope for `fresh-install-auth-ux`:**
- The `--reset-passphrase` behavior was not modified (documented as-is in `docs/lan-mode.md`)
- Live reload of passphrase after out-of-band reset was not considered
- The "claimed but unconfigured" edge case was partially handled but the user never sees their passphrase in that path

**Nothing regressed.** These issues are pre-existing and now reachable by the correct flow.

---

## Design Decisions

### Decision A: Show recovery guidance in claimed-but-unconfigured onboarding

**Decision:** When `alreadyClaimed` is true, add a lightweight "Your Instance is Secured" informational step to the onboarding wizard that shows passphrase recovery paths without regenerating the passphrase.

**Rationale:** When the instance is already claimed, we cannot show the existing passphrase (only the bcrypt hash is stored). Regenerating the passphrase in the browser would bypass the one-shot claim security model. Instead, we show clear guidance: "Your instance is already secured. After setup, log in with your passphrase, or reset it via CLI or Settings."

**Alternative rejected:** Allow re-claiming when `alreadyClaimed` is true. This would weaken the security model by allowing any browser to overwrite an existing passphrase.

**Implementation:**
- `ClaimStep.svelte` gets an `alreadyClaimed` mode prop showing recovery info instead of passphrase generation
- `onboarding/+page.svelte` always includes the 'Secure' step in web mode (remove the `!alreadyClaimed` exclusion)
- The step's advance condition in `alreadyClaimed` mode is just an acknowledgment checkbox

### Decision B: Make `--reset-passphrase` a fast-path maintenance command

**Decision:** The `--reset-passphrase` flag triggers passphrase reset and exits BEFORE DB init, API token creation, config loading, LLM setup, Watchtower startup, and port binding.

**Rationale:** The reset operation only needs `data_dir` (to locate the `passphrase_hash` file). It does not need a database, API token, config, LLM provider, watchtower, or network binding. Moving it before all of these eliminates noise and prevents the "Address already in use" error.

**Implementation in `crates/tuitbot-server/src/main.rs`:** After `Cli::parse()` and `db_dir` derivation (after line 62), add:
```rust
if cli.reset_passphrase {
    let new_passphrase = passphrase::reset_passphrase(db_dir)?;
    println!("{new_passphrase}");
    return Ok(());
}
```

This is 4 lines. The rest of `main.rs` is unchanged. The tracing init (lines 51-53) and `tracing::info!("starting tuitbot server")` (lines 64-69) should remain above the early exit or be moved below it to keep the output completely clean. Decision: move the "starting tuitbot server" log line below the early exit, keep tracing init above (needed for error formatting).

**Output contract:** The command prints ONLY the new passphrase on stdout and exits with code 0. No tracing, no banners, no formatting. This makes it scriptable (`NEW_PASS=$(tuitbot-server --reset-passphrase)`).

### Decision C: Mtime-based passphrase hash refresh on login

**Decision:** The login handler re-reads the `passphrase_hash` file from disk when the file's modification time is newer than the last load time.

**Rationale:** Login attempts are infrequent (rate-limited to 5/minute per IP). A single `stat()` syscall on each login attempt adds negligible overhead and catches out-of-band resets immediately. This avoids the complexity of a filesystem watcher for a single-file check on an infrequent code path.

**Alternatives considered and rejected:**
- **Filesystem watcher (`notify` crate):** Adds a dependency, a background task, and complexity for a problem that only manifests on login. Overengineered.
- **Signal-based reload (`SIGHUP`):** Not portable to Windows or Tauri.
- **Periodic timer:** Adds a background task. Login frequency is already low enough that check-on-access is simpler.

**Implementation:**
- `crates/tuitbot-server/src/state.rs`: Add `passphrase_hash_mtime: RwLock<Option<std::time::SystemTime>>` field
- `crates/tuitbot-core/src/auth/passphrase.rs`: Add `pub fn passphrase_hash_mtime(data_dir: &Path) -> Option<SystemTime>` helper
- `crates/tuitbot-server/src/auth/routes.rs`: Before verifying passphrase in the login handler:
  1. `stat()` the `data_dir/passphrase_hash` file to get mtime
  2. Compare with `state.passphrase_hash_mtime`
  3. If newer (or first check), re-read the file and update both the hash and mtime in AppState
  4. Proceed with verification against the (possibly refreshed) hash
- Use `>=` comparison (not `>`) to handle filesystem mtime granularity

---

## Implementation Sessions

### Session 02: Onboarding and Recovery UX (Frontend)

**Goal:** Ensure the `alreadyClaimed` onboarding path shows passphrase recovery guidance.

**Files to modify:**

| File | Change |
|------|--------|
| `dashboard/src/routes/onboarding/+page.svelte` | Always include 'Secure' step in web mode; pass `alreadyClaimed` prop to ClaimStep |
| `dashboard/src/lib/components/onboarding/ClaimStep.svelte` | Add `alreadyClaimed` mode showing recovery info instead of passphrase generation |
| `dashboard/src/routes/login/+page.svelte` | Verify copy consistency (likely no-op; already has "Forgot your passphrase?" section) |

**Acceptance criteria:**
1. Config deleted + passphrase_hash exists + web mode: onboarding shows "Your Instance is Secured" step with recovery guidance.
2. Fresh web install (no passphrase_hash): onboarding shows the existing "Secure Your Instance" claim step unchanged.
3. Tauri mode: onboarding skips the Secure step unchanged.
4. `npm run check` passes.
5. `npm run build` passes.

**Risks:**

| Risk | Impact | Mitigation |
|------|--------|------------|
| ClaimStep changes break the working claim flow | High | Test both `alreadyClaimed=false` (generation) and `alreadyClaimed=true` (info) modes |
| Svelte 5 rune issues with new prop | Medium | Run `npm run check` as quality gate |

### Session 03: Reset Command and Live Reload (Backend)

**Goal:** Make `--reset-passphrase` a fast-path exit and enable mtime-based passphrase hash refresh on login.

**Files to modify:**

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/main.rs` | Move `--reset-passphrase` to early fast path after `db_dir` derivation, before DB init |
| `crates/tuitbot-server/src/auth/routes.rs` | Add mtime check before passphrase verification in login handler |
| `crates/tuitbot-server/src/state.rs` | Add `passphrase_hash_mtime: RwLock<Option<SystemTime>>` field |
| `crates/tuitbot-core/src/auth/passphrase.rs` | Add `passphrase_hash_mtime(data_dir) -> Option<SystemTime>` helper |
| `docs/lan-mode.md` | Update reset command documentation to reflect clean output |

**Acceptance criteria:**
1. `tuitbot-server --reset-passphrase` prints ONLY the new passphrase and exits with code 0.
2. `tuitbot-server --reset-passphrase` works whether or not port 3001 is occupied.
3. Login after out-of-band CLI reset works immediately (no server restart needed).
4. Login after in-process LAN reset works as before (regression guard).
5. `cargo fmt --all && cargo fmt --all --check` passes.
6. `RUSTFLAGS="-D warnings" cargo test --workspace` passes.
7. `cargo clippy --workspace -- -D warnings` passes.

**Risks:**

| Risk | Impact | Mitigation |
|------|--------|------------|
| Early exit for `--reset-passphrase` changes error behavior | Low | Test that reset returns exit code 0 on success |
| Mtime granularity issues on some filesystems | Low | Use `>=` comparison; always reload on first login |
| Parallel login + file read race condition | Very Low | `RwLock` ensures atomic read/write; worst case is one stale verification |

### Session 04: Release Validation

**Goal:** End-to-end verification of all three fixes, full CI gates, release-readiness report.

**Files to create:**

| File | Purpose |
|------|---------|
| `docs/roadmap/passphrase-lifecycle-ux/release-readiness.md` | Go/no-go report |
| `docs/roadmap/passphrase-lifecycle-ux/session-04-handoff.md` | Final handoff |

**Acceptance criteria:**
1. All acceptance scenarios (below) pass manual verification.
2. Full CI checklist passes: `cargo fmt`, `cargo test`, `cargo clippy`, `npm run check`, `npm run build`.
3. Release-readiness report is GO or lists specific blockers.

---

## Acceptance Scenarios

| # | Scenario | Expected Behavior |
|---|----------|-------------------|
| A1 | Fresh web install: onboarding with claim | User sees passphrase in Secure step, copies it, completes onboarding, lands in dashboard authenticated |
| A2 | Config deleted, passphrase_hash exists: web onboarding | User sees "Your Instance is Secured" guidance with recovery paths, completes config, redirected to login |
| A3 | Tauri fresh install: onboarding | No claim step, no passphrase shown (unchanged) |
| A4 | `--reset-passphrase` while server IS running | Clean output: new passphrase printed to stdout, exit code 0, no server startup noise, no "Address already in use" |
| A5 | `--reset-passphrase` while server is NOT running | Same clean output as A4 |
| A6 | Login after out-of-band CLI reset (no server restart) | New passphrase works immediately, old passphrase rejected |
| A7 | Login after LAN reset endpoint (in-process) | Works immediately (regression guard, same as today) |
| A8 | Returning web user, valid session | Dashboard directly (unchanged) |
| A9 | Returning web user, expired session | Login page with "Forgot your passphrase?" section showing CLI reset command |

---

## Security Model (Preserved)

| Property | Before | After |
|----------|--------|-------|
| Passphrase stored as bcrypt hash only | Yes | Yes |
| Plaintext never written to disk or logs | Yes | Yes |
| Session cookies are HttpOnly | Yes | Yes |
| CSRF required for mutating cookie-auth requests | Yes | Yes |
| Rate limiting on login attempts | Yes | Yes |
| Bearer token auth for Tauri/API/MCP | Yes | Yes (unchanged) |
| `/settings/init` is auth-exempt | Yes | Yes |
| Config init is one-shot (409 on repeat) | Yes | Yes |
| Claim is one-shot (passphrase_hash exists check) | Yes | Yes |

No security properties change. The mtime-based refresh adds a `stat()` syscall on login, which is a read-only filesystem operation. The early exit for `--reset-passphrase` reduces the attack surface (fewer subsystems initialized for a maintenance command).

---

## Out of Scope

- Multi-user / multi-account auth (Tuitbot is single-user, local-first)
- OAuth/SSO integration
- Email-based passphrase recovery
- Changing the passphrase format (4-word EFF scheme is well-established)
- Modifying Tauri bearer-token behavior
- Adding a browser-based passphrase reset for unauthenticated users (would require email or other verification)
- Filesystem watcher for passphrase changes (mtime check on login is sufficient)
