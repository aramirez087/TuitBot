# Draft Studio — Metadata and Filters

## Organization Model

Draft Studio provides per-draft organizational tools that are secondary to the writing canvas. All metadata editing happens inline in a collapsible details panel — no modal forms.

### Metadata Fields

| Field | Type | Editable | Location |
|-------|------|----------|----------|
| Title | string, nullable | Yes, inline input | Details panel |
| Notes | text, nullable | Yes, expandable textarea | Details panel |
| Tags | many-to-many via `content_tag_assignments` | Yes, inline picker | Details panel |
| Source | string (manual/assist/discovery) | Read-only | Rail item badge + details panel |
| Status | string (draft/scheduled/posted) | Derived from workflow | Details panel |
| Content type | string (tweet/thread) | Read-only | Rail item badge + details panel |
| Ready state | boolean | Derived (content length > 10) | Rail dot + details panel |

### Tags

Tags are per-account and globally scoped (not per-draft). A draft can have multiple tags. Tags have an optional `color` field (UI for color selection is deferred).

**Storage**: `content_tags` table (id, account_id, name, color) and `content_tag_assignments` join table (content_id, tag_id).

**API**:
- `GET /api/tags` — list account tags
- `POST /api/tags` — create tag `{ name, color? }`
- `GET /api/drafts/:id/tags` — list tags on a draft
- `POST /api/drafts/:id/tags/:tag_id` — assign tag
- `DELETE /api/drafts/:id/tags/:tag_id` — unassign tag

### Filter and Sort

| Filter | Client/Server | Notes |
|--------|---------------|-------|
| Search | Client-side on title + content_preview | Debounced 300ms |
| Sort | Client-side | Options: updated, created, title A-Z, scheduled |
| Tag filter | Server-side via `?tag=` param | Reloads collection |
| Tab filter | Client-side | Existing: active, scheduled, posted, archive |

### Ready State

A simple derived indicator — green dot if content preview is >10 characters, amber otherwise. This is a lightweight heuristic, not a validation gate. Shown as a small dot on rail items and a label in the details panel.

## Architecture Decisions

### Details panel: right column, not modal
The details panel is a third column in the shell grid (280px), collapsible via `Cmd+Shift+D`. On viewports below 1024px, the panel hides entirely. This keeps metadata subordinate to the writing canvas while remaining easily accessible.

### Tags fetched separately per draft
Tags are loaded for the selected draft via `GET /api/drafts/:id/tags` rather than included in the list summary. This avoids a JOIN on every list call and keeps the summary endpoint fast.

### Tag filter triggers server reload
When a tag filter is set, the collection reloads from the server with `?tag=` param because tag assignments aren't on the summary. When cleared, a regular reload runs.

### Search is client-side
The collection is typically <200 drafts, making client-side search adequate. Server-side search exists on the list endpoint but is used only when tag filter is also active.

## Non-Goals

- **Tag colors in picker**: Tags support `color` field but no color picker UI yet.
- **Tags on rail items**: Tags are only visible in the details panel to avoid clutter and N+1 fetches.
- **Bulk tagging**: No multi-select tag operations.
- **Mobile details drawer**: Below 1024px the panel hides. A proper bottom drawer is deferred to Session 10.
- **Content type filter chips**: The filter bar supports tag and sort but not content type chips (most accounts use only tweets).

## Keyboard Shortcuts

| Key | Context | Action |
|-----|---------|--------|
| `Cmd+Shift+D` | Shell | Toggle details panel |
| `/` | Rail list | Focus search input |
| All existing rail shortcuts | Rail list | Unchanged (N, D, R, Delete, arrows, 1-4) |
