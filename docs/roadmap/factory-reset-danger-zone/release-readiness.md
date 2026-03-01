# Factory Reset -- Release Readiness Report

**Date:** 2026-02-28
**Session:** 4 (Validation)
**Decision:** GO

---

## Quality Gate Results

All four quality gates pass with zero failures:

| Gate | Result |
|------|--------|
| `cargo fmt --all && cargo fmt --all --check` | Clean |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | All passed (0 failed, 11 ignored -- all ignored tests are pre-existing live/API tests) |
| `cargo clippy --workspace -- -D warnings` | Clean |
| `cd dashboard && npm run check` | 0 errors (6 pre-existing warnings in unrelated files) |
| `cd dashboard && npm run build` | Success (static adapter wrote to `build/`) |

### Test Coverage

Factory-reset-specific tests:

| Suite | Tests | Status |
|-------|-------|--------|
| `tuitbot-core` unit (`storage/reset.rs`) | 5 | All pass |
| `tuitbot-server` integration (`factory_reset.rs`) | 7 | All pass |
| Dashboard build (includes `DangerZoneSection`) | 1 | Pass |

---

## Critical Path Verification

Traced the full user flow through code without running the app:

### 1. Configured instance -> Danger Zone

- `+layout.svelte` (line 36-38): Bearer mode checks `configStatus()`,
  redirects to `/onboarding` if `configured=false`. Configured instances
  proceed to settings.
- `+page.svelte` (line 55): `{ id: 'danger', label: 'Danger', icon: AlertTriangle }`
  in sections array -- nav entry present.
- `+page.svelte` (line 191): `<DangerZoneSection />` rendered after
  `<LanAccessSection />`.
- **Verified.**

### 2. Confirmed reset

- `DangerZoneSection.svelte` (line 16): `canReset` derived from exact
  match `confirmationText === CONFIRMATION_PHRASE && !resetting`.
- Button disabled until `canReset` is true (line 89).
- `handleReset()` (line 18-35): calls `api.settings.factoryReset()`.
- `api.ts` (line 916-921): sends `POST /api/settings/factory-reset` with
  `{ confirmation }` body.
- Server handler (`settings.rs` line 441-536): validates phrase, stops
  runtimes, clears DB (30 tables in FK-safe order in single tx), deletes
  config/passphrase/media files, clears in-memory state, returns response
  with `Set-Cookie: Max-Age=0`.
- **Verified.**

### 3. Reset -> Onboarding redirect

- On success: `clearSession()` -> `resetStores()` -> `disconnectWs()` ->
  `goto('/onboarding')` with `window.location.href` fallback.
- `clearSession()` (auth.ts line 39-43): clears CSRF token, sets
  `authMode` to `'none'`, sets `isAuthenticated` to `false`.
- `resetStores()` (settings.ts line 185-194): nulls config/defaults/draft,
  resets loading to true, clears errors.
- **Verified.**

### 4. Fresh init path after reset

- `+layout.svelte` (line 36-38/50-58): checks `configStatus()`. After
  reset, `configured=false` and `claimed=false`.
  - Bearer mode: redirects to `/onboarding`.
  - Cookie mode: redirects to `/onboarding`.
- `POST /api/settings/init` is auth-exempt. Creates new config.toml.
- Integration test `factory_reset_allows_re_onboarding` confirms this
  (factory_reset.rs line 334-355).
- **Verified.**

---

## Data Cleanup Verification

Traced what is cleared vs preserved against the charter:

| Charter Requirement | Implementation | Verified |
|---------------------|----------------|----------|
| All 30 user tables cleared | `TABLES_TO_CLEAR` has 30 entries; `tables_to_clear_covers_all_user_tables` test validates against `sqlite_master` | Yes |
| `_sqlx_migrations` preserved | Not in `TABLES_TO_CLEAR`; dedicated test `factory_reset_preserves_migrations` | Yes |
| `config.toml` deleted | Handler line 471-478, `remove_file` with NotFound tolerance | Yes |
| `passphrase_hash` file deleted | Handler line 481-489, `remove_file` with NotFound tolerance | Yes |
| `passphrase_hash` in-memory cleared | Handler line 503, `*state.passphrase_hash.write().await = None` | Yes |
| `media/` directory deleted | Handler line 492-500, `remove_dir_all` with NotFound tolerance | Yes |
| Sessions cleared (DB) | `sessions` table is in `TABLES_TO_CLEAR` (position 29) | Yes |
| Session cookie cleared | Handler line 530, `Set-Cookie: Max-Age=0` | Yes |
| Runtimes stopped | Handler line 453-460, `drain()` with `shutdown()` | Yes |
| Watchtower cancelled | Handler line 463-465, `token.cancel()` | Yes |
| Content generators cleared | Handler line 504 | Yes |
| Login attempts cleared | Handler line 505 | Yes |
| `api_token` preserved | Not in any deletion path | Yes |
| DB schema preserved | DELETE not DROP; dedicated test | Yes |
| `backups/` preserved | Not in any deletion path | Yes |
| No content source folders touched | Not in any deletion path | Yes |

---

## Auth Protection Verification

- `/api/settings/factory-reset` is NOT in `AUTH_EXEMPT_PATHS`
  (middleware.rs lines 36-49).
- Integration test `factory_reset_requires_auth` sends POST without auth
  and asserts 401.
- Bearer and cookie+CSRF paths both covered by standard middleware.
- **Verified: destructive action is behind auth and CSRF.**

---

## Artifact Reconciliation

| Artifact | Divergence | Resolution |
|----------|-----------|------------|
| `charter.md` | Step 7 (VACUUM) listed but code skips it (decided in Session 2) | Annotated as dropped with cross-reference to session-02-handoff |
| `reset-contract.md` | None | Endpoint, request body, response shape, error codes, post-reset behavior all match code exactly |
| `frontend-flow.md` | None | User journey, confirmation UX, auth mode handling, state cleanup sequence, known limitations all match DangerZoneSection.svelte |
| `session-01-handoff.md` | None | Charter and scope definitions match |
| `session-02-handoff.md` | None | Backend implementation matches |
| `session-03-handoff.md` | None | Frontend implementation matches |

---

## Residual Risks and Known Limitations

### Low Risk

1. **Multi-tab stale state**: Other open tabs are not notified of the
   reset. They will see stale data until refresh. The layout boot check
   redirects to `/onboarding` on next navigation. Mitigation: document
   as known limitation; `BroadcastChannel` is a future enhancement.

2. **Server subsystem hot-restart**: After re-onboarding, the user must
   manually start the runtime. Consistent with initial onboarding flow.

3. **Other frontend stores not reset**: Stores outside of `settings`
   (analytics, approval, activity) are not explicitly cleared. They fail
   gracefully on the next API call and are inaccessible because the user
   is redirected to onboarding.

4. **Concurrent requests during reset**: A request arriving between
   DB clearing and response could see partially-cleared state. Mitigated
   by runtime stop-first ordering and DB transaction atomicity.

### No Risk

- **Table coverage regression**: The `tables_to_clear_covers_all_user_tables`
  test checks at runtime against `sqlite_master`. Any new migration adding
  a table not in `TABLES_TO_CLEAR` will fail this test automatically.

- **Auth bypass**: Route is not in exempt list; integration test enforces.

---

## Release Decision

**GO.**

All quality gates pass. The critical path is verified end-to-end through
code audit. All 12 factory-reset-specific tests pass. All artifacts match
the shipped code (with one charter annotation for the VACUUM omission).
No blockers identified. Residual risks are documented and low-severity.

---

## Follow-Up Items (Post-Release)

These are enhancements, not blockers:

1. **`BroadcastChannel` cross-tab notification**: Notify other open tabs
   when factory reset completes so they redirect to onboarding immediately.

2. **Post-reset transition animation**: Toast or animation before the
   redirect to onboarding for better UX feedback.

3. **Runtime auto-start after re-onboarding**: Server-side change to
   optionally start the runtime after `POST /api/settings/init`.

4. **VACUUM option**: Consider adding an optional VACUUM step (behind a
   query parameter) for users who want to reclaim disk space immediately.
