# Draft Studio — Autosave and Sync Semantics

## Save Pipeline

When a user edits a draft in the Draft Studio, changes flow through a two-tier save pipeline:

```
Edit → 500ms localStorage (crash recovery) → 1500ms server PATCH (canonical save)
```

### Tier 1: Local crash recovery (500ms debounce)

- Saves the full editor state (mode, text, thread blocks with IDs, media paths) to localStorage under `tuitbot:compose:draft:{draftId}`.
- Exists solely to recover from browser crashes, tab kills, and power failures.
- Cleared on successful server save, draft switch, and normal component teardown.
- Never used as the source of truth for hydration — the server always wins on normal load.

### Tier 2: Server autosave (1500ms debounce)

- PATCHes `content` and `content_type` to `PATCH /api/drafts/:id` with the client's last-known `updated_at` for conflict detection.
- On success: updates the `DraftSaveManager`'s internal `lastServerUpdatedAt` and sets sync status to `saved`.
- On 409 (stale_write): sets sync status to `conflict`, shows conflict resolution UI.
- On network error: sets sync status to `offline`, re-queues the payload for retry on the next edit.

## DraftSaveManager Lifecycle

Each selected draft gets its own `DraftSaveManager` instance, created when the shell fetches and hydrates a draft. The lifecycle:

1. **Create**: `new DraftSaveManager(draftId, serverUpdatedAt, onSyncStatus)` — on draft selection.
2. **Save**: `.save(mode, tweetText, blocks, media)` — on every content change.
3. **Flush**: `.flush()` — called before submit or on `beforeunload`.
4. **Destroy**: `.destroy()` — on draft switch or component teardown. Flushes pending save, clears draft-scoped localStorage, sets `destroyed` flag to discard in-flight responses.

The `{#key draftId}` pattern on `ComposeWorkspace` in the shell ensures full component destruction and recreation on draft switch, preventing any cross-draft state leakage.

## Conflict Detection

The server uses `updated_at` comparison for optimistic concurrency:

- Client sends `{ content, content_type, updated_at }` where `updated_at` is the last-known server timestamp.
- Server compares against the DB row's `updated_at`.
- **Match**: update succeeds → `200 { id, updated_at }`.
- **Mismatch**: stale write → `409 { error: "stale_write", server_updated_at }`.

### Conflict Resolution

When a 409 occurs, the `DraftSyncBadge` shows two options:

- **Use mine**: Re-fetches the draft to get the current `updated_at`, then the manager retries the PATCH with updated timestamp. The user's local content overwrites the server version.
- **Reload server**: Re-fetches the draft and re-hydrates the composer, discarding local changes. Triggers a `{#key}` remount.

## Recovery Precedence

On draft selection (hydration):

1. Fetch full draft from server via `GET /api/drafts/:id`.
2. Parse content into composer format (tweet text or thread blocks).
3. Check for crash recovery data in `localStorage[tuitbot:compose:draft:{id}]`.
4. **If local timestamp > server `updated_at`**: show recovery banner (user chose to recover or dismiss).
5. **Otherwise**: discard local data, hydrate from server.

This ensures the server is always the canonical source unless there's evidence of a crash with unsaved local changes.

## Sync Status States

| Status | Meaning | UI |
|--------|---------|----|
| `saved` | Last edit successfully saved to server | Checkmark, subtle |
| `saving` | PATCH in flight | Spinner |
| `unsaved` | Edits pending, not yet sent | Dot, warning color |
| `offline` | Server unreachable | Cloud-off icon, warning |
| `conflict` | 409 stale_write received | Alert icon, action buttons |

## Content Serialization

The PATCH autosave sends content in the same format as `buildComposeRequest`:

- **Tweet mode**: `tweetText.trim()`
- **Thread mode**: `JSON.stringify(validBlocks.map(b => b.text))` — JSON array of strings

The localStorage crash recovery stores the full `AutosavePayload` (with `ThreadBlock[]` including IDs, media_paths, order) for complete editor state restoration.

## Cross-Draft Isolation

Switching drafts uses `{#key hydrationDraftId}` on `ComposeWorkspace`, which triggers Svelte to:

1. Call `onDestroy` on the old instance (flushes pending save, destroys manager).
2. Fully unmount the old component tree (clears all local state, timers, undo snapshots).
3. Mount a fresh instance with new props (creates new manager, hydrates from new draft).

This eliminates any risk of state leakage between drafts.

## Failure Handling

| Failure | Behavior |
|---------|----------|
| Network error on autosave | Status → `offline`, payload re-queued, retried on next edit |
| 409 stale_write | Status → `conflict`, user-facing resolution buttons |
| Network error on draft fetch | Error state shown with retry button |
| localStorage quota exceeded | Silently ignored (server save is the canonical path) |
| JSON parse error on thread content | Falls back to treating content as single tweet text |
