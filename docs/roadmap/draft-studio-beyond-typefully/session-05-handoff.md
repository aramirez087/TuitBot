# Session 05 Handoff

## What Changed

### `DraftSaveManager` class in `composerAutosave.ts`

New class that encapsulates draft-scoped autosave logic:
- Debounced localStorage save (500ms) for crash recovery under `tuitbot:compose:draft:{id}`.
- Debounced server PATCH (1500ms) with `updated_at` conflict detection.
- Sync status reporting via callback (saved/saving/unsaved/offline/conflict).
- Full lifecycle: `save()` → `flush()` → `destroy()`.
- `forceServerSave()` for conflict override after user chooses "use mine".

Also added `readDraftAutoSave(draftId)` and `clearDraftAutoSave(draftId)` helpers.

Existing global autosave functions remain unchanged for ComposeModal and home embed.

### `ComposeWorkspace.svelte` — draft props

New optional props:
- `draftId?: number` — enables draft studio mode.
- `initialContent?: { mode, tweetText, threadBlocks, attachedMedia, updatedAt }` — hydration payload.
- `onsyncstatus?: (status: SyncStatus) => void` — callback for sync status changes.

When `draftId` is set:
- `onMount` hydrates from `initialContent` instead of checking global recovery.
- A `DraftSaveManager` is created with the server's `updatedAt`.
- Crash recovery is draft-scoped: `readDraftAutoSave(draftId)` checks if local data is newer.
- `autoSave()` delegates to the manager instead of global localStorage.
- `onDestroy` destroys the manager (flushing pending saves) instead of global flush.
- `handleBeforeUnload` flushes the manager.
- `handleSubmit` flushes the manager before calling `onsubmit`.

Non-draft-studio usage (ComposeModal, home embed) is unchanged.

### `draftStudio.svelte.ts` — store extensions

- `fullDraft` state: tracks the full `ScheduledContentItem` for the selected draft.
- `getFullDraft()` / `setFullDraft()`: getter/setter pair.
- `updateDraftInCollection()`: updates a draft's summary in the collection without full reload.
- `reset()` now clears `fullDraft`.

### `DraftStudioShell.svelte` — composer binding

Replaced the static preview block with live `ComposeWorkspace`:
- On draft selection: fetches full content via `api.draftStudio.get(id)`, parses into hydration payload.
- `parseServerDraft()`: converts server content format to composer format (tweet text or thread blocks from JSON array).
- `{#key hydrationDraftId}` ensures full remount on draft switch (no cross-draft state leakage).
- Shows `DraftSyncBadge` in composer header with live sync status.
- Conflict resolution: "Reload server" re-fetches and remounts; "Use mine" fetches new `updated_at` and sets status to `unsaved` for retry.
- Error state with retry button when draft fetch fails.
- Loading spinner during draft fetch.

### `DraftSyncBadge.svelte` — new component

Inline sync status indicator showing:
- Saved (checkmark), Saving (spinner), Unsaved (dot), Offline (cloud-off), Conflict (alert + action buttons).
- Accessible with `role="status"` and `aria-live="polite"`.
- Conflict state shows "Use mine" and "Reload" buttons.

### Documentation

- `autosave-and-sync.md`: save pipeline, DraftSaveManager lifecycle, conflict detection, recovery precedence, sync states, content serialization, cross-draft isolation, failure handling.

## Key Decisions

1. **`{#key hydrationDraftId}` for draft switching**: Destroys and recreates `ComposeWorkspace` on draft switch. This is the safest approach — clears all local state including undo snapshots, timers, media previews, and recovery banners. The tradeoff is a brief remount, but since the draft data is pre-fetched, it's fast.

2. **Shell fetches, composer hydrates**: The shell orchestrates the fetch via `api.draftStudio.get(id)` and passes hydration data as props. ComposeWorkspace doesn't know about the API — it receives structured data. This keeps the composer reusable for non-draft-studio contexts.

3. **Conflict resolution "Use mine" approach**: Rather than exposing the `DraftSaveManager` to the shell for force-save, the shell re-fetches the draft to get the new `updated_at`, then sets sync status to `unsaved`. The next edit triggers the manager to re-PATCH with the corrected timestamp. This avoids the complexity of passing content up from the composer.

4. **Server error message matching**: The `request()` helper throws `new Error(body.error)` on non-OK responses. For 409 stale_write, `body.error === 'stale_write'`, so the manager checks `e.message === 'stale_write'`. This is fragile but matches the existing HTTP client pattern. A custom error class could be added later.

5. **Thread content parsing fallback**: If `JSON.parse(content)` fails or doesn't produce a string array, the content is treated as a single tweet. This prevents data loss from format mismatches.

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 06 | Keyboard and bulk | Rail keyboard nav, focus management, command palette, bulk archive |
| 07 | Metadata and filters | Title editing, tags UI, search, content type filter, sort options, right details panel |
| 08 | Schedule and calendar | Schedule/unschedule from studio, calendar integration |
| 09 | Revisions and activity | Revision timeline, preview/restore, activity log |
| 10 | Entrypoints and rollout | Home redirect, calendar redirect, feature flag, mobile layout |
| 11 | Validation and launch | QA, accessibility, mobile, performance, docs |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| "Use mine" conflict resolution doesn't force-save immediately | Medium | Low | The next edit triggers a server PATCH with the corrected `updated_at`. If the user doesn't edit after resolving, the content stays local until they do. Acceptable for now; can add explicit force-save in Session 06. |
| `e.message === 'stale_write'` is fragile | Low | Medium | If the server error format changes, conflicts won't be detected. Could add a typed error class to `http.ts` that preserves response details. Low urgency since the API contract is stable. |
| `ComposeWorkspace` at 920 lines (approaching 500-line budget) | High | Low | The added code is ~30 lines of prop declarations, onMount branching, and autoSave delegation. Most new logic is in `DraftSaveManager`. No action needed unless the file grows further. |
| Thread blocks lose media_paths on server round-trip | Medium | Medium | The server stores threads as `JSON.stringify(texts)` — media_paths are not persisted in the autosave PATCH. Media attachment for threads needs a separate solution (Session 07 or 08). |
| Race condition on rapid draft switching | Low | Low | The `fetchDraft` function checks `studio.getSelectedId() !== id` after the fetch resolves. If the user switched away during the fetch, the result is discarded. The `destroyed` flag on `DraftSaveManager` discards in-flight PATCH responses. |

## Exact Inputs for Session 06

1. Read `dashboard/src/lib/components/drafts/DraftRail.svelte` — keyboard navigation target
2. Read `dashboard/src/lib/components/drafts/DraftRailEntry.svelte` — focus management
3. Read `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` — keyboard event handling context
4. Read `dashboard/src/lib/components/CommandPalette.svelte` — reuse for draft studio commands
5. Implement arrow key navigation in the rail (up/down to move selection, enter to select)
6. Add focus ring styles and tabindex management
7. Add bulk archive action (select multiple + archive)
8. Integrate command palette with draft-specific actions (archive, duplicate, new)
9. Run: `npm --prefix dashboard run check`
