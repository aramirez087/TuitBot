# Draft Studio QA Matrix

## Quality Gates

| Gate | Status | Notes |
|------|--------|-------|
| `npm run check` | PASS | 0 errors, 6 warnings (all pre-existing a11y/reactivity, not introduced by this epic) |
| `npm run build` | PASS | Production build succeeds after `$derived` export fix |
| `cargo fmt --all --check` | PASS | Clean |
| `cargo clippy --workspace -- -D warnings` | PASS | Clean |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | PASS | All tests pass |

## Functional Test Coverage

### Tweet Compose

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 1 | Create draft | Click "New Draft" on home quickstart or rail "+" button | Draft created, appears in rail, composer loads | PASS | Telemetry logged: `draft_created` |
| 2 | Type content | Type in composer textarea | Content appears, character count updates | PASS | |
| 3 | Autosave fires | Type content, wait 1.5s | Sync badge shows "Saving..." → "Saved" | PASS | DraftSaveManager handles server-backed saves |
| 4 | Content persists on reload | Type content, reload page | Draft content preserved | PASS | Server-backed persistence via autosave |

### Thread Compose

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 5 | Switch to thread mode | Click thread toggle in toolbar | Thread flow lane appears with 2 blocks | PASS | |
| 6 | Add tweet to thread | Click "+" divider between blocks | New block inserted | PASS | |
| 7 | Reorder blocks | Drag block handles | Blocks reorder, thread saves | PASS | |
| 8 | Delete block | Click delete on a thread block | Block removed, thread re-indexed | PASS | |

### Media

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 9 | Attach image | Click media button or paste image | Preview renders in editor | PASS | |
| 10 | Remove image | Click X on media preview | Image removed from draft | PASS | |
| 11 | Save with media | Attach image, wait for autosave | Media path persisted in draft | PASS | |

### Autosave Failure & Recovery

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 12 | Offline detection | Disconnect network while editing | Sync badge shows "Offline" | PASS | `save_failed` telemetry logged |
| 13 | Reconnect resync | Reconnect network | Autosave resumes, badge returns to "Saved" | PASS | |
| 14 | Tab close recovery | Close tab during edit, reopen | Draft content preserved via server-backed autosave | PASS | No more localStorage-only autosave for Draft Studio |
| 15 | Conflict detection | Edit same draft in two tabs | Conflict UI appears in sync badge | PASS | Resolution options: "Use mine" or "Reload server" |

### Schedule

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 16 | Set schedule | Set date/time in details panel, click Schedule | Draft moves to "Scheduled" tab | PASS | `transition` telemetry logged |
| 17 | Appears in calendar | Schedule a draft | Draft visible on calendar at scheduled time | PASS | |
| 18 | Unschedule | Click "Unschedule" on scheduled draft | Returns to "Active" tab | PASS | |
| 19 | Reschedule | Click "Reschedule", set new time, save | Schedule updated | PASS | |

### Publish

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 20 | Publish draft | Submit draft for publishing | Status changes to "Posted", moves to posted tab | PASS | |
| 21 | Duplicate posted | Click "Duplicate as draft" on posted draft | New draft created with same content | PASS | |

### History & Restore

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 22 | View revisions | Open History panel (Cmd+Shift+H) | Revision list loads | PASS | |
| 23 | Restore older version | Click restore on a revision | Content reverts, new revision created | PASS | `restore_executed` telemetry logged |
| 24 | Activity log | View History panel | Activity entries show actions | PASS | |

### Keyboard Navigation

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 25 | Cmd+N new draft | Press Cmd+N from any page | Navigates to `/drafts?new=true`, draft created | PASS | |
| 26 | Escape to rail | Press Escape in composer | Focus moves to rail | PASS | |
| 27 | Cmd+Shift+D toggle details | Press Cmd+Shift+D | Details panel toggles | PASS | |
| 28 | Cmd+Shift+H toggle history | Press Cmd+Shift+H | History panel toggles | PASS | |
| 29 | Tab through zones | Tab key navigation | Focus moves rail → composer → details | PASS | |

### Mobile / Responsive Behavior

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 30 | ≤1024px: details hidden | Resize to 1024px | Details panel hidden, grid becomes 2-column | PASS | CSS media query in DraftStudioShell |
| 31 | Rail + composer at 768px | Resize to 768px | Rail and composer remain visible | PASS | |

### Entry Points

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 32 | Home "New Draft" | Click on home page quickstart | Draft created, navigate to `/drafts?id={id}` | PASS | |
| 33 | Home recent drafts | Click a recent draft on home | Navigate to `/drafts?id={id}` | PASS | |
| 34 | Calendar slot click | Click a time slot on calendar | Draft created with `prefill_schedule`, redirect to Draft Studio | PASS | |
| 35 | Calendar "New Draft" button | Click header button on calendar | Draft created, redirect to Draft Studio | PASS | |
| 36 | Cmd+N from any page | Press Cmd+N | Navigate to `/drafts?new=true` | PASS | |
| 37 | Rail "+" button | Click "+" on draft rail | New draft created inline | PASS | |

### prefill_schedule

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 38 | Calendar time-slot prefill | Click calendar slot → land in Draft Studio | Schedule date/time pickers pre-populated | PASS | |
| 39 | Invalid prefill_schedule | Navigate with `?prefill_schedule=invalid` | Falls back to default (tomorrow), URL cleaned | PASS | |
| 40 | Already-scheduled draft | Open scheduled draft with prefill param | Existing schedule preserved, prefill ignored | PASS | |

### Archive / Restore

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 41 | Archive draft | Click archive on a draft | Moves to archive tab | PASS | |
| 42 | Restore from archive | Click restore on archived draft | Returns to active tab | PASS | |

### Tags

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 43 | Assign tag | Open tag picker, click a tag | Tag pill appears on draft | PASS | |
| 44 | Create new tag | Type in tag input, press Enter | New tag created and assigned | PASS | |
| 45 | Remove tag | Click X on tag pill | Tag unassigned | PASS | |
| 46 | Filter by tag | Select tag filter in rail | Only drafts with that tag shown | PASS | |

### Search / Sort

| # | Test Case | Steps | Expected | Status | Notes |
|---|-----------|-------|----------|--------|-------|
| 47 | Search by title | Type in search field | Rail filters to matching drafts | PASS | |
| 48 | Search by content | Search for content text | Matching drafts shown | PASS | |
| 49 | Sort by updated | Select "Updated" sort | Drafts ordered by last update | PASS | |
| 50 | Sort by created | Select "Created" sort | Drafts ordered by creation date | PASS | |

## Pre-existing Warnings (Not Introduced by This Epic)

1. `AddTargetModal.svelte:54` — a11y `div` with click handler
2. `WeeklyTrendChart.svelte:23` — non-reactive `canvas` update
3. `ComposerPreviewSurface.svelte:62` — a11y click without keyboard handler
4. `PolicySection.svelte:406,408` — a11y `div` with click handler
5. `AccountsSection.svelte:215` — autofocus usage

These are tracked but out of scope for the Draft Studio epic.
