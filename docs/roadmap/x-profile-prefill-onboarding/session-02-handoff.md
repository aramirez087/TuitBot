# Session 02 Handoff

## What Changed

Implemented the unified onboarding entry path with X sign-in as the primary CTA across all deployment modes.

### New Files

| File | Purpose |
|------|---------|
| `crates/tuitbot-server/src/routes/onboarding.rs` | Pre-account OAuth PKCE endpoints for onboarding (`start`, `callback`, `status`) |
| `dashboard/src/lib/stores/onboarding-session.ts` | Client-side onboarding auth session state (X connection, user identity) |
| `docs/roadmap/x-profile-prefill-onboarding/session-02-handoff.md` | This handoff |

### Modified Files

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/x_api/types.rs` | Added `description`, `location`, `url` fields to `User` struct + tests |
| `crates/tuitbot-core/src/x_api/client/mod.rs` | Updated `USER_FIELDS` to include `description,location,url` |
| `crates/tuitbot-core/src/x_api/local_mode/cookie_transport.rs` | Added new fields to `User` construction in `fetch_viewer()` |
| `crates/tuitbot-server/src/routes/mod.rs` | Registered `onboarding` module |
| `crates/tuitbot-server/src/lib.rs` | Wired 3 onboarding routes before accounts routes |
| `crates/tuitbot-server/src/auth/middleware.rs` | Added 6 auth exemptions for onboarding endpoints |
| `crates/tuitbot-server/src/routes/settings.rs` | Added token migration from `onboarding_tokens.json` to default account in `init_settings` |
| `dashboard/src/lib/api/client.ts` | Added `api.onboarding.{startAuth, completeAuth, authStatus}` methods |
| `dashboard/src/lib/components/onboarding/XApiStep.svelte` | Added "Continue with X" button, polling-based OAuth flow, connected state display |
| 14 test/mock files across `tuitbot-core`, `tuitbot-mcp` | Added `description: None, location: None, url: None` to `User` struct constructions |

## Key Decisions Made

### D1: New `/api/onboarding/x-auth/*` endpoints (not reusing account-scoped)
Created separate onboarding-specific endpoints using `__onboarding__` sentinel as the account ID in `PendingOAuth`. This prevents cross-flow state theft and avoids coupling onboarding to account lifecycle.

### D2: Temporary token storage at `{data_dir}/onboarding_tokens.json`
Single well-known path. Migrated to `account_token_path(data_dir, DEFAULT_ACCOUNT_ID)` during `POST /api/settings/init` via `std::fs::rename`. Non-fatal if migration fails.

### D3: Skip `entities` field on User struct
Added only `description`, `location`, `url` (all `Option<String>` with `#[serde(default)]`). The `entities` field is deeply nested and covers <5% of remaining inference value.

### D4: Polling-based callback detection
After opening the auth URL, the frontend polls `GET /api/onboarding/x-auth/status` every 2 seconds for up to 5 minutes. This works regardless of how the callback code reaches the server (Tauri local server, manual paste, or future redirect handler).

### D5: OAuth connection is encouraged but not blocking
The `canAdvance()` logic for step 1 (X Access) remains unchanged: it requires a non-empty `client_id` for API mode but does not require OAuth connection. Users can connect later in Settings.

### D6: Onboarding session state is client-only
The `onboarding-session.ts` store holds ephemeral auth/connection state. It's separate from `onboarding.ts` (config data submitted to `/api/settings/init`) to keep concerns clean.

## API Contracts

### `POST /api/onboarding/x-auth/start`
**Auth:** Exempt
**Request:** Empty body
**Response:** `{ authorization_url: string, state: string }`
**Errors:** 400 if `x_client_id` not configured

### `POST /api/onboarding/x-auth/callback`
**Auth:** Exempt
**Request:** `{ code: string, state: string }`
**Response:**
```json
{
  "status": "connected",
  "user": {
    "id": "123456",
    "username": "handle",
    "name": "Display Name",
    "profile_image_url": "https://...",
    "description": "Bio text",
    "location": "City, Country",
    "url": "https://t.co/..."
  }
}
```
**Errors:** 400 invalid/expired state, 400 account mismatch, 500 token exchange failure

### `GET /api/onboarding/x-auth/status`
**Auth:** Exempt
**Response:** `{ connected: true, user: {...} }` or `{ connected: false }`

## Open Issues

1. **Stale onboarding tokens cleanup** — If a user starts onboarding OAuth but never finishes setup, `onboarding_tokens.json` persists on disk. Proposed: clean up on server startup if file is older than 1 hour. Not implemented this session (low risk — tokens expire in 2 hours anyway).

2. **Callback routing for web/LAN mode** — The current flow assumes the Tauri callback server at port 8080 handles the OAuth callback and calls the server endpoint. In pure web mode (self-host over LAN), the callback server may not exist. Two options for Session 03:
   - Add a server-side callback handler at `/api/onboarding/x-auth/redirect` that receives the browser redirect
   - Keep the polling approach and have users paste the callback URL manually (current fallback)

3. **`get_me()` call in callback adds ~200-500ms** — Acceptable for a one-time operation. If latency proves problematic, split into async: return `status: linked` immediately, fetch identity on subsequent `status` poll.

4. **Token migration race condition** — If `init_settings` is called while an OAuth flow is in-flight (tokens not yet saved), migration is a no-op. The flow completes later and tokens stay in `onboarding_tokens.json`. Mitigation: Session 03 should consider polling or retrying migration.

## What Session 03 Must Do

### Mission
Build the profile analysis endpoint that fetches the X user profile (using onboarding tokens) and runs LLM-powered inference to pre-fill business/brand fields.

### Prerequisites from this session
- `User` struct now includes `description`, `location`, `url` from X API
- Onboarding tokens are available at `onboarding_tokens.json` (pre-init) or `account_token_path` (post-init)
- `api.onboarding.authStatus()` returns the full user profile including bio

### Files to Create/Modify

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/workflow/profile_inference.rs` | New module: `InferredProfile` type, `infer_profile()` function using LLM |
| `crates/tuitbot-server/src/routes/onboarding.rs` | Add `POST /api/onboarding/analyze-profile` endpoint |
| `dashboard/src/lib/stores/onboarding-session.ts` | Add `inferred_profile` field |
| `dashboard/src/lib/components/onboarding/ProfileReviewStep.svelte` | New step: editable prefill review with confidence badges |
| `dashboard/src/routes/onboarding/+page.svelte` | Update step flow for profile analysis + review |

### Decisions Session 03 Must Make

1. Whether to implement the two-pass fallback (heuristic extraction first, LLM enrichment after LLM step) or keep the simpler single-pass approach (LLM required before profile analysis).
2. Exact `InferredProfile` field mapping from the inference contract in `inference-contract.md`.
3. Whether the analyze endpoint should also fetch recent tweets for richer inference, or just use the profile fields.

### Verification

After Session 03:
- `cargo fmt --all && cargo fmt --all --check` passes
- `RUSTFLAGS="-D warnings" cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` passes
- `cd dashboard && npm run check` passes
- Profile analysis endpoint returns inferred fields from X profile data
- Inferred fields are editable on a single review screen before advancing
