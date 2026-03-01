# Release Readiness Report — Fresh-Install Auth UX

**Date:** 2026-02-28
**Branch:** `feat/init_improvements`
**Epic:** fresh-install-auth-ux (Sessions 01–04)

---

## Status: GO

All quality gates pass, all charter requirements are implemented, the security model is preserved, and no P0 blockers remain. The feature is ready to merge.

---

## Scope of Change

The fresh-install auth UX epic resolves the **auth-before-onboarding inversion** in web/LAN mode. Previously, a fresh install required a passphrase (shown once in terminal output) before the user could reach the onboarding wizard — a dead end for users who missed it.

### What changed

| Session | Scope |
|---------|-------|
| 01 | Charter: problem analysis, target UX design, 6 architectural decisions |
| 02 | Backend: `POST /settings/init` claim extension, `is_claimed()`/`create_passphrase_hash()`, conditional startup passphrase, 8 integration tests |
| 03 | Frontend: layout gate reordering, onboarding claim step, EFF wordlist, login UX improvements, LAN reset confirmation |
| 04 | Validation: charter compliance verification, Gap 3.1 fix (claimed-but-unconfigured edge case), documentation reconciliation, this report |

### Files modified (across Sessions 02–04)

**Rust (backend):**
- `crates/tuitbot-core/src/auth/passphrase.rs` — `is_claimed()`, `create_passphrase_hash()`
- `crates/tuitbot-core/src/auth/error.rs` — `AuthError::AlreadyClaimed` variant
- `crates/tuitbot-server/src/routes/settings.rs` — claim handling in `init_settings`, `claimed` field in `config_status`
- `crates/tuitbot-server/src/main.rs` — conditional startup passphrase (`--host 0.0.0.0` vs localhost)

**Frontend (dashboard):**
- `dashboard/src/routes/+layout.svelte` — gate reordering (config check before auth for web), claimed param propagation
- `dashboard/src/routes/onboarding/+page.svelte` — dynamic step count, claim step, 409 handling
- `dashboard/src/routes/login/+page.svelte` — context-aware errors, "forgot passphrase?" section
- `dashboard/src/routes/(app)/settings/LanAccessSection.svelte` — two-click reset confirmation
- `dashboard/src/lib/api.ts` — `claimed` field in `ConfigStatus`, custom `init()` fetch
- `dashboard/src/lib/stores/auth.ts` — `claimSession()` function
- `dashboard/src/lib/wordlist.ts` — EFF short wordlist (1,296 words)
- `dashboard/src/lib/components/onboarding/ClaimStep.svelte` — passphrase creation UI

**Documentation:**
- `docs/lan-mode.md` — updated for claim flow and first-time browser setup
- `docs/roadmap/fresh-install-auth-ux/charter.md`
- `docs/roadmap/fresh-install-auth-ux/backend-contract.md`
- `docs/roadmap/fresh-install-auth-ux/session-{01..04}-handoff.md`
- `docs/roadmap/fresh-install-auth-ux/release-readiness.md` (this file)

---

## Verification Results

### Quality Gates

| Gate | Result |
|------|--------|
| `cargo fmt --all --check` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | Pass (all tests, including 8 claim-specific integration tests) |
| `cargo clippy --workspace -- -D warnings` | Pass |
| `npm run check` (svelte-check) | Pass (0 errors, 6 pre-existing warnings) |
| `npm run build` (production build) | Pass |

### Charter Compliance

All 6 design decisions verified in code:

| # | Decision | Status |
|---|----------|--------|
| 1 | Instance claiming via `POST /settings/init` | Implemented — claim extraction, bcrypt hash, session creation, 409 on re-claim |
| 2 | Frontend gate reordering | Implemented — `configStatus()` before `checkAuth()` for web users |
| 3 | Session bootstrap at claim time | Implemented — `claimSession()` mirrors `login()` exactly |
| 4 | Client-side passphrase generation | Implemented — EFF wordlist in `wordlist.ts`, `ClaimStep.svelte` |
| 5 | Deferred passphrase for Tauri | Implemented — `showClaimStep` derived excludes Tauri and already-claimed |
| 6 | Conditional startup passphrase | Implemented — `main.rs` branches on `cli.host` |

### Target UX Flows Verified

| Flow | Charter Ref | Status |
|------|-------------|--------|
| Fresh install, web mode | Lines 103–116 | Verified — layout → configStatus → onboarding → claim → dashboard |
| Returning user, valid session | Lines 118–126 | Verified — layout → configStatus → checkAuth → dashboard |
| Returning user, expired session | Lines 128–137 | Verified — layout → configStatus → checkAuth fails → login |
| Tauri user (unchanged) | Lines 139–145 | Verified — bearer → config check → onboarding or dashboard |

### Security Model Preserved

| Property | Charter Promise | Implementation |
|----------|----------------|----------------|
| Hash only on disk | Yes | `create_passphrase_hash` writes bcrypt hash |
| Plaintext never on disk | Yes | Client-side generation; only hash stored server-side |
| HttpOnly cookies | Yes | `HttpOnly; SameSite=Strict` in cookie string |
| CSRF for mutating cookie-auth | Yes | Middleware checks `X-CSRF-Token` header |
| Rate limiting on login | Yes | `login_attempts` in AppState |
| Bearer token unchanged | Yes | Tauri path identical pre/post |
| `/settings/init` auth-exempt | Yes | In `AUTH_EXEMPT_PATHS` |
| One-shot claim (409) | Yes | `is_claimed()` check + 409 response |

### Test Coverage

- 8 integration tests for claim flow (Session 02): `claim_creates_passphrase_and_session`, `claim_rejects_short_passphrase`, `claim_rejects_already_claimed`, `init_with_claim_produces_valid_session`, `double_init_returns_409`, `config_status_includes_claimed_false`, `config_status_includes_claimed_true`, `init_without_claim_works_as_before`
- Unit tests for passphrase module: `create_passphrase_hash`, `is_claimed`
- Frontend type checking passes (svelte-check)

---

## Issues Found and Resolved

### Fixed in Session 04

**Gap 3.1: Config-deleted-but-passphrase-exists edge case**

When `config.toml` is deleted but `passphrase_hash` still exists (`configured: false, claimed: true`), the layout previously redirected to onboarding with the claim step. The backend would reject the claim with 409 and never create the config.

**Fix:**
1. Layout passes `?claimed=1` query param to onboarding when `status.claimed` is true.
2. Onboarding reads the param and skips the "Secure" step when already claimed (8 steps instead of 9).
3. Submit function handles 409 "already claimed" from a race condition by retrying without the claim, then redirecting to login.

---

## Known Issues (Non-Blocking)

| Issue | Impact | Mitigation |
|-------|--------|------------|
| Progress bar crowding at narrow viewports (9 steps) | Cosmetic — labels may truncate at <375px width | Step labels are short (max 8 chars); real devices 375px+ handle it |
| Manual `/onboarding` navigation on configured instance | User sees wizard again; submit fails with 409 | 409 prevents any data corruption; cosmetic only |
| Two-browser simultaneous claim race condition | One browser's claim succeeds, the other gets 409 | 409 is handled gracefully — retry without claim + redirect to login |

---

## Residual Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| First-claim race condition (simultaneous browsers) | Very low | Low | Atomic file creation; first writer wins; loser gets 409 handled gracefully |
| User misses passphrase during claim step | Low | Medium | `beforeunload` guard, "I've saved this" checkbox, copy-to-clipboard button |
| Passphrase leaked via browser dev tools / memory | Very low | Low | Same risk as any password input; bcrypt hash on server; session cookie is HttpOnly |

---

## Recommended Follow-Up Work

| Item | Priority | Effort |
|------|----------|--------|
| E2E automated tests (Playwright) for all 4 flows | Medium | 1 session |
| Responsive progress bar for 9-step wizard | Low | Small |
| Passphrase strength indicator (visual bar) | Low | Small |
| Custom passphrase show/hide toggle | Low | Trivial |
| Rate limit the claim endpoint (DoS mitigation) | Low | Small — currently one-shot by design |

---

## Conclusion

The fresh-install auth UX resolves the original problem: web/LAN users on a fresh install no longer encounter a passphrase dead end. The implementation follows the charter's 6 design decisions, preserves all security properties, passes all CI gates, and handles edge cases gracefully. **Ship it.**
