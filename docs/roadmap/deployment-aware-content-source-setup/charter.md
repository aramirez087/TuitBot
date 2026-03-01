# Deployment-Aware Content Source Setup -- Charter

## Problem Statement

Tuitbot's content-source onboarding treats all deployment modes nearly
identically, creating friction for self-hosted and cloud users:

| # | Problem | Impact |
|---|---------|--------|
| 1 | SelfHost onboarding defaults to `local_fs` and asks for a raw server-side path | Confusing UX -- the path must be valid on the server, not the browser's machine |
| 2 | Google Drive requires a service-account JSON key file on the server filesystem | Complex setup: create GCP project, enable API, create service account, share folder, place key file |
| 3 | No user-account OAuth flow for Google Drive | Cannot "Connect my Google Drive" -- only service-account JWT |
| 4 | No connector abstraction for future remote sources | Adding OneDrive/Dropbox/Notion would require ad-hoc config fields and provider code |
| 5 | `service_account_key` path is stored in TOML and returned in `GET /api/settings` | Key file path exposed in JSON responses |
| 6 | No migration path when the config shape changes | Existing `google_drive` configs with `service_account_key` would break |

## Current State

### Config model (`crates/tuitbot-core/src/config/types.rs`)

- `DeploymentMode` enum: `Desktop` (default), `SelfHost`, `Cloud`.
- `DeploymentCapabilities` derived per mode:
  - **Desktop**: `local_folder=true, manual_local_path=true, google_drive=true, inline_ingest=true, file_picker_native=true`.
  - **SelfHost**: same as Desktop except `file_picker_native=false`.
  - **Cloud**: `local_folder=false, manual_local_path=false, google_drive=true, inline_ingest=true, file_picker_native=false`.
- `ContentSourceEntry` supports two source types: `"local_fs"` and `"google_drive"`.
- Google Drive requires `service_account_key` (filesystem path to JSON key) and `folder_id`.

### Google Drive provider (`crates/tuitbot-core/src/source/google_drive.rs`)

- Authentication is service-account JWT only.
- Reads JSON key from a local path, builds RS256 JWT, exchanges for access token.
- No user-account OAuth flow.
- Custom minimal RSA/PKCS#8 implementation (lines 298-677).
- Token cached in-memory with 60-second expiry margin.

### Watchtower (`crates/tuitbot-core/src/automation/watchtower/mod.rs`)

- Splits configured sources into local (watchable via `notify`) and remote (pollable).
- `GoogleDriveProvider` instantiated with `folder_id` + `service_account_key_path`.
- Both local and remote funnel through `ingest_content()`.
- Fallback polling when `notify` watcher fails.

### Server settings (`crates/tuitbot-server/src/routes/settings.rs`)

- `GET /api/settings/status` returns `deployment_mode` and `capabilities` (unauthenticated).
- `POST /api/settings/init` accepts full JSON config, writes TOML.
- `PATCH /api/settings` merges partial JSON into existing TOML.
- `GET /api/settings` returns the full parsed config as JSON -- including `service_account_key` path.
- No connector/link management endpoints exist.

### Frontend onboarding (`SourcesStep.svelte`, `onboarding.ts`, `+page.svelte`)

- Default `source_type` is `"local_fs"` regardless of deployment mode (hardcoded in `onboarding.ts:50`).
- Cloud mode auto-switches to `"google_drive"` via `$effect` when `local_folder` is false.
- SelfHost users see the same `local_fs`-first flow as Desktop, but without native picker -- they type server-side paths manually.
- Google Drive setup requires pasting `folder_id` and `service_account_key` path.
- Onboarding assembles config and POSTs to `/api/settings/init`.

### Frontend settings (`ContentSourcesSection.svelte`)

- Post-onboarding settings mirror the same two-source UI.
- Same auto-switch logic for cloud.
- Same service-account-key text input for Google Drive.

---

## Design Decisions

### D1: Mode-Specific Onboarding Defaults

| Mode | Primary source in onboarding | Secondary option | Rationale |
|------|------------------------------|------------------|-----------|
| **Desktop** | Local folder (native picker) | Google Drive (linked account) | User's vault is on their machine |
| **SelfHost** | Google Drive (linked account) | Local folder (server-side path, collapsed/advanced) | Server filesystem is not intuitive from a browser |
| **Cloud** | Google Drive (linked account) | Manual ingest only (no local_fs) | No filesystem access |

**Rule:** The `source_type` default in `onboarding.ts` must be derived from a
backend-provided `preferred_source_default` field, not hardcoded to `"local_fs"`.

### D2: Connector Model for Remote Sources

Replace per-source-type config fields with a generic connector abstraction:

```
ContentSourceEntry {
    source_type: String,       // "local_fs", "google_drive", future types
    connection_id: Option<i64>, // references `connections` table (remote sources only)
    path: Option<String>,      // local_fs only
    folder_id: Option<String>, // google_drive locator
    // ... existing fields preserved
}
```

A new `connections` table in SQLite:

| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | Auto-increment |
| `connector_type` | TEXT | `"google_drive"`, future: `"onedrive"`, `"dropbox"`, `"notion"` |
| `account_email` | TEXT | Display only, for UI |
| `encrypted_credentials` | BLOB | Refresh token, encrypted at rest |
| `status` | TEXT | `"active"`, `"expired"`, `"revoked"` |
| `created_at` | TEXT | ISO 8601 |
| `updated_at` | TEXT | ISO 8601 |

API responses never include `encrypted_credentials` -- only `account_email`,
`status`, and `connector_type`.

### D3: Google Drive User-Account OAuth Flow

Replace service-account JWT with user-account OAuth 2.0 + PKCE for new installs:

1. Frontend calls `POST /api/connectors/google-drive/link` -- server generates
   `state` + PKCE `code_verifier`, stores them, returns `authorization_url`.
2. User opens URL, grants Drive read-only access, Google redirects to local callback.
3. `GET /api/connectors/google-drive/callback` exchanges authorization code for
   access + refresh tokens.
4. Server encrypts and stores the refresh token, caches access token in memory.
5. Watchtower's `GoogleDriveProvider` uses the stored refresh token instead of a
   service-account key.

This mirrors the existing X API auth pattern (`tuitbot auth` already does OAuth
2.0 PKCE). The default redirect URI is
`http://localhost:3001/api/connectors/google-drive/callback`. LAN mode users can
override this via `TUITBOT_GOOGLE_REDIRECT_URI` env var.

### D4: Legacy Migration Boundaries

| Existing config | Migration path |
|-----------------|---------------|
| `local_fs` with `path` field | **No change** -- fully preserved. Desktop and SelfHost continue to support it. |
| `google_drive` with `service_account_key` + `folder_id` | **Preserved at runtime** -- the old service-account JWT code path remains as a fallback. New installs use linked accounts. |
| `service_account_key` field in TOML | **Not deleted** -- old field accepted during deserialization via `#[serde(default)]`. New UIs never write it. Docs recommend migrating to linked account. |
| New `connection_id` field | Defaults to `None` -- old configs without this field continue to work. |

**Zero-breakage policy:** Existing configs must continue to work without user
intervention after upgrade. A future `tuitbot upgrade-config` CLI command can
assist opt-in migration.

### D5: Security Rules

1. `encrypted_credentials` column never returned in any API response.
2. `service_account_key` file path (legacy) redacted to `"[redacted]"` in
   `GET /api/settings` response body.
3. Refresh tokens encrypted at rest using AES-256-GCM with a per-instance random
   key stored in `~/.tuitbot/connector_key` with `0600` permissions.
4. Access tokens cached in memory only -- never persisted to disk or database.
5. Log messages must not include tokens, keys, or credentials. Use redacting
   wrappers where necessary.

### D6: Capability Reporting Extension

Add `preferred_source_default` to `DeploymentCapabilities`:

```rust
pub struct DeploymentCapabilities {
    // ... existing fields ...
    pub preferred_source_default: String,  // "local_fs" or "google_drive"
}
```

- Desktop returns `"local_fs"`.
- SelfHost and Cloud return `"google_drive"`.

The frontend reads this field to set the initial `source_type` in the onboarding
store, replacing the hardcoded `"local_fs"` default.

---

## Mode-Specific UX Summary

### Desktop Onboarding

```
[Source Type: Local Folder (default)]
  [Browse...] button (native Tauri picker)
  [Watch for changes] toggle (on)
  [Loop back] toggle (on)

  -- or switch to --

[Source Type: Google Drive]
  [Connect Google Drive] button -> OAuth flow
  [Poll for changes] toggle (on)
```

### SelfHost / LAN Onboarding

```
[Source Type: Google Drive (default)]
  [Connect Google Drive] button -> OAuth flow
  [Poll for changes] toggle (on)

  -- expand "Advanced: Local Folder" --

[Source Type: Local Folder]
  Path text input (server-side path)
  [Watch for changes] toggle (on)
  [Loop back] toggle (on)
```

### Cloud Onboarding

```
[Source Type: Google Drive (only option)]
  [Connect Google Drive] button -> OAuth flow
  [Poll for changes] toggle (on)

  Hint: "Local folders are not available in cloud mode"
```

---

## Implementation Slices

### Session 02: Config and Credential Model (Backend)

Files to create/modify:
- `crates/tuitbot-core/src/config/types.rs` -- add `preferred_source_default` to `DeploymentCapabilities`; add `connection_id` to `ContentSourceEntry`; keep `service_account_key` as `Option` for backward compat
- `crates/tuitbot-core/src/config/mod.rs` -- update validation to accept both legacy and new shapes
- `crates/tuitbot-core/src/config/tests.rs` -- test legacy deserialization, new shape, and capability reporting
- `migrations/NNNN_create_connections_table.sql` -- create `connections` table
- `crates/tuitbot-core/src/storage/watchtower/mod.rs` -- add CRUD for `connections` table
- `crates/tuitbot-server/src/routes/settings.rs` -- redact `service_account_key` in GET responses
- `dashboard/src/lib/api.ts` -- add TypeScript types for updated capabilities

Deliverables:
- `docs/roadmap/deployment-aware-content-source-setup/source-connection-contract.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-02-handoff.md`

### Session 03: Drive Connection Backend (Backend)

Files to create/modify:
- `crates/tuitbot-core/src/source/connector/` -- new module: `mod.rs` (trait), `google_drive.rs` (OAuth flow logic)
- `crates/tuitbot-core/src/source/connector/crypto.rs` -- credential encryption/decryption
- `crates/tuitbot-server/src/routes/connectors.rs` -- link, callback, status, disconnect endpoints
- `crates/tuitbot-server/src/routes/mod.rs` -- register connector routes
- `crates/tuitbot-server/src/lib.rs` -- wire routes
- `crates/tuitbot-server/tests/api_tests.rs` -- connector endpoint tests

Deliverables:
- `docs/roadmap/deployment-aware-content-source-setup/drive-connection-flow.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-03-handoff.md`

### Session 04: Watchtower Provider Refactor (Backend)

Files to create/modify:
- `crates/tuitbot-core/src/source/google_drive.rs` -- add constructor from `connection_id`; keep old constructor as legacy fallback
- `crates/tuitbot-core/src/automation/watchtower/mod.rs` -- check for `connection_id` first, fall back to legacy `service_account_key`; add token refresh retry logic
- Tests for both legacy and linked-account code paths

Deliverables:
- `docs/roadmap/deployment-aware-content-source-setup/session-04-handoff.md`

### Session 05: Onboarding and Settings UX (Frontend)

Files to create/modify:
- `dashboard/src/lib/stores/onboarding.ts` -- default `source_type` from `preferred_source_default`
- `dashboard/src/lib/stores/runtime.ts` -- surface `preferred_source_default` from capabilities
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte` -- mode-aware UI: Desktop picker first, SelfHost/Cloud connector first; replace service-account-key input with "Connect" button
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` -- same refactor; show connected account status, disconnect button
- `dashboard/src/lib/api.ts` -- add connector API client methods
- `dashboard/src/routes/onboarding/+page.svelte` -- reference `connection_id` instead of `service_account_key`

Deliverables:
- `docs/roadmap/deployment-aware-content-source-setup/session-05-handoff.md`

### Session 06: Desktop Compatibility and Migration (Backend + Frontend)

Files to create/modify:
- `crates/tuitbot-cli/src/commands/` -- add `upgrade-config` subcommand for legacy Drive config migration
- `docs/configuration.md` -- update content sources section with new connector flow
- `docs/architecture.md` -- update Content Source Pipeline section
- Integration tests for legacy config -> new provider path

Deliverables:
- `docs/roadmap/deployment-aware-content-source-setup/session-06-handoff.md`

### Session 07: Validation and Release Readiness

Verification:
- Full CI: `cargo fmt --all && cargo fmt --all --check`, `RUSTFLAGS="-D warnings" cargo test --workspace`, `cargo clippy --workspace -- -D warnings`
- Dashboard: `cd dashboard && npm run check && npm run build`
- E2E flow: onboarding on each mode, settings edit, Drive connect/disconnect
- Migration: existing config files with legacy shape load correctly

Deliverables:
- `docs/roadmap/deployment-aware-content-source-setup/release-readiness.md`

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Google OAuth requires a registered redirect URI | Document required GCP console setup; provide default redirect URI; allow env var override for LAN |
| Credential encryption adds complexity | Start simple: random key in `~/.tuitbot/connector_key` with `0600` perms, AES-256-GCM via `aes-gcm` crate |
| Custom RSA implementation in google_drive.rs | Keep for legacy service-account path only; new user-account path uses standard `reqwest` POST |
| Large frontend refactor in SourcesStep.svelte | Keep existing local_fs branch intact; add connector flow as a parallel branch |
| Breaking existing configs | Zero-breakage policy: old fields always deserialize, old providers always instantiate, `#[serde(default)]` on all new fields |
