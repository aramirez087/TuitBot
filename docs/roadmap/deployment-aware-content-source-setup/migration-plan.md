# Migration Plan -- Deployment-Aware Content Source Setup

This document covers the upgrade path for existing Tuitbot installs to the
new deployment-aware content source model introduced in Sessions 01-06.

---

## Overview

The deployment-aware model adds three new config sections:

| Section | Purpose |
|---------|---------|
| `deployment_mode` | Declares desktop / self_host / cloud context |
| `[connectors.google_drive]` | OAuth credentials for linked-account Drive flow |
| `[content_sources]` | Content source entries (local_fs, google_drive) |

Existing installs may have none, some, or all of these. The upgrade path is
designed to be non-breaking: every existing config continues to work without
modification.

---

## Scenario Matrix

### 1. Desktop user with `local_fs` source

**Current state:** `[[content_sources.sources]]` with `source_type = "local_fs"` and a `path`.

**What happens on upgrade:**
- `deployment_mode` defaults to `"desktop"` (the default). No change needed.
- `local_fs` source continues to work identically.
- `tuitbot update` offers to add `[connectors.google_drive]` scaffold and
  `deployment_mode` key, but does not modify existing sources.
- The dashboard shows "Obsidian Vault / Notes Folder" as the primary option.

**Action required:** None.

### 2. Self-hosted user with `service_account_key` Drive source

**Current state:** `[[content_sources.sources]]` with `source_type = "google_drive"`,
`folder_id`, and `service_account_key`.

**What happens on upgrade:**
- Source continues to work. Watchtower uses `service_account_key` as auth strategy.
- `tuitbot update` detects missing `deployment_mode`, `[connectors]`, and prints
  a tip about switching to a linked account.
- The dashboard Settings page shows a "Legacy SA key" notice with a
  "Connect Google Drive" button to upgrade.

**Action required:** None (optional migration to linked account).

**Optional migration steps:**
1. Add `deployment_mode = "self_host"` to config (or set env var).
2. Add `[connectors.google_drive]` with GCP OAuth credentials.
3. Open dashboard > Settings > Content Sources > Connect Google Drive.
4. Complete OAuth. The dashboard adds `connection_id` to the source entry.
5. `connection_id` takes precedence over `service_account_key` at runtime.
6. Optionally remove `service_account_key` from config after verifying.

### 3. Self-hosted user with manual `local_fs` path

**Current state:** `[[content_sources.sources]]` with `source_type = "local_fs"` and
a server-side `path`.

**What happens on upgrade:**
- Source continues to work identically.
- `tuitbot update` offers to add `deployment_mode` and connector scaffold.
- Dashboard shows local folder under "Advanced: Local Server Folder" toggle
  in self_host mode.

**Action required:** None.

### 4. Cloud user with `service_account_key` Drive source

**Current state:** Same as Scenario 2 but deployed on managed infrastructure.

**What happens on upgrade:**
- Same as Scenario 2. Source continues to work.
- Setting `deployment_mode = "cloud"` disables `local_fs` sources in
  validation, but existing `google_drive` + SA key sources are unaffected.
- Migration to linked account follows the same steps as Scenario 2.

**Action required:** Set `deployment_mode = "cloud"` (or env var) if desired.

### 5. Fresh install (no existing config)

**What happens:**
- `tuitbot init` or the onboarding wizard creates config with
  `deployment_mode`, `[connectors]`, and `[content_sources]` from the start.
- Desktop onboarding defaults to local folder source.
- Self-host/cloud onboarding defaults to Google Drive connection flow.

**Action required:** Follow the onboarding wizard.

### 6. Pre-content-sources install (no `[content_sources]` section)

**Current state:** Config from before content sources were added (pre-Session 01).

**What happens on upgrade:**
- `tuitbot update` detects missing `content_sources`, `deployment_mode`, and
  `connectors` sections.
- Interactive mode: prompts for deployment mode, connector credentials (or skip),
  and prints a notice about configuring sources in the dashboard.
- Non-interactive mode: adds empty scaffolds with safe defaults.

**Action required:** Run `tuitbot update` (or `tuitbot update --config-only`).

---

## What `tuitbot update` Does

The `tuitbot update` command (Phase 2: config upgrade) detects and patches
missing feature groups:

| Group | Detection Key | Interactive | Non-interactive |
|-------|--------------|-------------|-----------------|
| DeploymentMode | `deployment_mode` absent | Prompt: Desktop/SelfHost/Cloud | Default: `"desktop"` (or `TUITBOT_DEPLOYMENT_MODE` env) |
| Connectors | `connectors.google_drive.client_id` absent | Prompt: client_id + client_secret (or skip) | Add empty scaffold |
| ContentSources | `content_sources` absent | Print notice: configure in dashboard | Add empty `[content_sources]` |

Legacy SA-key notice: After processing groups, if any `service_account_key`
entries exist without a sibling `connection_id`, a non-blocking tip is printed.

---

## Rollback

If an upgrade causes issues:

1. The upgrade wizard creates a `.toml.bak` backup before any changes.
2. Restore: `cp ~/.tuitbot/config.toml.bak ~/.tuitbot/config.toml`
3. All new fields are optional with safe defaults. Removing them entirely
   returns to pre-upgrade behavior.
4. `connection_id` entries can be removed to fall back to `service_account_key`.
5. `deployment_mode` can be removed to default back to `"desktop"`.

---

## Environment Variables for Non-Interactive Environments

Docker and CI users who rely on `--non-interactive` can configure the new
sections entirely through environment variables:

```bash
# Deployment mode
TUITBOT_DEPLOYMENT_MODE=self_host

# Google Drive connector credentials
TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_ID=...
TUITBOT_CONNECTORS__GOOGLE_DRIVE__CLIENT_SECRET=...
TUITBOT_CONNECTORS__GOOGLE_DRIVE__REDIRECT_URI=http://myhost:3001/api/connectors/google-drive/callback
```

Content sources are configured via the dashboard or manual TOML editing.
There is no env-var path for individual source entries (they are structured
arrays in TOML).

---

## Security Notes

- `service_account_key` values are **redacted** in `GET /api/settings` responses
  (returned as `"[redacted]"`).
- `connection_id` is a numeric reference; the actual refresh token is stored
  encrypted in the database and never exposed via API or logs.
- Connector `client_secret` values are **not** returned by the settings API.
- The OAuth callback validates `state` parameter to prevent CSRF.
- `postMessage` origin is validated against the server's base URL.
