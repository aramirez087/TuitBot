# Session 06 Handoff

## What Changed

### `DraftRailItem.svelte` (new, replaces `DraftRailEntry.svelte`)
- Roving tabindex support via `tabindex` prop (0 or -1)
- `focused` class for visual focus ring (`box-shadow: inset 0 0 0 1.5px var(--color-accent)`)
- Quick action buttons: Duplicate (Copy), Archive (Archive) on active/scheduled tabs; Restore (RotateCcw) on archive tab
- Actions visible on hover and keyboard focus, hidden otherwise
- Exposes `focus()` and `scrollIntoViewIfNeeded()` methods for parent control
- `role="option"` with `aria-selected` for listbox pattern

### `DraftRail.svelte` (major rewrite)
- Four tabs: Drafts, Scheduled, Posted, Archive with live counts
- `role="listbox"` container with `keydown` handler for full keyboard navigation
- Arrow keys move focus ring; Enter selects; N creates; Delete/Backspace archives; D duplicates; R restores (archive tab)
- Number keys 1-4 switch tabs
- Undo toast at bottom of rail after archive (5s auto-dismiss, click to restore)
- `focusedIndex` state clamped on list changes, reset on tab switches
- Exports `focus()` method for shell to call on Escape from composer
- New callback props: `onarchive`, `onduplicate`, `onrestore`

### `DraftStudioShell.svelte` (extended)
- Wires `onarchive`, `onduplicate`, `onrestore` callbacks to studio store
- Shell-level `Escape` handler: returns focus from composer zone to rail
- Defines `draftStudioPaletteActions` array (New, Duplicate, Archive, Jump to rail) passed to `ComposeWorkspace` via `extraPaletteActions`
- `ondraftaction` callback handles palette-invoked draft actions
- `bind:this` on `DraftRail` for focus management

### `CommandPalette.svelte` (extended)
- New `extraActions` prop merged into the action list
- `PaletteAction` interface exported for use by other components
- Category union expanded: `'DraftStudio'` added to `categoryOrder`

### `ComposeWorkspace.svelte` (minimal touch)
- New props: `extraPaletteActions` (forwarded to CommandPalette), `ondraftaction` (callback for unhandled action IDs)
- `handlePaletteAction` default case forwards unknown IDs to `ondraftaction`

### `shortcuts.ts` (extended)
- `ShortcutDef.category` union includes `'DraftStudio'`
- `formatCombo` handles `backspace` (⌫) and `delete` (Del) keys
- Five new entries in `SHORTCUT_CATALOG` for draft studio shortcuts

### `draftStudio.svelte.ts` (extended)
- Tab type expanded to `'active' | 'scheduled' | 'posted' | 'archive'`
- `postedDrafts` derived (filters `status === 'posted'`)
- `currentTabDrafts` handles the `'posted'` case
- `tabCounts` includes `posted`

### `DraftRailEntry.svelte` (deleted)
Fully replaced by `DraftRailItem.svelte`.

### Documentation
- `rail-interactions.md`: keyboard map, focus model, undo flow, quick action rules, accessibility notes

## Key Decisions

1. **Roving tabindex over `aria-activedescendant`**: Roving tabindex gives natural `Tab` escape from the rail to the composer. `aria-activedescendant` would require the container to hold focus, making Tab behavior ambiguous.

2. **`focusedIndex` lives in DraftRail, not the store**: Focus state is ephemeral UI state. It doesn't need to survive navigation or be shared. The store's `selectedId` (which draft is open) is separate from `focusedIndex` (which item has keyboard focus).

3. **Posted tab shows empty state**: The list API may not return `status: 'posted'` items in the current collection fetch. The tab shows "Posted drafts appear here" until the server-side filter is wired (Session 07).

4. **No bulk archive**: Multi-select (checkboxes, shift-click ranges) is significant UX. Deferred to a later session. Single-item keyboard archive covers the 80% case.

5. **Undo toast is rail-local**: No global toast system. The `lastArchived` state in `DraftRail` holds the undo context. Auto-clears on tab switch or timeout.

6. **Command palette actions use `ds-` prefix**: Action IDs (`ds-new-draft`, `ds-archive`, etc.) avoid collision with existing composer actions. The workspace forwards unknown IDs to `ondraftaction`.

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 07 | Metadata and filters | Title editing, tags UI, search, content type filter, sort options, right details panel |
| 08 | Schedule and calendar | Schedule/unschedule from studio, calendar integration |
| 09 | Revisions and activity | Revision timeline, preview/restore, activity log |
| 10 | Entrypoints and rollout | Home redirect, calendar redirect, feature flag, mobile layout |
| 11 | Validation and launch | QA, accessibility, mobile, performance, docs |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Posted tab empty indefinitely | High | Low | Server needs a `status=posted` filter or the list API needs to return posted items. Flag for Session 07. |
| `bind:this` on `DraftRailItem` in `{#each}` array | Low | Medium | Svelte 5 supports array `bind:this` in `{#each}`. Verified compiles clean. |
| Bare letter shortcuts (N, D, R) conflict with text input | Low | Medium | Only active when rail list has focus. When the composer textarea has focus, these are normal text input. |
| ComposeWorkspace now at ~925 lines | High | Low | Added ~5 lines (2 props, 3 lines in switch). No action needed. |
| Undo toast doesn't persist across tab switches | Low | Low | By design — tab switch commits the archive. User can switch to Archive tab to restore manually. |

## Exact Inputs for Session 07

1. Read `dashboard/src/lib/components/drafts/DraftRail.svelte` — search/filter integration target
2. Read `dashboard/src/lib/components/drafts/DraftRailItem.svelte` — title editing target
3. Read `dashboard/src/lib/stores/draftStudio.svelte.ts` — search/sort/filter state
4. Read `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` — right details panel context
5. Implement search input in the rail (filter `currentTabDrafts` by title/content)
6. Add sort options (updated, created, alphabetical)
7. Add inline title editing on `DraftRailItem`
8. Add right-side details panel (metadata, tags)
9. Wire posted tab server-side filter if API supports it
10. Run: `npm --prefix dashboard run check`
