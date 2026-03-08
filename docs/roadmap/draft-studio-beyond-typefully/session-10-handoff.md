# Session 10 Handoff

## What Changed

### Home Page (`(app)/+page.svelte`)
- Removed `ComposeWorkspace` import and the embedded home composer
- Removed `api.content.compose`, `schedule`, `canPublish` state and all related loading logic
- Added `DraftStudioQuickStart` component for the `'drafts'` surface option
- `AnalyticsHome` branch unchanged
- Title changed from "Compose" to "Home"

### New Component: `DraftStudioQuickStart.svelte`
- Lightweight launch pad at `dashboard/src/lib/components/home/DraftStudioQuickStart.svelte`
- "New Draft" button creates a server-backed draft and navigates to `/drafts?id={id}`
- Recent drafts list (up to 5) with click-to-resume
- "Open Draft Studio" link for direct navigation
- Telemetry: `draft_created` (source: `home-quickstart`), `draft_selected` (source: `home-resume`)

### `homeSurface` Store
- Type changed from `'composer' | 'analytics'` to `'drafts' | 'analytics'`
- Default changed from `'composer'` to `'drafts'`
- Migration: if persisted value is `'composer'`, auto-remaps to `'drafts'` and re-persists

### Settings (`WorkspaceSection.svelte`)
- "Composer home" → "Draft Studio"
- "Jump straight into writing" → "Jump straight into your drafts"
- Type references updated from `'composer'` to `'drafts'`

### Layout (`+layout.svelte`)
- `Cmd+N` handler simplified: navigates to `/drafts?new=true` from any page
- Removed `tuitbot:compose` event dispatch and home-page special-casing

### Calendar (`content/+page.svelte`)
- Removed `ComposeModal` import and rendered instance
- Removed `composeOpen`, `composePrefillTime`, `composePrefillDate` state
- Removed `handleCompose` function and `composeContent` import
- All compose actions (`openCompose`, slot clicks, day clicks, empty state, header button, onboarding redirect) now call `createDraftAndRedirect()` which creates a server-backed draft and navigates to `/drafts?id={id}`
- Time-slot context preserved via `prefill_schedule` URL param
- Button label changed from "Compose" to "New Draft"
- Empty state description updated to reference Draft Studio

### DraftStudioShell
- Handles `?new=true` URL param on mount: creates a draft, cleans the URL
- Added structured telemetry (`console.info('[draft-studio]', ...)`) at 6 lifecycle points:
  - `draft_created` (sources: cmd-n, rail-button)
  - `draft_selected` (source: fetch)
  - `save_failed` (on offline status)
  - `restore_executed` (on revision restore)
  - `transition` (draft↔scheduled)

### Documentation
- `docs/composer-mode.md`: Updated intro, Auto-Save section (server-backed), Drafts section (now "Draft Studio"), troubleshooting tables
- `docs/roadmap/draft-studio-beyond-typefully/entrypoints-and-rollout.md`: Entry point matrix, rollout strategy, API compatibility, telemetry catalog, deferred cleanup list

## Key Decisions

1. **QuickStart card instead of redirect**: Home page shows a launch pad with "New Draft" + recent drafts instead of a hard redirect to `/drafts`. This preserves the analytics-home option and avoids a jarring redirect for users who land on `/`.

2. **Calendar creates drafts + redirects**: Calendar compose actions no longer open an inline modal. They create a server-backed draft and redirect to Draft Studio. This is a UX change (one extra navigation step) but ensures every draft gets autosave, revision history, and the full keyboard-first experience. Time-slot context is preserved via `prefill_schedule` URL param.

3. **No feature flag**: All entry points switch simultaneously. The old `api.content.compose()` endpoint is preserved for MCP tools and API consumers but is no longer called by the dashboard.

4. **Telemetry via console.info**: Structured JSON events logged to browser/Tauri console. No external service dependency. Easy to grep in devtools.

5. **`prefill_schedule` param deferred**: The calendar passes a `prefill_schedule` URL param to Draft Studio, but the Shell doesn't yet read it to auto-populate the schedule picker. This is deferred to Session 11 since the details panel's schedule picker is already functional and users can set the time manually.

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 11 | Validation and launch | QA, accessibility, mobile layout, performance, component extraction, dead code cleanup |

### Specific Session 11 Tasks

- Handle `prefill_schedule` URL param in DraftStudioShell
- Extract `DraftDetailsPanel` sub-components (still at 935 lines)
- Extract `DraftStudioShell` panel zone into sub-component
- Delete dead components: `ComposeModal` (if no other imports), `ComposerPromptCard`, `ComposerShortcutBar`, `ComposerTipsTray`
- Remove `tuitbot:compose` CustomEvent if no listeners remain
- Revision list pagination
- Mobile-responsive layout for three-zone layout
- Full accessibility audit (WCAG AA)
- Performance audit (bundle size, API call waterfall)
- Calendar month view day-click should create-and-redirect (already done in this session)

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Calendar UX regression for quick-compose users | Medium | Medium | Time-slot context is preserved via URL param. Users land in Draft Studio with full editing capabilities. Document the change in release notes. |
| Orphan drafts from `Cmd+N` + immediate nav-away | Low | Low | Drafts are cheap and soft-deletable. Archive cleanup handles this. Same behavior as rail "+" button. |
| `prefill_schedule` not yet consumed | Medium | Low | Users can manually set schedule time in the details panel. Auto-population deferred to Session 11. |
| Legacy `'composer'` preference in Tauri store | High | Low | Explicit migration in `loadHomeSurface()` handles this transparently. |
| `ComposeModal` still exists as dead code | True | Low | Not imported or rendered anywhere after this session. Clean deletion in Session 11. |

## Exact Inputs for Session 11

1. Read `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` — `prefill_schedule` handling, component extraction
2. Read `dashboard/src/lib/components/drafts/DraftDetailsPanel.svelte` — sub-component extraction
3. Read `dashboard/src/lib/components/ComposeModal.svelte` — verify no remaining imports, then delete
4. Read `dashboard/src/lib/components/home/ComposerPromptCard.svelte` — verify no remaining imports, then delete
5. `grep -r "ComposeModal\|ComposerPromptCard\|ComposerShortcutBar\|ComposerTipsTray\|tuitbot:compose"` — find dead references
6. Mobile layout audit: test three-zone grid at 768px and 1024px breakpoints
7. Run full accessibility audit with keyboard-only navigation
8. Run: `npm --prefix dashboard run check`, `npm --prefix dashboard run build`, full Rust CI
