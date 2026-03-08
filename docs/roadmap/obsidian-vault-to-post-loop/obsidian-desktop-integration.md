# Obsidian Desktop Integration

Documents how the Tuitbot desktop app deep-links into Obsidian for local-vault users.

## Obsidian URI Scheme

Obsidian registers a custom `obsidian://` protocol handler.  Tuitbot constructs URIs of the form:

```
obsidian://open?vault={vault_name}&file={relative_path_without_extension}
```

### Construction logic

1. **Vault name** is derived from the last path component of the configured `local_fs` source path.  For example, `/Users/alice/notes/marketing` yields vault name `marketing`.
2. **Relative path** is the note's `relative_path` from the vault sources API, with the `.md` extension stripped and URL-encoded.
3. A vault-root URI (no file) is used for the "Open vault in Obsidian" action in Settings: `obsidian://open?vault={vault_name}`.

### Utility module

`dashboard/src/lib/utils/obsidianUri.ts` exports:

- `buildObsidianUri(vaultPath, relativePath)` — returns a note-level URI or `null`
- `buildObsidianVaultUri(vaultPath)` — returns a vault-root URI or `null`
- `openExternalUrl(url)` — invokes the Tauri `open_external_url` command; returns `false` outside Tauri

## Platform Guards

All desktop-only behavior uses **runtime detection**, not compile-time guards:

1. **`deploymentMode` store** (`$lib/stores/runtime`) — derived from the server's `/api/runtime/status` response.  Values: `desktop`, `self_host`, `cloud`.
2. **Dynamic `import('@tauri-apps/api/core')`** — wrapped in try/catch.  Outside Tauri the import fails silently and `openExternalUrl` returns `false`.
3. **UI gating** — "Open in Obsidian" buttons only render when `isDesktop && vaultPath` is truthy.

The same frontend bundle runs in all three deployment modes.  No conditional compilation is needed.

## Tauri Command: `open_external_url`

Defined in `dashboard/src-tauri/src/lib.rs`.

### Allowed schemes

Only `obsidian://` and `file://` are permitted.  The URL is parsed by the `url` crate before being passed to the OS.  Any other scheme is rejected with an error.

### OS dispatch

| OS      | Command                              |
|---------|--------------------------------------|
| macOS   | `open <url>`                         |
| Windows | `cmd /C start "" <url>`              |
| Linux   | `xdg-open <url>`                     |

## Where Deep Links Appear

| Location | Trigger | URI type |
|----------|---------|----------|
| Citation chips (composer) | External-link icon per chip | Note-level (`obsidian://open?vault=...&file=...`) |
| Settings > Knowledge Vault | "Open vault in Obsidian" link below path input | Vault-root (`obsidian://open?vault=...`) |

## Server-Side Support

The `GET /api/vault/sources` response includes an optional `path` field on each source item, populated from `config_json` for `local_fs` sources.  This lets the frontend derive the vault name without reading the settings draft.

## Fallback Behavior

| Scenario | Behavior |
|----------|----------|
| Not in Tauri (web/self-host) | Open button hidden entirely |
| Desktop but no vault path configured | Open button hidden |
| Vault name cannot be derived (empty path) | Open button hidden |
| Obsidian not installed | OS shows "no handler" dialog |
| Vault name doesn't match Obsidian vault name | Obsidian opens but can't find vault — shows error |

## Operator Requirements

- **Obsidian must be installed** on the user's machine for `obsidian://` URIs to work.
- **Directory name must match the Obsidian vault name.**  Most users have this naturally since Obsidian names vaults after the root directory.  If they differ, the user must rename the directory or their Obsidian vault.

## Known Limitations

- **No heading-level deep linking.**  Obsidian supports `#heading` anchors in URIs, but our chunk storage does not persist heading anchors.  We link to the note, not the specific section.
- **No vault name override.**  The vault name is always inferred from the directory name.  A future settings field could allow explicit override.
