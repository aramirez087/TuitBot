# Entry Points & Rollout — Draft Studio

## Entry Point Matrix

Every surface that initiates writing now flows through Draft Studio.

| Surface | Before Session 10 | After Session 10 |
|---------|-------------------|------------------|
| **Home page** (`/`) | Embedded `ComposeWorkspace` calling `api.content.compose()` — no server-backed drafts, no revision history | `DraftStudioQuickStart` card with "New Draft" and recent drafts list. Creates server-backed draft, navigates to `/drafts?id={id}` |
| **`Cmd+N` shortcut** | Navigated to `/` and dispatched `tuitbot:compose` event to focus the home composer | Navigates to `/drafts?new=true`. DraftStudioShell reads the param, creates a draft, cleans the URL |
| **Calendar "Compose" button** | Opened `ComposeModal` inline — created `scheduled_content` directly via `composeContent()` | Creates a server-backed draft via `api.draftStudio.create()`, navigates to `/drafts?id={id}` |
| **Calendar time-slot click** | Opened `ComposeModal` with prefilled date/time | Creates a draft, navigates to `/drafts?id={id}&prefill_schedule={iso}`. Shell parses param, pre-populates schedule pickers |
| **Calendar day click (month view)** | Opened `ComposeModal` with prefilled date | Creates a draft, navigates to `/drafts?id={id}` |
| **Calendar `?compose=true` (onboarding)** | Opened `ComposeModal` | Creates a draft, navigates to `/drafts?id={id}` |
| **Calendar empty state** | Opened `ComposeModal` | Creates a draft, navigates to `/drafts` |
| **Draft rail "+" button** | Already in Draft Studio — creates draft via `studio.createDraft()` | Unchanged |
| **Sidebar "Draft Studio" link** | Navigates to `/drafts` | Unchanged |
| **Settings "Default Landing Page"** | Options: "Composer home" / "Analytics overview" | Options: "Draft Studio" / "Analytics overview". Legacy `'composer'` preference auto-migrated to `'drafts'` |

## Rollout Strategy

### All-or-nothing within this branch

There is no feature flag or gradual rollout. When `feat/composer_improvements` merges to `main`, all entry points switch simultaneously. This is appropriate because:

1. The old and new flows are mutually exclusive — a half-migrated state would confuse users more than a clean switch
2. The Draft Studio API and data model have been stable since Session 3
3. The `api.content.compose()` endpoint still exists and works — MCP tools and external API consumers are unaffected

### Preference Migration

The `homeSurface` store handles the `'composer'` → `'drafts'` migration transparently:
- On load, if the persisted value is `'composer'`, it's remapped to `'drafts'` and re-persisted
- No user action required; the setting page shows "Draft Studio" selected

### API Compatibility

| Endpoint | Status |
|----------|--------|
| `POST /api/content/compose` | Still works. Not called by the dashboard but preserved for MCP tools and API consumers |
| `POST /api/drafts` | Primary creation path for all dashboard flows |
| `PATCH /api/drafts/{id}` | Autosave endpoint |
| `POST /api/drafts/{id}/schedule` | Scheduling from details panel |
| All other draft endpoints | Unchanged |

## Telemetry

Structured `console.info('[draft-studio]', { event, ...data })` calls at key lifecycle points:

| Event | Payload | Source locations |
|-------|---------|-----------------|
| `draft_created` | `{ id, source }` — source is `home-quickstart`, `calendar-slot`, `calendar-button`, `calendar-day`, `calendar-empty`, `onboarding-redirect`, `cmd-n`, `rail-button` | QuickStart, calendar page, Shell |
| `draft_selected` | `{ id, source }` — source is `fetch`, `home-resume` | Shell, QuickStart |
| `save_failed` | `{ id, syncStatus }` | Shell |
| `restore_executed` | `{ id, revisionId }` | Shell |
| `transition` | `{ id, from, to }` — e.g. `draft→scheduled` | Shell |

These events are visible in browser devtools and Tauri's console. No external telemetry service is used.

## Removed Components

| Component | Action | Reason |
|-----------|--------|--------|
| `ComposeModal.svelte` | Deleted (Session 11) | Zero imports after calendar migration. Calendar creates drafts and redirects. |
| `ComposerPromptCard.svelte` | Deleted (Session 11) | Only used by `ComposeWorkspace` in `embedded && !draftId` path, which is now dead |
| `ComposerTipsTray.svelte` | Deleted (Session 11) | Same as above |
| `ComposerShortcutBar.svelte` | Deleted (Session 11) | Zero imports found |
| `ComposeWorkspace` (home usage) | Import removed from `(app)/+page.svelte` (Session 10) | Replaced by `DraftStudioQuickStart` |
| `composeContent` import | Removed from calendar store imports (Session 10) | No longer called from calendar page |

## Session 11 Cleanup (Completed)

All deferred cleanup items have been resolved:

- Deleted `ComposeModal.svelte` — zero remaining imports
- Deleted `ComposerPromptCard.svelte`, `ComposerShortcutBar.svelte`, `ComposerTipsTray.svelte`
- Removed `tuitbot:compose` event listener/handler from `ComposeWorkspace.svelte`
- Removed dead code paths in `ComposeWorkspace` (`tipsVisible`, `promptDismissed`, `showPromptCard`, `dismissTips`, `handleUseExample`)
- `prefill_schedule` URL param handled: Shell parses and validates the ISO string, passes to `DraftScheduleSection` which pre-populates date/time pickers
- Extracted `DraftDetailsPanel` (943 → 156 lines) into 5 sub-components
