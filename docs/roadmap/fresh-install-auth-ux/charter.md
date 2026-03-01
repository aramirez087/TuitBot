# Fresh-Install Auth UX Charter

**Status:** Approved
**Date:** 2026-02-28
**Epic:** fresh-install-auth-ux
**Sessions:** 4 (charter → backend → frontend → validation)

---

## Problem Statement

Tuitbot has an **auth-before-onboarding inversion** in web/LAN mode. On a fresh install, the browser frontend demands a passphrase (login screen) before the user can reach the onboarding wizard. The passphrase was only printed once to the server's terminal output at startup. If the user missed it — running as a systemd service, Docker container, or Tauri sidecar on first launch — they are locked out with no self-service recovery path from the browser.

### Current Flow (Web/LAN — broken for fresh installs)

```
Server starts
  → ensure_passphrase() generates passphrase, prints to stdout, writes hash to disk

User opens browser → +layout.svelte runs:
  1. No bearer token (not Tauri)
  2. checkAuth() → no session cookie → redirect to /login
  3. User must enter passphrase from terminal
  4. Only AFTER login → check if configured → redirect to /onboarding
```

Step 2→3 is a dead end for users who never saw the terminal output. The login page says *"Enter the passphrase shown in your server terminal"* but provides no alternative.

### Current Flow (Tauri — works correctly)

```
Tauri reads ~/.tuitbot/api_token → Bearer mode → all requests authenticated
+layout.svelte → config check → not configured → redirect to /onboarding
```

Tauri bypasses the passphrase entirely. This flow is correct and must remain unchanged.

---

## Audit of Current Behavior

### `+layout.svelte` — Frontend Gate

- **Lines 18–46:** Auth check runs first. For non-bearer users with no valid session cookie, redirects to `/login` regardless of whether config exists.
- **Lines 49–57:** Config check runs second. If not configured, redirects to `/onboarding`.
- **Problem:** Login gate fires before onboarding check. A fresh install with no passphrase knowledge is a dead end.

### `login/+page.svelte` — Login UI

- Accepts a 4-word passphrase, calls `login()` store function.
- No "first-time setup" alternative. No passphrase reset from the browser.
- Subtitle hardcodes: *"Enter the passphrase shown in your server terminal."*

### `main.rs` — Server Startup (lines 78–94)

- `ensure_passphrase(db_dir)` runs unconditionally at startup.
- If no hash file exists: generates passphrase, prints to stdout, writes bcrypt hash.
- Passphrase plaintext is shown once and never stored.
- Generation happens before the server knows whether a config exists.

### `settings.rs` — Config Init Endpoint

- **`GET /api/settings/status`** — auth-exempt. Returns `{ configured, deployment_mode, capabilities }`.
- **`POST /api/settings/init`** — auth-exempt. Creates initial `config.toml`. Returns 409 if already configured.
- Neither endpoint interacts with the passphrase system.

### `lan.rs` — LAN Passphrase Management

- **`POST /api/settings/lan/reset-passphrase`** — requires auth. Returns new plaintext passphrase.
- Cannot be used by unauthenticated users to bootstrap.

### `middleware.rs` — Auth Middleware

- Exempt paths: `/health`, `/settings/status`, `/settings/init`, `/ws`, `/auth/login`, `/auth/status`.
- `/settings/init` is exempt — the onboarding wizard can POST config without auth.
- No way to establish a session without already having the passphrase.

### `passphrase.rs` — Core Passphrase Module

- `generate_passphrase()`: 4 random words from EFF short wordlist (1,296 words).
- `hash_passphrase()`: bcrypt cost 12.
- `ensure_passphrase(data_dir)`: Creates hash file if missing, returns `Some(plaintext)` on first run.
- `load_passphrase_hash(data_dir)`: Reads hash from file.
- `reset_passphrase(data_dir)`: Overwrites hash with new one.

### `state.rs` — AppState

- `passphrase_hash: RwLock<Option<String>>` — supports runtime updates.
- `data_dir: PathBuf` — where hash file lives.

### `docs/lan-mode.md`

- Documents passphrase-on-first-start behavior.
- Recommends `--reset-passphrase` for recovery.
- No mention of browser-based first-run claim flow.

---

## Target UX

### Fresh Install — Web/LAN User

```
Server starts → no passphrase_hash exists yet → prints nothing about passphrase

User opens browser → +layout.svelte runs:
  1. No bearer token (not Tauri)
  2. GET /api/settings/status → { configured: false, claimed: false }
  3. Redirect to /onboarding (skip login entirely)
  4. User completes onboarding wizard
  5. Final step: claim passphrase (generated client-side or user-chosen)
  6. Submit: POST /api/settings/init with { ..config, claim: { passphrase } }
  7. Server creates config + passphrase_hash atomically
  8. Response includes Set-Cookie (session) + csrf_token
  9. User is authenticated and configured → redirect to dashboard
```

### Returning User — Web/LAN (has valid session)

```
Browser → +layout.svelte:
  1. No bearer token
  2. checkAuth() → session cookie valid
  3. Config check → configured
  4. Render dashboard
```

### Returning User — Web/LAN (session expired)

```
Browser → +layout.svelte:
  1. No bearer token
  2. checkAuth() → no valid session
  3. GET /api/settings/status → { configured: true, claimed: true }
  4. Redirect to /login
  5. Enter passphrase → authenticated → dashboard
```

### Tauri User (unchanged)

```
Bearer token from api_token file → always authenticated
Config check → configured or not → onboarding or dashboard
No passphrase interaction at any point
```

---

## Design Decisions

### 1. Instance Claiming via `POST /api/settings/init`

**Decision:** Extend `POST /api/settings/init` to accept an optional `claim` object containing the passphrase. If the instance is unclaimed (no `passphrase_hash` file), the endpoint creates both the config and the passphrase hash atomically.

**Rationale:**
- `/settings/init` is already auth-exempt — no new unauthenticated surface area.
- Atomic config+passphrase creation prevents partial states.
- The endpoint already returns 409 if config exists, so claiming is inherently one-shot.

**Security:** The passphrase must be provided by the client. The plaintext never touches disk or logs. Only the bcrypt hash is stored.

### 2. Frontend Gate Reordering

**Decision:** In `+layout.svelte`, check config/claimed status *before* checking auth for web users. If unconfigured AND unclaimed, redirect to `/onboarding`. If configured but unauthenticated, redirect to `/login`.

**Rationale:**
- Fixes the auth-before-onboarding inversion.
- Tauri bearer-token path is unaffected (bearer always authenticates first).
- No security weakening: unconfigured instances have no sensitive data to protect.

### 3. Session Bootstrap at Claim Time

**Decision:** When `/api/settings/init` processes a claim, it also creates a session and returns `Set-Cookie` + CSRF token, identical to the login endpoint's response shape.

**Rationale:**
- User doesn't need to re-enter the passphrase they just set.
- Smooth UX: onboarding → dashboard without an intermediate login screen.
- Same security properties as a regular login session.

### 4. Passphrase Generation in the Frontend

**Decision:** The frontend generates a suggested 4-word passphrase using the same EFF wordlist (bundled as a static asset), with a "regenerate" button. Users can also type their own.

**Rationale:**
- Client-side generation means the plaintext only crosses the wire once (in the claim POST).
- Using the same EFF wordlist maintains consistency with server-generated passphrases.
- User agency: they can choose their own passphrase.

**Alternative rejected:** Server generates passphrase and returns it. This means the plaintext crosses the wire in the response, and we'd need to trust the server not to log it.

### 5. Deferred Passphrase for Tauri

**Decision:** In Tauri/bearer mode, skip the passphrase claim step during onboarding. The passphrase is only needed for web/LAN access. Users can set one later from the LAN settings page.

**Rationale:**
- Tauri users authenticate via bearer token and never need a passphrase.
- Forcing passphrase creation adds friction with no benefit.
- The existing `ensure_passphrase()` at startup can be made conditional.

### 6. Conditional Startup Passphrase Generation

**Decision:** Make `ensure_passphrase()` in `main.rs` conditional:
- If `--host 0.0.0.0` (LAN mode) AND no passphrase_hash exists: generate and print as today (backward compatible for CLI users who expect terminal output).
- If `--host 127.0.0.1` (localhost): skip generation, defer to claim flow.

**Rationale:**
- Preserves the existing behavior for users who explicitly start in LAN mode from the CLI.
- Removes the confusing "save this passphrase" message for localhost-only users.
- The claim flow handles the web/LAN case where the user may not see terminal output.

**Fallback:** If a user starts with `--host 0.0.0.0` and misses the terminal output, they can still claim via the browser if no config exists yet (claim checks for hash file, not just config).

---

## Security Model (Preserved)

| Property | Current | After Change |
|----------|---------|--------------|
| Passphrase stored as bcrypt hash only | Yes | Yes |
| Plaintext never written to disk | Yes | Yes |
| Session cookies are HttpOnly | Yes | Yes |
| CSRF required for mutating cookie-auth requests | Yes | Yes |
| Rate limiting on login attempts | Yes | Yes |
| Bearer token auth for Tauri/API/MCP | Yes | Yes (unchanged) |
| `/settings/init` is auth-exempt | Yes | Yes (extended with claim) |
| Config init is one-shot (409 on repeat) | Yes | Yes |

**New security consideration:** The claim flow is a one-shot operation. Once a passphrase_hash file exists, the claim path in `/settings/init` returns 409. This prevents an attacker from reclaiming an already-configured instance. The race condition window (two browsers claiming simultaneously) is mitigated by atomic file creation.

---

## Implementation Sessions

### Session 02: Backend Claim Bootstrap

**Goal:** Implement the backend contract so `POST /api/settings/init` can accept an optional claim, create the passphrase hash, and return a session cookie.

**Files to modify:**

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/auth/passphrase.rs` | Add `create_passphrase_hash(data_dir, plaintext) -> Result<()>` |
| `crates/tuitbot-server/src/routes/settings.rs` | Extend `init_settings` with optional `claim` field; create hash + session on claim |
| `crates/tuitbot-server/src/main.rs` | Make startup passphrase generation conditional on `--host 0.0.0.0` |
| `crates/tuitbot-server/src/routes/lan.rs` | No change needed — `passphrase_configured` field already exists |

**Files to create:**

| File | Purpose |
|------|---------|
| `docs/roadmap/fresh-install-auth-ux/session-02-handoff.md` | Session handoff |

**Acceptance criteria:**
1. `POST /api/settings/init` with `claim.passphrase` creates config + passphrase_hash atomically.
2. Response includes `Set-Cookie: tuitbot_session=...` and `csrf_token` field.
3. If passphrase_hash already exists, claim is rejected (409).
4. Existing callers without `claim` field work identically (backward compatible).
5. `GET /api/settings/status` returns `claimed: bool` field.
6. All CI checks pass: `cargo fmt`, `cargo clippy`, `cargo test`.

**Risks:**

| Risk | Mitigation |
|------|------------|
| Race condition: two browsers claim simultaneously | `create_passphrase_hash` uses existence check before writing; first writer wins |
| Passphrase plaintext in logs | Never log the plaintext; use `tracing::info!("instance claimed")` |
| Breaking existing `init_settings` callers | `claim` is `Option<ClaimRequest>` — omitting preserves current behavior |
| CLI users expect terminal passphrase | Keep `ensure_passphrase` for `--host 0.0.0.0`; skip for localhost |

### Session 03: Frontend First-Run UX

**Goal:** Implement frontend routing changes so fresh installs reach onboarding before login, and the onboarding wizard handles instance claiming.

**Files to modify:**

| File | Change |
|------|--------|
| `dashboard/src/routes/+layout.svelte` | Reorder gate: check config/claimed before auth for non-bearer users |
| `dashboard/src/routes/onboarding/+page.svelte` | Add passphrase claim step with generation, copy, save warning |
| `dashboard/src/routes/login/+page.svelte` | Update copy; add "forgot passphrase?" hint |
| `dashboard/src/lib/api.ts` | Ensure `settings.init()` handles `Set-Cookie` response |
| `dashboard/src/lib/stores/auth.ts` | Handle claim-time session bootstrap |
| `dashboard/src/lib/stores/onboarding.ts` | Add passphrase field (never persisted to localStorage) |

**Files to create:**

| File | Purpose |
|------|---------|
| `dashboard/src/lib/components/onboarding/ClaimStep.svelte` | Passphrase creation UI with EFF wordlist |
| `docs/roadmap/fresh-install-auth-ux/session-03-handoff.md` | Session handoff |

**Acceptance criteria:**
1. Fresh install in web mode: browser → onboarding → claim passphrase → dashboard (no login screen).
2. Fresh install in Tauri mode: same as today (bearer → onboarding → dashboard, no passphrase step).
3. Returning web user with expired session: → login → passphrase → dashboard.
4. Configured web user with valid session: → dashboard directly.
5. Passphrase shown clearly during claim with copy-to-clipboard and save warning.
6. Login page has "forgot passphrase?" guidance.
7. `npm run check` and `npm run build` pass.

**Risks:**

| Risk | Mitigation |
|------|------------|
| User navigates away without saving passphrase | `beforeunload` warning; "I've saved this" checkbox |
| EFF wordlist bundle size | ~13KB gzipped — acceptable; can lazy-load |
| CSRF token not stored after claim | Ensure response handler mirrors `login()` behavior |
| Race between layout gate and onboarding submit | Local `claiming` state prevents double navigation |

### Session 04: Validation and Release Readiness

**Goal:** End-to-end validation of all flows, documentation updates, go/no-go report.

**Files to modify/create:**

| File | Purpose |
|------|---------|
| `docs/lan-mode.md` | Update to reflect claim flow |
| `docs/roadmap/fresh-install-auth-ux/release-readiness.md` | Go/no-go report |
| `docs/roadmap/fresh-install-auth-ux/session-04-handoff.md` | Final handoff |
| Various | Fix small validation findings |

**Manual test matrix:**

| Scenario | Expected |
|----------|----------|
| Delete `~/.tuitbot/`, start with `--host 0.0.0.0`, open browser | → onboarding, not login |
| Complete onboarding with claim | → dashboard, authenticated |
| Open incognito window on configured instance | → login page |
| Enter passphrase | → dashboard |
| Start Tauri app on fresh install | → onboarding (bearer), no passphrase step |
| Start Tauri app on configured instance | → dashboard |

---

## API Contract Changes (Summary)

### `GET /api/settings/status` (modified)

**Before:**
```json
{ "configured": false, "deployment_mode": "desktop", "capabilities": {...} }
```

**After:**
```json
{ "configured": false, "claimed": false, "deployment_mode": "desktop", "capabilities": {...} }
```

New `claimed` field: `true` if `passphrase_hash` file exists in `data_dir`.

### `POST /api/settings/init` (modified)

**Before (still works):**
```json
{ "x_api": {...}, "business": {...}, "llm": {...} }
```
Response: `{ "status": "created", "config": {...} }`

**After (with claim):**
```json
{
  "x_api": {...},
  "business": {...},
  "llm": {...},
  "claim": { "passphrase": "word1 word2 word3 word4" }
}
```
Response:
```json
{ "status": "created", "config": {...}, "csrf_token": "..." }
```
Plus `Set-Cookie: tuitbot_session=...; HttpOnly; Path=/; SameSite=Lax; Max-Age=604800`

**Error cases:**
- 409 if config already exists (unchanged).
- 409 if `claim` present but passphrase_hash already exists.
- 400 if `claim.passphrase` is empty or too short.

---

## Out of Scope

- Multi-user / multi-account auth (Tuitbot is single-user, local-first).
- OAuth/SSO integration.
- Email-based passphrase recovery.
- Changing the passphrase format (4-word EFF scheme is well-established).
- Modifying Tauri bearer-token behavior.
