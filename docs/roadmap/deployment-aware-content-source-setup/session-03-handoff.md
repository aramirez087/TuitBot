# Session 03 Handoff -- Drive Connection Backend

## Completed Work

1. **AES-256-GCM credential encryption.**
   New `source/connector/crypto.rs` module: `ensure_connector_key` (generates
   and persists a 32-byte key at `<data_dir>/connector_key` with 0600 perms),
   `encrypt_credentials`, `decrypt_credentials`. Ciphertext format:
   `nonce(12) || ciphertext(N) || tag(16)`. Added `aes-gcm = "0.10"` to
   `tuitbot-core/Cargo.toml`.

2. **RemoteConnector trait and error types.**
   New `source/connector/mod.rs`: `ConnectorError` enum (NotConfigured,
   InvalidState, TokenExchange, TokenRefresh, Revocation, Encryption, Storage,
   Network), `RemoteConnector` trait with `authorization_url`, `exchange_code`,
   `refresh_access_token`, `revoke`, `user_info`. Supporting types: `TokenSet`,
   `RefreshedToken`, `UserInfo`.

3. **GoogleDriveConnector implementation.**
   New `source/connector/google_drive.rs`: Implements `RemoteConnector` using
   raw reqwest (no oauth2 crate dependency for Google flow). Handles:
   - Authorization URL generation with PKCE S256 + `access_type=offline`
   - Token exchange with form-encoded POST to Google's token endpoint
   - Access token refresh from encrypted refresh token
   - Token revocation (best-effort, tolerates 400 "already revoked")
   - User info fetch (email + display name)
   - Test infrastructure via thread-local URL overrides + wiremock

4. **Connection storage extensions.**
   Added to `storage/watchtower/connections.rs`:
   - `get_connections_by_type(pool, connector_type)` -- filter by type
   - `store_encrypted_credentials(pool, id, ciphertext)` -- explicit opt-in
   - `read_encrypted_credentials(pool, id)` -- explicit opt-in
   - `update_connection_metadata(pool, id, metadata_json)` -- update non-secret metadata

5. **Server state: PendingOAuth.**
   Added `PendingOAuth` struct and `pending_oauth: Mutex<HashMap<String, PendingOAuth>>`
   to `AppState`. Updated all AppState constructions in main.rs and test files.

6. **Connector API routes.**
   New `routes/connectors.rs` with four endpoints:
   - `POST /api/connectors/google-drive/link` -- starts PKCE flow, returns auth URL
   - `GET /api/connectors/google-drive/callback` -- exchanges code, encrypts
     refresh token, stores connection, returns HTML success page
   - `GET /api/connectors/google-drive/status` -- returns active connections
   - `DELETE /api/connectors/google-drive/{id}` -- revokes + deletes connection

7. **Auth exemption for callback.**
   Added `/connectors/google-drive/callback` and `/api/connectors/google-drive/callback`
   to `AUTH_EXEMPT_PATHS` in `middleware.rs`. Callback is state-validated
   (unpredictable CSRF token, 10-min TTL, one-shot consumption).

8. **Tests added.**
   - 7 crypto tests: round-trip, wrong key, corrupt data, short blob, wrong
     key length, key creation + idempotency, Unix file permissions.
   - 8 Google Drive connector tests (wiremock): auth URL params, not-configured,
     exchange happy/error, refresh happy/revoked, revoke happy/already-revoked,
     user info parsing.
   - 2 integration tests: full encrypt-store-decrypt round trip, different
     keys produce different ciphertext.
   - 4 storage tests: get_connections_by_type, store/read encrypted credentials,
     read missing credentials, update metadata.
   - 10 API tests: link not configured, link requires auth, callback missing
     params, callback invalid state, callback auth-exempt, status empty,
     status requires auth, status with seeded connection, disconnect not found,
     disconnect requires auth.

9. **Documentation.**
   - `drive-connection-flow.md`: sequence diagram, security model, endpoint
     reference, error states, PKCE lifecycle, connection lifecycle.
   - This handoff document.

10. **CI clean.**
    `cargo fmt`, `cargo clippy`, and full test suite (1785 tests) all pass.

## Design Decisions Made

- **KD4 (raw reqwest over oauth2 crate):** Used reqwest directly for Google
  OAuth, matching the existing `x_api/auth.rs` pattern. Avoids fighting the
  oauth2 crate's BasicClient assumptions for Google's form-body preference.

- **Thread-local URL overrides for testing:** Rather than adding URL fields
  to the connector struct (which would bloat production code), used
  `thread_local!` for test-only URL injection. This keeps the production
  struct clean.

- **`base64` and `chrono` deps added to tuitbot-server:** Needed for PKCE
  code_challenge encoding and connection metadata timestamps. Both are
  already transitive dependencies via tuitbot-core.

- **Callback returns HTML (not JSON):** The callback is hit by the browser
  after Google's consent screen, so HTML is the appropriate response format.
  Includes a `postMessage` script for dashboard detection.

## Open Issues

None blocking Session 04.

**Note:** The `force=true` auto-disconnect path in the link endpoint checks
for existing connections but does not auto-disconnect them yet. The 409
response instructs the client to disconnect manually first. This is the
safer path -- auto-disconnect can be added in a later session if needed.

## Inputs for Session 04

| Input | Location | Notes |
|-------|----------|-------|
| Connector trait | `crates/tuitbot-core/src/source/connector/mod.rs` | `RemoteConnector` with `refresh_access_token()` |
| GoogleDriveConnector | `crates/tuitbot-core/src/source/connector/google_drive.rs` | Full OAuth implementation |
| Crypto module | `crates/tuitbot-core/src/source/connector/crypto.rs` | `encrypt_credentials`, `decrypt_credentials`, `ensure_connector_key` |
| Connection CRUD | `crates/tuitbot-core/src/storage/watchtower/connections.rs` | Full CRUD + credential storage |
| Connector routes | `crates/tuitbot-server/src/routes/connectors.rs` | Link, callback, status, disconnect |
| Drive flow docs | `docs/roadmap/.../drive-connection-flow.md` | Security model and endpoint reference |
| Content source provider | `crates/tuitbot-core/src/source/mod.rs` | `ContentSourceProvider` trait -- Watchtower integration point |
| Google Drive provider | `crates/tuitbot-core/src/source/google_drive.rs` | Existing service-account JWT path (legacy fallback) |

Session 04 deliverables:
- Extend `GoogleDriveProvider` to use connection credentials when `connection_id` is set
- Wire Watchtower to prefer connection-based auth over service-account-key
- Add frontend connector linking UI in the dashboard
