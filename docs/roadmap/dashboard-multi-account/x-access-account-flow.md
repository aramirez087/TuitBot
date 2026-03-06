# X Access Account Flow

Per-account credential management from the dashboard, covering OAuth PKCE token linking and browser scraper session import.

## OAuth PKCE Flow

### Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/accounts/{id}/x-auth/start` | Generate PKCE challenge, return authorization URL |
| POST | `/api/accounts/{id}/x-auth/callback` | Exchange authorization code for tokens |
| GET | `/api/accounts/{id}/x-auth/status` | Check OAuth + scraper credential status |
| DELETE | `/api/accounts/{id}/x-auth/tokens` | Delete token file, evict TokenManager cache |

### Link Flow

1. User clicks "Link X Account" on a specific account's credential card.
2. Frontend calls `POST /api/accounts/{id}/x-auth/start`.
3. Server generates PKCE challenge (code_verifier + code_challenge), stores in `pending_oauth` map keyed by random `state`, returns `{ authorization_url, state }`.
4. Frontend opens `authorization_url` in a new browser tab via `window.open()`.
5. User authorizes in the X consent screen, gets redirected to the callback URI with `code` and `state` params.
6. User copies the authorization code and pastes it into the dashboard input.
7. Frontend calls `POST /api/accounts/{id}/x-auth/callback` with `{ code, state }`.
8. Server validates state, exchanges code for tokens, saves to account-specific `tokens.json`, evicts cached TokenManager.
9. Frontend syncs account profile (avatar, username) and reloads runtime capabilities.

### Relink Flow

Same as Link but replaces existing tokens. The old token file is overwritten atomically.

### Unlink Flow

1. User clicks "Unlink" on OAuth section of credential card.
2. Frontend calls `DELETE /api/accounts/{id}/x-auth/tokens`.
3. Server deletes the token file at `accounts/{id}/tokens.json` and evicts cached TokenManager.
4. Frontend reloads auth status and runtime capabilities for the active account.

## Browser Scraper Session

### Endpoints

These use the existing scraper session endpoints with explicit `X-Account-Id` header override:

| Method | Path | Header Override |
|--------|------|----------------|
| GET | `/api/settings/scraper-session` | `X-Account-Id: {target_account_id}` |
| POST | `/api/settings/scraper-session` | `X-Account-Id: {target_account_id}` |
| DELETE | `/api/settings/scraper-session` | `X-Account-Id: {target_account_id}` |

### Import Flow

1. User expands credential card, clicks "Import Session".
2. Inline form appears with `auth_token`, `ct0`, and optional `username` fields.
3. Frontend calls POST with the target account's ID in `X-Account-Id` header.
4. On success, form closes and auth status refreshes.

### Remove Flow

1. User clicks "Remove" on scraper section.
2. Frontend calls DELETE with the target account's ID.
3. Auth status refreshes.

## Cross-Account Isolation

- Each API call targets a specific account via path parameter (OAuth) or `X-Account-Id` header override (scraper).
- The `request()` function in `http.ts` merges headers with `...options?.headers` last, so explicit `X-Account-Id` overrides the module-level default.
- Credential changes on non-active accounts do not trigger runtime capability reload (only the active account's `can_post` matters for the composer).
- Tests verify that unlinking Account A's OAuth does not affect Account B's scraper session.

## Immediate UI Refresh Contract

After any credential change (link, relink, unlink, import, remove):

1. `loadAuthStatuses()` is called to refresh all account credential badges.
2. If the changed account is the active account, `reloadCapabilities()` is called to update runtime `can_post` status.
3. After OAuth link, `syncAccountProfile()` is called to pull avatar/display name from X.

## Security Notes

- PKCE state entries expire after 600 seconds.
- State parameter is validated against the account ID to prevent cross-account token theft.
- Token files are written with 0600 permissions.
- The unlink endpoint requires `require_mutate()` authorization.
