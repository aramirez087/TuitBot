# Session 02 Handoff — Onboarding and Recovery UX

**Date:** 2026-03-01
**Status:** Complete
**Branch:** fix/uninstall

---

## Completed

- [x] `ClaimStep.svelte` — added `alreadyClaimed` prop with recovery guidance UI
- [x] `onboarding/+page.svelte` — `showClaimStep` now derives as `!isTauri` (always shows Secure step in web mode)
- [x] `onboarding/+page.svelte` — `canAdvance()` case 8 supports acknowledgment-only mode for `alreadyClaimed`
- [x] `onboarding/+page.svelte` — submit redirects `alreadyClaimed` users to `/login` instead of `/content`
- [x] `onboarding/+page.svelte` — passes `alreadyClaimed` prop to `ClaimStep`
- [x] Login page verified — already has adequate recovery copy (no changes needed)
- [x] Layout verified — correctly routes claimed instances to `/onboarding?claimed=1` (no changes needed)
- [x] `npm run check` — 0 errors, 6 pre-existing warnings (unrelated)
- [x] `npm run build` — successful production build
- [x] `cargo fmt --all && cargo fmt --all --check` — passes
- [x] `RUSTFLAGS="-D warnings" cargo test --workspace` — all 12 test suites pass, no failures
- [x] `cargo clippy --workspace -- -D warnings` — passes

## Decisions Made

1. **Reused `ClaimStep.svelte` with a mode prop** (not a new component). The two modes share the same wizard position, styling conventions, and security concern. A prop-based branch keeps component count low.

2. **Reused the `saved` bindable for the acknowledgment checkbox.** In fresh-claim mode, `saved` means "I've saved my passphrase." In `alreadyClaimed` mode, it means "I understand my recovery options." The parent only cares about a boolean gate.

3. **`alreadyClaimed` users redirect to `/login` after submit** (not `/content`). The `alreadyClaimed` path does not create a session (no claim, no `csrf_token`). Redirecting to `/content` would cause the layout gate to bounce to `/login` — a confusing double redirect.

4. **No changes to login page.** Already has "Forgot your passphrase?" section with both CLI reset commands and clear copy. Consistent with the recovery guidance added to `ClaimStep`.

5. **No changes to `+layout.svelte`.** The layout already correctly sends `claimed=1` parameter when `configStatus()` returns `{ configured: false, claimed: true }`.

## Files Modified

| File | Change |
|------|--------|
| `dashboard/src/lib/components/onboarding/ClaimStep.svelte` | Added `alreadyClaimed` prop; template branches between recovery guidance and existing claim UI; added styles for recovery paths, info box |
| `dashboard/src/routes/onboarding/+page.svelte` | `showClaimStep = !isTauri` (removed `&& !alreadyClaimed`); `canAdvance()` case 8 supports acknowledgment-only; redirect to `/login` for `alreadyClaimed`; pass `alreadyClaimed` to ClaimStep |
| `docs/roadmap/passphrase-lifecycle-ux/session-02-handoff.md` | This document |

## Files Verified (No Changes Needed)

| File | Reason |
|------|--------|
| `dashboard/src/routes/login/+page.svelte` | Recovery copy already adequate (lines 91-96) |
| `dashboard/src/routes/+layout.svelte` | Correctly routes `claimed=1` parameter (lines 50-57) |

## What Did NOT Change

- **Backend code** — no modifications to any Rust crate
- **Tauri bearer-token auth** — `isTauri` still skips the Secure step entirely
- **Passphrase generation logic** — existing `ClaimStep` claim flow moved into `{:else}` branch with zero modifications
- **API client, auth store, onboarding store** — untouched
- **All other onboarding step components** — untouched
- **Layout routing logic** — untouched

## Manual Verification Steps

These acceptance scenarios require manual verification (not covered by automated checks):

### A1 — Fresh web install: onboarding with claim
1. Delete `passphrase_hash` and `config.toml` from data dir
2. Open browser to `localhost:5173`
3. Verify 9-step wizard with "Secure" as step 9
4. Complete all steps — verify passphrase is generated and visible
5. Check "I've saved my passphrase" and submit
6. Verify redirect to `/content?compose=true` with active session

### A2 — Config deleted, passphrase_hash exists: web onboarding
1. Delete only `config.toml` (keep `passphrase_hash`)
2. Open browser to `localhost:5173`
3. Verify redirect to `/onboarding?claimed=1`
4. Verify 9-step wizard with "Secure" as step 9
5. At step 9: verify title is "Your Instance is Secured"
6. Verify recovery paths shown (CLI commands, Settings path)
7. Verify NO passphrase generation UI (no code block, no copy button, no regenerate)
8. Check acknowledgment checkbox and submit
9. Verify redirect to `/login` (NOT `/content`)

### A3 — Tauri fresh install: onboarding
1. Run via `npm run tauri dev`
2. Verify 8-step wizard (no "Secure" step)
3. Complete onboarding — verify no passphrase interaction

## Open Issues

None blocking Session 03.

## Inputs for Session 03

**Read first:**
- `docs/roadmap/passphrase-lifecycle-ux/charter.md` — Design Decisions B and C

**Scope:** Backend changes only (reset command fast path, mtime-based login refresh).

**Files to modify:**

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/main.rs` | Early exit for `--reset-passphrase` after `db_dir` derivation, before DB init. Move "starting tuitbot server" log below early exit. |
| `crates/tuitbot-server/src/state.rs` | Add `passphrase_hash_mtime: RwLock<Option<SystemTime>>` field |
| `crates/tuitbot-server/src/auth/routes.rs` | Mtime check before passphrase verify in login handler |
| `crates/tuitbot-core/src/auth/passphrase.rs` | Add `passphrase_hash_mtime()` helper |
| `docs/lan-mode.md` | Update reset docs to reflect clean output |

**Quality gates:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Key constraints:**
- `--reset-passphrase` must print ONLY the new passphrase on stdout and exit with code 0
- Must work whether or not port 3001 is occupied
- Login after out-of-band CLI reset must work immediately (no server restart)
- Use `>=` mtime comparison to handle filesystem granularity
- Do not modify frontend code
