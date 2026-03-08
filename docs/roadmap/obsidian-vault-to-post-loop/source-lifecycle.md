# Content Source Lifecycle Contract

## Source States

| State      | Meaning                                      |
|------------|----------------------------------------------|
| `active`   | Source is registered and last sync succeeded  |
| `syncing`  | Scan or poll in progress                     |
| `error`    | Last sync failed (see `error_message`)       |
| `disabled` | Source exists but is not participating        |

State transitions happen at:
- **Startup / initial scan**: `syncing` -> `active` or `error`
- **Fallback poll tick**: `syncing` -> `active` or `error`
- **Remote poll**: `syncing` -> `active` or `error`
- **Reindex API**: `syncing` -> `active` or `error`
- **Connection broken** (Google Drive): -> `error`

## Configuration Fields

### `enabled` (Option\<bool\>, default: `None`)

Whether this source participates in ingestion at all.

When `None`, falls back to the legacy `watch` field. When `Some(false)`,
the source is completely skipped during Watchtower startup and polling.

### `change_detection` (String, default: `"auto"`)

How changes are detected for an enabled source:

| Value    | Local FS behavior                          | Google Drive behavior  |
|----------|--------------------------------------------|------------------------|
| `"auto"` | notify watcher + fallback poll (5 min)     | interval poll          |
| `"poll"` | fallback poll only (no notify watcher)     | interval poll          |
| `"none"` | initial scan only, no ongoing monitoring   | initial poll only      |

### `watch` (bool, default: `true`) — DEPRECATED

Legacy field. When `enabled` is `None`, the value of `watch` is used as
the effective enabled state. New configurations should use `enabled` and
`change_detection` instead.

**Migration path**: `watch: true` is equivalent to `enabled: true, change_detection: "auto"`.
`watch: false` is equivalent to `enabled: false`.

## Local vs Remote Behavior Matrix

| Source Type    | `enabled` | `change_detection` | Initial Scan | Notify Watcher | Fallback Poll | Notes                    |
|----------------|----------|--------------------|--------------|-----------|-----------|-------------------------------------------------|
| `local_fs`     | true     | `auto`             | Yes          | Yes       | Yes       | Full real-time + fallback                        |
| `local_fs`     | true     | `poll`             | Yes          | No        | Yes       | Good for NFS / network mounts                    |
| `local_fs`     | true     | `none`             | Yes          | No        | No        | One-shot import                                  |
| `local_fs`     | false    | (any)              | No           | No        | No        | Completely skipped                                |
| `google_drive` | true     | `auto`/`poll`      | Yes          | N/A       | Yes       | Always polls (no real-time API)                  |
| `google_drive` | true     | `none`             | Yes          | N/A       | No        | One-shot import                                  |
| `google_drive` | false    | (any)              | No           | N/A       | No        | Completely skipped                                |
| `manual`       | N/A      | N/A                | N/A          | N/A       | N/A       | Inline ingest via POST /api/ingest               |

## Config Reload Behavior

When `PATCH /api/settings` modifies `content_sources` or `deployment_mode`
for the default account:

1. The current Watchtower loop is cancelled via its `CancellationToken`.
2. Config is reloaded from disk.
3. A new `WatchtowerLoop` is spawned with the updated sources.
4. The new loop performs initial scans for all newly enabled sources.

This is a cancel-and-respawn pattern. In-flight scan iterations complete
before the cancellation takes effect (cooperative cancellation).

## Validation Rules

- `change_detection` must be one of: `"auto"`, `"poll"`, `"none"`.
- `poll_interval_seconds` must be >= 30 when set.
- Enabled `local_fs` sources must have a non-empty `path`.
- Enabled `google_drive` sources must have a non-empty `folder_id`.
- Source types are validated against deployment mode capabilities.

## API Endpoints

### `GET /api/sources/status`

Returns all registered source contexts with runtime status.

### `POST /api/sources/{id}/reindex`

Triggers a full rescan of a local_fs source. Returns immediately with
`{ "status": "reindex_started", "source_id": <id> }`. The rescan runs
in a background task. Source status transitions to `"syncing"` during
the reindex and `"active"` or `"error"` on completion.

## Migration from `watch` to `enabled`/`change_detection`

Existing configs using `watch: true` continue working with no changes.
The `is_enabled()` method falls back to `watch` when `enabled` is `None`.

To adopt the new fields:
```toml
# Before (legacy):
[[content_sources.sources]]
source_type = "local_fs"
path = "~/vault"
watch = true

# After (explicit):
[[content_sources.sources]]
source_type = "local_fs"
path = "~/vault"
enabled = true
change_detection = "auto"
```

The `watch` field is not removed to maintain backward compatibility.
When both `enabled` and `watch` are set, `enabled` takes precedence.
