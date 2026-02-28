# Session 03 Handoff — Frontend First-Run UX

**Date:** 2026-02-28
**Branch:** `feat/init_improvements`

---

## What Changed

This session implemented the frontend claim, onboarding, login, and LAN UX so fresh installs avoid the passphrase dead end while returning users keep secure access.

### Files Modified

| File | Change |
|------|--------|
| `dashboard/src/lib/api.ts` | Added `claimed: boolean` to `ConfigStatus`; rewrote `settings.init()` as custom fetch with `credentials: 'include'` and `csrf_token` return type |
| `dashboard/src/lib/stores/auth.ts` | Added `claimSession(csrfToken)` function for session bootstrap after claim |
| `dashboard/src/routes/+layout.svelte` | Reordered gate: web users now check config/claimed status before auth; fresh installs go to onboarding (not login) |
| `dashboard/src/routes/onboarding/+page.svelte` | Dynamic STEPS array (8 in bearer, 9 in web); claim passphrase state; `submit()` includes claim payload and establishes session; `beforeunload` guard |
| `dashboard/src/routes/login/+page.svelte` | Updated subtitle copy; added "forgot passphrase?" section; improved error messages (context-aware); added `focus-visible` styles; added ARIA attributes |
| `dashboard/src/routes/(app)/settings/LanAccessSection.svelte` | Added two-click reset confirmation; ARIA labels on buttons; `focus-visible` styles; error display for failed reset; updated helper copy |
| `docs/lan-mode.md` | Updated flow description for new claim-based first run; updated troubleshooting guidance |

### Files Created

| File | Purpose |
|------|---------|
| `dashboard/src/lib/wordlist.ts` | EFF short wordlist (1,296 words) + `generatePassphrase()` using `crypto.getRandomValues()` |
| `dashboard/src/lib/components/onboarding/ClaimStep.svelte` | Passphrase creation UI: generated phrase display, regenerate, copy, custom input, save confirmation checkbox |
| `docs/roadmap/fresh-install-auth-ux/session-03-handoff.md` | This file |

---

## Design Decisions Made

### D1: EFF wordlist delivery — inline TypeScript array

The 1,296-word EFF short wordlist ships as a `string[]` constant in `dashboard/src/lib/wordlist.ts`. At ~11KB raw / ~4KB gzipped, it's small enough to inline. No async fetch required — instant generation on page load. The wordlist was copied directly from `crates/tuitbot-core/assets/eff_short_wordlist.txt` to ensure consistency.

### D2: Claim step position — step 9 ("Secure"), after Review (step 8)

The claim step is appended as the final step in web mode. Users see their full config review before being asked to create a passphrase. In Tauri/bearer mode, the step is omitted entirely — the STEPS array is computed reactively based on `authModeStore`.

### D3: Conditional claim step based on auth mode

`isTauri` is derived from `$authModeStore === 'tauri'`. The layout gate sets `authModeStore` to `'tauri'` before redirecting to onboarding, so the step count is correct by the time onboarding renders. Bearer mode gets 8 steps (no Secure); web mode gets 9 steps.

### D4: CSRF token storage after claim mirrors login exactly

`claimSession(csrfToken)` calls `setCsrfToken()`, `setAuthMode('cookie')`, `authMode.set('web')`, and `isAuthenticated.set(true)` — the same sequence as `login()`. This ensures authenticated requests work immediately after claim.

### D5: `beforeunload` guard scoped to claim step

A `$effect` attaches a `beforeunload` listener when the claim step is active and a passphrase has been generated. It's automatically cleaned up by the effect teardown when the user leaves the step (e.g., after successful submit or navigating back).

### D6: `settings.init()` rewritten as custom fetch

The `request()` helper only sets `credentials: 'include'` when `authMode === 'cookie'`. At init time, auth mode is still `'bearer'` (default). To ensure the browser stores the `Set-Cookie` header from the claim response, `init()` was rewritten as a direct `fetch()` call with `credentials: 'include'` always set.

### D7: Layout gate reordering — config check before auth for web users

The `+layout.svelte` `onMount` now follows this flow for non-bearer users:
1. `GET /api/settings/status` → `{ configured, claimed }`
2. If `!configured` → redirect to `/onboarding` (skip login entirely)
3. If `configured` → `checkAuth()` → session valid → dashboard; no session → `/login`

The bearer path is unchanged: token → auth mode → connectWs → config check → onboarding or app.

### D8: Two-click reset confirmation in LanAccessSection

The reset button now requires two clicks within 3 seconds. First click shows "Confirm Reset" with danger styling; second click executes the reset. Reverts to "Reset Passphrase" after 3 seconds if not confirmed.

### D9: Context-aware login error messages

Login errors are now parsed: "invalid"/"unauthorized" messages → "Incorrect passphrase. Check your spelling and try again."; "fetch"/"network" messages → "Cannot reach the server. Is it running?"; other errors pass through unchanged.

---

## CI Results

All checks pass:

```
cargo fmt --all --check         ✅
RUSTFLAGS="-D warnings" cargo test --workspace  ✅ (all tests pass)
cargo clippy --workspace -- -D warnings          ✅
cd dashboard && npm run check                    ✅ (0 errors, 6 pre-existing warnings)
cd dashboard && npm run build                    ✅ (production build succeeds)
```

---

## Flow Verification (Mental Trace)

### Scenario 1: Fresh install, web mode
Server starts on localhost → no passphrase_hash, no config.toml → browser opens → `+layout.svelte` → no token → web path → `configStatus()` returns `{ configured: false }` → redirect to `/onboarding` → user completes steps 1–9 (including Secure) → `submit()` sends `POST /api/settings/init` with `claim: { passphrase: "..." }` → response: `{ status: "created", csrf_token: "..." }` + Set-Cookie → `claimSession()` + `connectWs()` → `goto('/content?compose=true')`.

### Scenario 2: Fresh install, Tauri mode
Bearer token available → bearer path → `configStatus()` → `{ configured: false }` → redirect to `/onboarding` → 8 steps (no Secure) → `submit()` without claim → `goto('/content?compose=true')`.

### Scenario 3: Returning web user, session expired
`configStatus()` → `{ configured: true }` → `checkAuth()` → no session → redirect to `/login` → user enters passphrase → login page shows "forgot passphrase?" section → success → dashboard.

### Scenario 4: Returning web user, valid session
`configStatus()` → `{ configured: true }` → `checkAuth()` → valid → `connectWs()` → dashboard.

---

## Open Issues

### Deferred to Session 04

| Issue | Notes |
|-------|-------|
| End-to-end manual testing | All four scenarios need manual verification against a running server |
| Passphrase strength indicator | Could add a visual strength bar — deferred as low priority |
| Custom passphrase "show/hide" toggle | The custom input uses `type="text"` for visibility; could add a toggle — cosmetic |
| Onboarding progress bar crowding with 9 steps | At narrow viewports, 9 step labels may overflow; may need responsive adjustment |
| Edge case: config deleted but passphrase exists | Layout redirects to onboarding; claim step will be present but backend returns 409 on claim if already claimed. Should handle gracefully. |

---

## Exact Inputs for Session 04

### Read First
- `docs/roadmap/fresh-install-auth-ux/charter.md` — full charter
- `docs/roadmap/fresh-install-auth-ux/backend-contract.md` — API contract
- `docs/roadmap/fresh-install-auth-ux/session-03-handoff.md` — this file

### Manual Test Matrix

| Scenario | Steps | Expected |
|----------|-------|----------|
| Fresh web install | Delete `~/.tuitbot/`, start server, open browser | → onboarding, not login; claim step shows passphrase |
| Complete onboarding with claim | Fill all steps, create passphrase, submit | → dashboard, authenticated, no login screen |
| Returning web user (expired) | Open incognito on configured instance | → login page with "forgot passphrase?" section |
| Returning web user (valid) | Open browser with existing session | → dashboard directly |
| Tauri fresh install | Start Tauri app on fresh install | → onboarding (8 steps, no Secure step) |
| Tauri configured | Start Tauri app on configured instance | → dashboard |
| Reset passphrase | Settings → LAN Access → Reset | Two-click confirmation → new passphrase shown |

### Files to Verify/Modify

1. Fix any issues found during manual testing
2. Update `docs/lan-mode.md` if manual testing reveals documentation gaps
3. Create `docs/roadmap/fresh-install-auth-ux/release-readiness.md` — go/no-go report
4. Create `docs/roadmap/fresh-install-auth-ux/session-04-handoff.md` — final handoff
