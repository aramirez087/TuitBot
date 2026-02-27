# Session 07 — Documentation Updates

**Date:** 2026-02-27
**Session:** 07 — Docs & Adoption Readiness

---

## Files Modified

| File | Change Summary |
|------|----------------|
| `docs/composer-mode.md` | Full rewrite: corrected all API endpoint paths, added 8 new sections, removed non-existent endpoints, documented thread composer UX, shortcuts, command palette, focus mode, auto-save, accessibility, migration notes, troubleshooting |

## Files Created

| File | Purpose |
|------|---------|
| `docs/roadmap/typefully-composer-ui-parity/shortcut-cheatsheet.md` | Complete keyboard shortcut reference for the compose modal — 14 shortcuts + 13 command palette actions |
| `docs/roadmap/typefully-composer-ui-parity/session-07-doc-updates.md` | This file |
| `docs/roadmap/typefully-composer-ui-parity/session-07-handoff.md` | Handoff for Session 08 |

---

## Endpoint Corrections

All corrections verified against `crates/tuitbot-server/src/lib.rs` route registrations.

| Previous (Incorrect) | Corrected | Change Type |
|-----------------------|-----------|-------------|
| `POST /api/drafts` | `POST /api/content/drafts` | Wrong prefix |
| `GET /api/drafts` | `GET /api/content/drafts` | Wrong prefix |
| `GET /api/drafts/{id}` | Removed | Route does not exist |
| `PUT /api/drafts/{id}` | `PATCH /api/content/drafts/{id}` | Wrong prefix + wrong HTTP method |
| `POST /api/drafts/{id}/publish` | `POST /api/content/drafts/{id}/publish` | Wrong prefix |
| `DELETE /api/drafts/{id}` | `DELETE /api/content/drafts/{id}` | Wrong prefix |
| (missing) | `POST /api/content/drafts/{id}/schedule` | Newly documented |
| `POST /api/discovery/feed/{tweet_id}/reply` | `POST /api/discovery/{tweet_id}/compose-reply` + `POST /api/discovery/{tweet_id}/queue-reply` | Wrong path; actually two separate endpoints |
| `GET /api/discovery/feed/stats` | Removed | Route does not exist in server |
| `POST /api/assist/compose` | Removed | Route does not exist; was confused with `/api/content/compose` |
| (missing) | `GET /api/assist/mode` | Newly documented |
| (missing) | `POST /api/content/compose` | Newly documented (primary compose endpoint) |
| (missing) | `POST /api/media/upload` | Newly documented |
| (missing) | `GET /api/media/file` | Newly documented |
| (missing) | `GET /api/discovery/keywords` | Newly documented |

---

## New Sections Added to composer-mode.md

| Section | Purpose |
|---------|---------|
| Thread Composer | Card-based editor, data model, validation rules, power actions, per-tweet media |
| Distraction-Free Mode | Full-viewport focus mode, escape cascade |
| Command Palette | Fuzzy search over 13 actions, category organization, navigation |
| Keyboard Shortcuts | Quick reference table for all 14 shortcuts |
| Auto-Save & Recovery | localStorage debounce, TTL, edge cases |
| Compose Endpoint | Request body format with `blocks` field documentation |
| Media Upload | Upload/serve endpoints, accepted types, size limits |
| Accessibility | Focus trap, focus return, ARIA, contrast, reduced motion, mobile, touch targets |
| Migration Notes | 7 items for users upgrading from pre-thread-composer versions |
| Troubleshooting | Compose errors, thread validation, auto-save recovery, media issues |

## Sections Preserved Unchanged

| Section | Reason |
|---------|--------|
| Enabling Composer Mode | Accurate as-is |
| What Changes | Table is accurate for current runtime behavior |
| MCP Tools | Accurate as-is |
| Switching Between Modes | Accurate as-is |

## Sections Updated

| Section | Change |
|---------|--------|
| AI Assist | Added inline AI improve (⌘J), generate from notes, AI button context-aware behavior. Fixed endpoint table: removed non-existent `/api/assist/compose`, added `GET /api/assist/mode`. |
| Drafts | Fixed all endpoint paths (`/api/drafts/*` → `/api/content/drafts/*`), fixed HTTP method (`PUT` → `PATCH`), removed non-existent `GET /api/drafts/{id}`, added `POST /api/content/drafts/{id}/schedule`. Added note about thread draft blocks format. |
| Discovery Feed | Fixed endpoint paths, removed non-existent `/api/discovery/feed/stats`, split single reply endpoint into two (compose-reply, queue-reply), added `/api/discovery/keywords`. |

---

## Superiority Differentiators Documented

The updated `composer-mode.md` explicitly calls out wins over Typefully in these dimensions:

### Writing Speed
- 14 keyboard shortcuts vs Typefully's 1 (`Cmd+Enter`)
- Command palette (`⌘K`) for instant access to any action
- Tab/Shift+Tab for fast thread card navigation

### Structural Control
- 4 power actions (reorder, duplicate, split, merge) vs Typefully's 1 (drag-and-drop reorder only)
- All power actions keyboard-accessible
- Per-tweet media in threads (not available in Typefully)

### Feedback Clarity
- Real-time side-panel preview with tweet-card rendering
- Auto-save with explicit recovery prompt (Typefully uses silent auto-save with no recovery UX)
- Inline validation (character count, empty card detection, media limit enforcement)

### Accessibility
- Full keyboard composability — every action achievable without a mouse
- Focus trap, focus return, ARIA roles, `aria-live` regions
- WCAG AA contrast (4.5:1 minimum) in both themes
- `prefers-reduced-motion` support
- 44px touch targets on touch devices
- Full-viewport mobile layout with iOS zoom prevention
