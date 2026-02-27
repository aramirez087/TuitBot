# Session 01 Handoff

**Date:** 2026-02-27
**Session:** 01 — Charter and UI Gap Audit
**Status:** Complete
**Next Session:** 02 — Data Model & API Contract

---

## What Changed

Five charter documents created under `docs/roadmap/typefully-composer-ui-parity/`:

| File | Purpose |
|------|---------|
| `charter.md` | Project charter: scope, non-goals, 7 architecture decisions, success criteria, timeline |
| `ui-gap-audit.md` | Feature-by-feature gap analysis: 6 critical gaps, 5 important gaps, 10 nice-to-have, 9 out-of-scope |
| `superiority-scorecard.md` | Measurable criteria: 4 dimensions, 16 metrics, baselines, targets, measurement methods |
| `session-execution-map.md` | Sessions 02-08: file targets, quality gates, exit criteria, dependency graph |
| `session-01-handoff.md` | This file |

No source code was modified. No Rust, TypeScript, Svelte, or CSS changes.

---

## Decisions Locked

| ID | Decision | Rationale |
|----|----------|-----------|
| A-1 | ThreadComposer.svelte as separate component | ComposeModal at 787 lines; thread logic would push past 400-line limit. ThreadComposer owns thread state; ComposeModal orchestrates |
| A-2 | Stable block IDs with client-generated UUID | Enables optimistic reorder without re-indexing. Per-block media references survive reorder |
| A-3 | Structured JSON for thread data (array of block objects) | Current `\n---\n` join loses per-tweet metadata. JSON preserves block IDs, ordering, per-block media. Backwards-compatible |
| A-4 | Side-panel preview (not inline WYSIWYG) | Avoids contenteditable complexity. Keeps textarea as reliable input. Clear separation between authoring and preview |
| A-5 | CommandPalette.svelte with Cmd+K | Declarative action registry. Mounted inside ComposeModal, active only during compose. No global store pollution |
| A-6 | Full-viewport focus mode via modal state | Simpler than separate route. Hides sidebar/header/chrome; shows only editor + preview. Toggle via Cmd+Shift+F |
| A-7 | localStorage auto-save with 500ms debounce | Zero backend cost. Recovery prompt on next open. Clear on submit. 10-slot cap, 7-day TTL |

---

## Risk Register

| # | Risk | Impact | Likelihood | Mitigation |
|---|------|--------|-----------|------------|
| R-1 | Thread block schema breaks existing drafts | High | Medium | Backwards-compatible: accept both `content` string and `blocks` array. Server normalizes internally. No DB migration required |
| R-2 | Drag-and-drop reorder conflicts with textarea focus | Medium | Medium | Use dedicated drag handle zones, not full-card drag. Keyboard reorder (Alt+Up/Down) as primary path |
| R-3 | WYSIWYG preview diverges from actual X rendering | Medium | High | Use conservative styling. Focus on structural accuracy (text length, media grid), not pixel-matching X |
| R-4 | ComposeModal refactor introduces regressions in tweet mode | High | Medium | Session 03 must preserve all existing tweet compose behavior: open, type, attach media, schedule, submit. Explicit regression test |
| R-5 | Mobile compose UX degraded by 520px modal | Medium | Low | Session 06 adds mobile breakpoints. For < 768px, modal expands to full viewport. Touch targets >= 44px |
| R-6 | Accessibility violations in suppressed `svelte-ignore` blocks | Medium | High | Session 06 removes all `svelte-ignore a11y_*` directives. Replace with proper ARIA attributes. Axe audit to verify zero critical violations |
| R-7 | Command palette conflicts with browser shortcuts | Low | Low | Cmd+K is safe in Tauri. In Chrome, overridable with `preventDefault()`. Test in both environments |
| R-8 | Per-tweet media upload UX confusion with reordering | Medium | Medium | Media slot visually attached to each card. Reordering moves media with card. Command palette action for "move media to card N" as escape hatch |
| R-9 | localStorage auto-save data growth | Low | Low | Clear on successful submit. Cap at 10 recovery slots. 7-day TTL with automatic cleanup on compose open |

---

## Exact Inputs for Session 02

### Documents to Read First

| File | Section | Purpose |
|------|---------|---------|
| `docs/roadmap/typefully-composer-ui-parity/charter.md` | Architecture Decisions A-2, A-3 | Block ID format and structured JSON design |
| `docs/roadmap/typefully-composer-ui-parity/session-01-handoff.md` | This file | Context and risk awareness |
| `docs/roadmap/typefully-composer-ui-parity/session-execution-map.md` | Session 02 section | Detailed file targets and exit criteria |

### Source Files to Read

| File | Purpose |
|------|---------|
| `crates/tuitbot-server/src/routes/content/compose.rs` | Current `ComposeRequest` struct and validation logic. Add `blocks` field |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | Current draft create/edit structs. Add `blocks` support |
| `crates/tuitbot-core/src/content/mod.rs` | Add `ThreadBlock` struct and `validate_thread_blocks()` |
| `dashboard/src/lib/api.ts` | Add `ThreadBlock` interface and update `ComposeRequest` type |

### Files to Create

| File | Purpose |
|------|---------|
| `crates/tuitbot-server/tests/compose_contract_tests.rs` | Contract tests for new and legacy compose flows |

### Commands to Run

```bash
# After all changes
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
```

### Key Constraints for Session 02

1. **Backwards compatibility is mandatory**: Existing `{content_type, content, scheduled_for, media_paths}` payloads must continue to work unchanged.
2. **`blocks` takes precedence over `content` when both present**: For thread payloads, if `blocks` is provided, use it. Ignore `content` in that case.
3. **Block ID uniqueness**: Validate that all block IDs within a request are unique.
4. **Block ordering**: Validate that `order` fields form a contiguous sequence starting at 0.
5. **Per-block media limits**: Each block independently enforces: max 4 images OR 1 GIF/video.
6. **No database schema changes**: Thread content continues to be stored in the existing `content` text column. Server serializes `blocks` to JSON string for storage and deserializes on read.
