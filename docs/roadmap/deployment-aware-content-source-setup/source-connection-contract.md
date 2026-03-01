# Source Connection Contract

Stable API surface for the connector model introduced in Session 02.
Sessions 03-07 build on these types and tables without redefining them.

---

## 1. `connections` Table

```sql
CREATE TABLE IF NOT EXISTS connections (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id            TEXT    NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    connector_type        TEXT    NOT NULL,          -- "google_drive", future: "onedrive", etc.
    account_email         TEXT,                      -- Display only
    display_name          TEXT,                      -- e.g. "My Google Drive"
    encrypted_credentials BLOB,                      -- Refresh token, encrypted at rest
    status                TEXT    NOT NULL DEFAULT 'active', -- "active", "expired", "revoked"
    metadata_json         TEXT    NOT NULL DEFAULT '{}',     -- Non-secret connector metadata
    created_at            TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at            TEXT    NOT NULL DEFAULT (datetime('now'))
);
```

**Index:** `idx_connections_type_status ON connections(connector_type, status)`

### Column semantics

| Column | Usage |
|--------|-------|
| `connector_type` | Discriminator for which OAuth provider to use. Currently only `"google_drive"`. |
| `account_email` | User-facing display only. Set during OAuth callback from the Google userinfo endpoint. |
| `encrypted_credentials` | AES-256-GCM encrypted refresh token (session 03 implements encryption). Never returned by any query in `connections.rs`. |
| `status` | Lifecycle: `active` (usable), `expired` (token expired, needs re-auth), `revoked` (user disconnected). |
| `metadata_json` | Non-secret connector config (e.g. folder preferences). Safe to include in API responses. |

---

## 2. `Connection` Struct (Public API)

```rust
pub struct Connection {
    pub id: i64,
    pub account_id: String,
    pub connector_type: String,
    pub account_email: Option<String>,
    pub display_name: Option<String>,
    pub status: String,
    pub metadata_json: String,
    pub created_at: String,
    pub updated_at: String,
}
```

**Security:** `encrypted_credentials` is intentionally excluded. Only the
connector module (session 03) reads credentials via a separate dedicated
function that requires explicit opt-in.

### CRUD functions (in `storage::watchtower::connections`)

| Function | Signature |
|----------|-----------|
| `insert_connection` | `(pool, connector_type, account_email?, display_name?) -> Result<i64>` |
| `get_connection` | `(pool, id) -> Result<Option<Connection>>` |
| `get_connections` | `(pool) -> Result<Vec<Connection>>` (active only) |
| `update_connection_status` | `(pool, id, status) -> Result<()>` |
| `delete_connection` | `(pool, id) -> Result<()>` |

---

## 3. `preferred_source_default` Capability

Added to `DeploymentCapabilities`:

```rust
pub preferred_source_default: String,
```

Values per deployment mode:

| Mode | Value |
|------|-------|
| Desktop | `"local_fs"` |
| SelfHost | `"google_drive"` |
| Cloud | `"google_drive"` |

Returned in `GET /api/settings/status` response under `capabilities`.
The frontend uses this to set the initial `source_type` in the onboarding
store, replacing the hardcoded `"local_fs"` default.

---

## 4. `connection_id` on `ContentSourceEntry`

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub connection_id: Option<i64>,
```

When set, the Watchtower uses the linked account's credentials (from the
`connections` table) instead of the legacy `service_account_key` path.

**Legacy safety:** Existing TOML configs without this field deserialize
with `connection_id = None`, triggering the legacy service-account-key
path in the Watchtower. Zero breakage per charter D4.

---

## 5. `ConnectorConfig` in TOML

```toml
[connectors.google_drive]
client_id = "123456.apps.googleusercontent.com"
client_secret = "GOCSPX-secret"
redirect_uri = "http://localhost:3001/api/connectors/google-drive/callback"
```

All fields are optional. Environment variable overrides:

| Env var | Config field |
|---------|-------------|
| `TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_ID` | `connectors.google_drive.client_id` |
| `TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_SECRET` | `connectors.google_drive.client_secret` |
| `TUITBOT_CONNECTORS__GOOGLE_DRIVE__REDIRECT_URI` | `connectors.google_drive.redirect_uri` |

Env overrides are applied *after* TOML load, same as all other env vars.
They take precedence over TOML values.

---

## 6. Redaction Rules

- `service_account_key` in `GET /api/settings` responses is replaced with
  `"[redacted]"` when non-null.
- `encrypted_credentials` is never selected in any `Connection` query.
- `ConnectorConfig.client_secret` is currently returned as-is in settings
  responses (it is an application credential, not a user secret). If needed
  in the future, it can be redacted the same way.

---

## 7. Legacy Compatibility

| Existing config | Behavior after upgrade |
|-----------------|----------------------|
| `local_fs` with `path` | Fully preserved. No change. |
| `google_drive` with `service_account_key` + `folder_id` | Preserved. Legacy JWT code path remains as fallback. |
| `service_account_key` field in TOML | Not deleted. Accepted via `#[serde(default)]`. Redacted in API responses. |
| New `connection_id` field | Defaults to `None`. Old configs continue to work unchanged. |
| No `[connectors]` section | Uses defaults (all `None`). Connector linking unavailable until configured. |

---

## 8. Factory Reset

The `connections` table is included in `TABLES_TO_CLEAR` in FK-safe order
(after `content_nodes`, before `thread_tweets`). Total table count is now 31.
