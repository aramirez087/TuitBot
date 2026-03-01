# Drive Connection Flow

End-to-end OAuth 2.0 flow for linking a user-owned Google Drive account
to Tuitbot. Implemented in Session 03.

---

## Sequence Diagram

```
  Dashboard               Server                Google
  --------               ------                ------
     |                      |                      |
     |  POST /link          |                      |
     |--------------------->|                      |
     |                      | Generate PKCE:       |
     |                      |  code_verifier       |
     |                      |  code_challenge      |
     |                      |  state (CSRF)        |
     |                      | Store in memory      |
     |  { authorization_url, state }               |
     |<---------------------|                      |
     |                      |                      |
     | Redirect user ------>|--------------------->|
     |                      |   Consent screen     |
     |                      |<---------------------|
     |                      | GET /callback?code=X&state=Y
     |                      |                      |
     |                      | Validate state       |
     |                      | Exchange code:       |
     |                      |  POST /token ------->|
     |                      |  { access, refresh } |
     |                      |<---------------------|
     |                      |                      |
     |                      | GET /userinfo ------->|
     |                      | { email, name }      |
     |                      |<---------------------|
     |                      |                      |
     |                      | Encrypt refresh tok  |
     |                      | Insert connection    |
     |                      | Store credentials    |
     |                      | Update metadata      |
     |                      |                      |
     |  HTML success page   |                      |
     |<---------------------|                      |
     |  (postMessage to     |                      |
     |   window.opener)     |                      |
```

---

## Security Model

### PKCE (Proof Key for Code Exchange)

- **code_verifier**: 128 hex chars (64 random bytes), generated server-side.
- **code_challenge**: `BASE64URL(SHA256(code_verifier))`, sent to Google in
  the authorization URL.
- **Binding**: Google returns the authorization code only if the challenge
  matches. The server exchanges the code with the original verifier.

### State Parameter (CSRF Protection)

- **state**: 64 hex chars (32 random bytes), unique per link request.
- **Storage**: In-memory `Mutex<HashMap<String, PendingOAuth>>` on `AppState`.
- **TTL**: 10 minutes. Expired entries are cleaned on each new link request.
- **Consumption**: One-shot -- removed from the map after use.

### Credential Encryption (AES-256-GCM)

- **Key**: 32 random bytes stored at `<data_dir>/connector_key` with 0600
  permissions. Generated once per Tuitbot instance.
- **Format**: `nonce(12) || ciphertext(N) || tag(16)`.
- **Storage**: The `encrypted_credentials` BLOB column in the `connections`
  table. Never selected in standard `Connection` queries.
- **Access**: Only `read_encrypted_credentials()` and
  `store_encrypted_credentials()` touch this column -- explicit opt-in.

### Auth Exemption

The callback endpoint (`GET /api/connectors/google-drive/callback`) is
auth-exempt because the browser navigates to it after Google's consent
screen -- there is no opportunity to attach a Bearer token. Security
is provided by the `state` parameter validation.

---

## Endpoint Reference

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/connectors/google-drive/link` | Required | Start link flow. Returns `{ authorization_url, state }`. Pass `?force=true` to auto-disconnect existing. |
| GET | `/api/connectors/google-drive/callback` | Exempt | OAuth callback. Exchanges code, stores connection. Returns HTML success page. |
| GET | `/api/connectors/google-drive/status` | Required | List active Google Drive connections (`{ connections: [...] }`). |
| DELETE | `/api/connectors/google-drive/{id}` | Required | Revoke + delete a connection. Returns `{ disconnected: true, id }`. |

---

## Error States and Recovery

| Scenario | Response | Recovery |
|----------|----------|----------|
| Connector not configured (no client_id/secret) | 400 "not configured" | Set `[connectors.google_drive]` in config.toml or env vars |
| Duplicate active connection | 409 "already exists" | Disconnect first, or pass `?force=true` |
| Missing code/state in callback | 400 "missing parameter" | Retry the link flow |
| Expired state (>10 min) | 400 "state expired" | Retry the link flow |
| Invalid/unknown state | 400 "invalid or expired state" | Retry the link flow |
| Token exchange failure | 400 "token exchange failed" | Check Google Cloud Console configuration |
| Encryption key error | 500 "encryption key error" | Check data_dir permissions |
| Already-revoked token on disconnect | 200 (best-effort) | No action needed |

---

## PKCE State Lifecycle

1. **Created**: On `POST /link`, stored in `AppState.pending_oauth`.
2. **Cleaned**: Expired entries (>10 min) pruned on each new link request.
3. **Consumed**: On `GET /callback`, removed from the map.
4. **Lost on restart**: Acceptable -- user retries the link flow. Maximum
   impact is 10 minutes of wasted state.

---

## Connection Lifecycle

```
POST /link --> user consents --> GET /callback --> INSERT (status=active)
                                                        |
                                    GET /status <-------+
                                                        |
                                    DELETE /{id} -----> revoke + DELETE row
```

Status values: `active`, `expired` (future: when refresh fails), `revoked`
(future: when Google notifies revocation).
