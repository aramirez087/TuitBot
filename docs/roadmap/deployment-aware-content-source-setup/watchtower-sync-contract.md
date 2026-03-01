# Watchtower Sync Contract

Stable contract for remote-source polling in the Watchtower loop after the
Session 04 provider refactor. Sessions 05-07 build on these types without
redefining them.

---

## 1. DriveAuthStrategy

```rust
pub enum DriveAuthStrategy {
    ServiceAccount { key_path: String },
    LinkedAccount {
        connection_id: i64,
        pool: DbPool,
        connector_key: Vec<u8>,
        connector: GoogleDriveConnector,
    },
}
```

**Dispatch in `get_access_token()`:** The provider calls
`DriveAuthStrategy::ServiceAccount` for legacy JWT signing (self-contained
RSA in `jwt.rs`) or `DriveAuthStrategy::LinkedAccount` for credential-based
refresh via `RemoteConnector::refresh_access_token()`.

### Token caching

Both strategies share a `Mutex<Option<CachedToken>>` that stores the last
successful access token and its `Instant`-based expiry. The cache uses a
60-second safety buffer — tokens are refreshed 60s before the reported
`expires_in`.

---

## 2. Error Propagation: ConnectionBroken

```rust
#[error("connection broken (id={connection_id}): {reason}")]
ConnectionBroken { connection_id: i64, reason: String },
```

Added to `SourceError`. Returned by the provider when:

- No encrypted credentials exist for the `connection_id`
- The refresh token is revoked (`invalid_grant`, `Token has been revoked`)
- Any connector error maps to a permanent failure

### Watchtower handling

When `poll_remote_sources()` receives `ConnectionBroken`:

1. Updates the `content_sources` row to `status = "error"`, storing the
   reason in `last_error`.
2. Updates the `connections` row to `status = "expired"`.
3. Logs a warning but does **not** crash the loop or affect other sources.

Transient errors (network timeouts, 5xx) are logged and retried on the
next poll cycle — they do **not** trigger ConnectionBroken.

---

## 3. Source Registration Priority

When `WatchtowerLoop::run()` registers remote sources from config:

| Priority | Field checked | Auth strategy |
|----------|--------------|---------------|
| 1 (highest) | `connection_id` is `Some` | `LinkedAccount` — load connector key from `data_dir`, build `GoogleDriveConnector` from `ConnectorConfig` |
| 2 | `service_account_key` is `Some` | `ServiceAccount` — legacy JWT path |
| 3 (skip) | Neither set | Log warning, skip source |

This ordering preserves backward compatibility: existing TOML configs with
`service_account_key` continue working. New configs with `connection_id`
take precedence.

---

## 4. WatchtowerLoop Construction

```rust
pub fn new(
    pool: DbPool,
    config: ContentSourcesConfig,
    connector_config: ConnectorConfig,
    data_dir: PathBuf,
) -> Self
```

New parameters added in Session 04:

| Parameter | Purpose |
|-----------|---------|
| `connector_config` | Application OAuth credentials (client_id, client_secret, redirect_uri) for building `GoogleDriveConnector` |
| `data_dir` | Data directory path for loading the connector encryption key via `ensure_connector_key()` |

---

## 5. Cursor Stability

The `scan_for_changes()` method's change-token/page-token cursor behavior
is unchanged. Connection-based sources use the same cursor storage
(`content_sources.change_token` column) as service-account sources.

This means:
- Restarting the server preserves the cursor from the last successful poll.
- Switching from service-account to linked-account auth on the same
  `folder_id` preserves the existing cursor.
- A `ConnectionBroken` error does **not** clear the cursor — the source
  can resume where it left off after re-authentication.

---

## 6. `is_revocation_error()` Helper

```rust
fn is_revocation_error(msg: &str) -> bool
```

Checks error messages for known Google revocation patterns:
- `"invalid_grant"`
- `"Token has been revoked"`

Used by `refresh_from_connection()` to distinguish between permanent
revocations (→ `ConnectionBroken`) and transient failures (→ generic
`SourceError::Fetch`).

---

## 7. Module Structure

After Session 04, the Google Drive provider is a module directory:

```
source/
  google_drive/
    mod.rs    (441 lines) — DriveAuthStrategy, GoogleDriveProvider, constructors, auth dispatch
    jwt.rs    (354 lines) — Legacy service-account JWT signing (RSA, DER parsing)
```

The split keeps both files under the 500-line limit and isolates the
legacy RSA crypto from the new connection-based auth path.
