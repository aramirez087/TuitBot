# Session 02 Handoff — Config and Credential Model

## Completed Work

1. **Migration: `connections` table.**
   Created `migrations/20260228000020_connections.sql` (and crate-level copy for
   `init_test_db()`). Schema matches charter connector model: `connector_type`,
   `account_email`, `encrypted_credentials` (BLOB), `status` lifecycle, composite
   index on `(connector_type, status)`.

2. **Config type extensions.**
   - `DeploymentCapabilities` now includes `preferred_source_default` (`"local_fs"`
     for Desktop, `"google_drive"` for SelfHost/Cloud).
   - `ContentSourceEntry` gains `connection_id: Option<i64>` with
     `#[serde(default, skip_serializing_if = "Option::is_none")]` — zero breakage
     for existing TOML configs.
   - New `ConnectorConfig` / `GoogleDriveConnectorConfig` structs for
     `[connectors.google_drive]` TOML section (client_id, client_secret,
     redirect_uri — all optional).
   - `Config` struct gains `#[serde(default)] pub connectors: ConnectorConfig`.

3. **Environment variable overrides.**
   Added `TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_ID`, `CLIENT_SECRET`, and
   `REDIRECT_URI` to `env_overrides.rs`.

4. **Validation.**
   Warning emitted when both `connection_id` and `service_account_key` are set on
   a single google_drive source (dual-config foot-gun per charter).

5. **Connection CRUD storage layer.**
   New `storage::watchtower::connections` submodule with `Connection` struct
   (intentionally excludes `encrypted_credentials`). Functions: `insert_connection`,
   `get_connection`, `get_connections` (active only), `update_connection_status`,
   `delete_connection`.

6. **Factory reset updated.**
   `connections` added to `TABLES_TO_CLEAR` in FK-safe order. Table count 30 → 31
   across reset.rs, settings.rs, and factory_reset.rs test assertions.

7. **Service-account-key redaction.**
   `GET /api/settings` now replaces non-null `service_account_key` values with
   `"[redacted]"` in content source entries (charter D5).

8. **Frontend type updates.**
   `api.ts` updated: `preferred_source_default` in `DeploymentCapabilities`,
   `connection_id` in source entry type, optional `connectors` on `TuitbotConfig`.

9. **Tests added.**
   - 13 new config tests: preferred_source_default per mode, connection_id
     defaults/roundtrip, legacy google_drive parsing, connector config
     TOML/env/optional, serialization skip behavior.
   - 5 new storage tests: migration check, insert/get, active-only filtering,
     status update, delete.
   - 2 new API tests: preferred_source_default in capability response,
     service_account_key redaction in GET settings.
   - All existing tests updated for new struct fields.

10. **Contract documentation.**
    `source-connection-contract.md` covers table schema, Connection struct,
    preferred_source_default values, ConnectorConfig TOML, redaction rules,
    legacy compatibility matrix, factory reset.

11. **CI clean.**
    `cargo fmt`, `cargo clippy`, and full test suite (1046 tests) all pass.

## Open Issues

None blocking Session 03.

**Note:** The `encrypted_credentials` column exists in the migration but no
encryption/decryption logic exists yet. Session 03 implements AES-256-GCM
encryption in `connector/crypto.rs`.

## Inputs for Session 03

| Input | Location | Notes |
|-------|----------|-------|
| Charter | `docs/roadmap/.../charter.md` | D1-D6 design decisions, OAuth flow design |
| Connection contract | `docs/roadmap/.../source-connection-contract.md` | Stable API surface for connector model |
| `connections` table | `migrations/20260228000020_connections.sql` | Schema ready; CRUD in `storage::watchtower::connections` |
| ConnectorConfig | `crates/tuitbot-core/src/config/types.rs` | `client_id`, `client_secret`, `redirect_uri` parsed from TOML/env |
| Google Drive source | `crates/tuitbot-core/src/source/google_drive.rs` | Existing service-account JWT path to keep as legacy fallback |
| Server routes | `crates/tuitbot-server/src/routes/` | Add `connectors.rs` with link/callback/status/disconnect |
| API tests | `crates/tuitbot-server/tests/api_tests.rs` | Add connector endpoint tests |

Session 03 deliverables:
- `crates/tuitbot-core/src/source/connector/` module (trait + google_drive OAuth + crypto)
- `crates/tuitbot-server/src/routes/connectors.rs` (link, callback, status, disconnect)
- `docs/roadmap/deployment-aware-content-source-setup/drive-connection-flow.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-03-handoff.md`
