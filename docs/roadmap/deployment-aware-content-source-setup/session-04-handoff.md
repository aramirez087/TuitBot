# Session 04 Handoff -- Watchtower Provider Refactor

## Completed Work

1. **DriveAuthStrategy enum and dual-auth dispatch.**
   New `DriveAuthStrategy` enum in `source/google_drive/mod.rs` with
   `ServiceAccount` and `LinkedAccount` variants. The `get_access_token()`
   method dispatches to JWT signing or connector-based refresh depending
   on strategy. Token caching with 60-second expiry buffer shared by both
   paths.

2. **ConnectionBroken error variant.**
   Added `SourceError::ConnectionBroken { connection_id, reason }` to
   `source/mod.rs`. Returned when credentials are missing, revoked, or
   otherwise irrecoverable. The Watchtower maps this to source status
   `"error"` and connection status `"expired"`.

3. **Module directory refactor.**
   Split `google_drive.rs` (684 lines) into `google_drive/mod.rs` (441
   lines) + `google_drive/jwt.rs` (354 lines). Legacy RSA/JWT signing
   code isolated in `jwt.rs`. Both files under the 500-line limit.

4. **`from_connection()` constructor.**
   New `GoogleDriveProvider::from_connection()` builds a provider using
   connection credentials: loads the connector key from `data_dir`,
   constructs a `GoogleDriveConnector` from `ConnectorConfig`, sets up
   `LinkedAccount` strategy. Reads encrypted refresh token from DB and
   calls `RemoteConnector::refresh_access_token()` on demand.

5. **`refresh_from_connection()` method.**
   Reads encrypted credentials from the connection row, decrypts with
   the connector key, calls `connector.refresh_access_token()`, caches
   the result. Maps revocation errors (`invalid_grant`, `Token has been
   revoked`) to `ConnectionBroken`.

6. **Watchtower connection-based source registration.**
   `WatchtowerLoop::run()` now branches on `connection_id` (priority 1)
   vs `service_account_key` (priority 2) vs neither (skip with warning).
   New `build_connection_provider()` helper method loads the connector
   key and builds the provider.

7. **Watchtower ConnectionBroken handling.**
   `poll_remote_sources()` catches `ConnectionBroken`, updates the source
   to `status = "error"` and the connection to `status = "expired"`, then
   continues the loop without crashing.

8. **Server wiring.**
   - Added `connector_config: ConnectorConfig` to `AppState`.
   - `main.rs` extracts `connector_config` from loaded config, passes to
     both `WatchtowerLoop::new()` and `AppState`.
   - Simplified `routes/connectors.rs` to read `connector_config` from
     `AppState` instead of re-loading config from disk.

9. **Tests added (10 new).**
   - 4 provider unit tests: `from_connection` gets token (wiremock),
     missing credentials returns ConnectionBroken, revoked token returns
     ConnectionBroken, connection construction.
   - 2 integration tests: revoked connection degrades gracefully (e2e),
     restart recovery preserves cursor (e2e).
   - 4 Watchtower tests: skips source without auth, handles broken
     connection, preserves cursor across restart, mixed legacy and
     connection sources.

10. **CI clean.**
    `cargo fmt`, `cargo clippy`, and full test suite (1795 tests) all pass.

## Design Decisions Made

- **KD1 (dual-auth enum over trait object):** Used an enum rather than a
  trait object for `DriveAuthStrategy` because there are exactly two
  strategies and the enum keeps the code simpler with exhaustive matching.

- **KD2 (60-second token cache buffer):** Access tokens are refreshed
  60 seconds before their reported `expires_in`. This avoids edge cases
  where a valid-looking token expires between check and use.

- **KD3 (ConnectionBroken vs generic error):** Introduced a dedicated
  error variant rather than reusing `SourceError::Fetch` because the
  Watchtower needs to distinguish "retry later" (transient) from "stop
  trying and mark expired" (permanent revocation).

- **KD4 (connector_config in AppState):** Moved `ConnectorConfig` into
  `AppState` so both the connector routes and the Watchtower can access
  it without re-reading config from disk. This is consistent with how
  other shared config (`api_token`, `bind_host`, etc.) is handled.

## Open Issues

None blocking Session 05.

**Note:** The `WatchtowerLoop` constructor now takes 4 parameters instead
of 2. Any future callers (e.g., test harnesses) need to provide
`ConnectorConfig` and `data_dir`. Both accept `Default::default()` and
`PathBuf::new()` respectively for tests that don't exercise connection
features.

## Inputs for Session 05

| Input | Location | Notes |
|-------|----------|-------|
| DriveAuthStrategy | `crates/tuitbot-core/src/source/google_drive/mod.rs` | Dual-auth dispatch |
| ConnectionBroken error | `crates/tuitbot-core/src/source/mod.rs` | Provider error → Watchtower handling |
| Watchtower connection flow | `crates/tuitbot-core/src/automation/watchtower/mod.rs` | Registration + polling |
| ConnectorConfig in AppState | `crates/tuitbot-server/src/state.rs` | Shared OAuth config |
| Connector routes | `crates/tuitbot-server/src/routes/connectors.rs` | Link, callback, status, disconnect |
| Sync contract docs | `docs/roadmap/.../watchtower-sync-contract.md` | Cursor stability, error handling |
| Drive flow docs | `docs/roadmap/.../drive-connection-flow.md` | Security model, endpoint reference |

Session 05 deliverables:
- Onboarding UI: source-type selection (local FS vs Google Drive)
- Settings UI: connector linking/unlinking in dashboard
- Frontend store integration for connection status
