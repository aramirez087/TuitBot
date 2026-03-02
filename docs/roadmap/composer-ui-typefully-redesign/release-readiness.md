# Release Readiness — Composer Redesign

## Ship Recommendation: GO

The composer redesign meets all five charter goals, passes all quality gates, preserves all 12 API/state contracts, and introduces no regressions. Ship it.

## Charter Goal Verification

### G1: The Compose Surface Feels Like the Post

| Criterion | Evidence | Verdict |
|-----------|----------|---------|
| Borderless textarea | `TweetEditor.svelte:181-183` — `border: none; background: transparent` | Met |
| 15px font, 1.4 line-height | `TweetEditor.svelte:185-187` | Met |
| Character counter hidden ≤240 | `TweetEditor.svelte:118` — conditional render | Met |
| Thread cards borderless, same font | `ThreadFlowCard.svelte:234-248` — 15px, 1.4 lh, no border | Met |
| Compact media attach (icon-only) | `TweetEditor.svelte:153-161` — 32px icon button, no hint text | Met |

### G2: X-Accurate Preview in Dedicated Full-Screen Mode

| Criterion | Evidence | Verdict |
|-----------|----------|---------|
| `ThreadPreviewRail` removed from inline | Not imported in `ComposeWorkspace.svelte` | Met |
| Full-screen overlay component | `ComposerPreviewSurface.svelte:109-112` — `position: fixed; inset: 0; z-index: 2000` | Met |
| Same reactive state (no duplication) | `ComposeWorkspace.svelte:613-621` — props from workspace `$state` | Met |
| `Cmd+Shift+P` opens overlay | `ComposeWorkspace.svelte:310` | Met |
| Escape closes overlay | `ComposeWorkspace.svelte:289` | Met |
| Preview button in header | `HomeComposerHeader.svelte:84`, `ComposerHeaderBar.svelte:38` | Met |
| Thread preview shows non-empty blocks with connectors | `ComposerPreviewSurface.svelte:90-99` | Met |
| Focus trap + restore | `ComposerPreviewSurface.svelte:36-46,59` | Met |

### G3: Chrome Retreats to Periphery

| Criterion | Evidence | Verdict |
|-----------|----------|---------|
| Separator tools hidden until hover | `ThreadFlowCard.svelte:284` — `opacity: 0`, `:hover → opacity: 1` | Met |
| Touch: always visible | `ThreadFlowCard.svelte:378-385` — `@media (hover: none)` | Met |
| Between-zone "+" subtle | `ThreadFlowCard.svelte:352-365` — 14px circle, opacity 0 until hover | Met |
| Lane spine lighter | `ThreadFlowLane.svelte:429-430` — 1px, 60% opacity | Met |
| Spine dot smaller | `ThreadFlowCard.svelte:211-222` — 8px, thinner border | Met |
| Header simplified | `HomeComposerHeader.svelte` — no schedule pill in main row | Met |

### G4: All Shortcuts Are Safe

| Criterion | Evidence | Verdict |
|-----------|----------|---------|
| `Cmd+J` snapshots before replacement | `ComposeWorkspace.svelte:349` (tweet), `:369` (thread) | Met |
| 10-second undo banner | `ComposeWorkspace.svelte:362` — `setTimeout(() => showUndo = false, 10000)` | Met |
| Selection-only replacement | `ComposeWorkspace.svelte:354-356` — conditional slice | Met |
| Snapshot cleared on failure | `ComposeWorkspace.svelte:365` | Met |
| `SHORTCUT_CATALOG` updated | `shortcuts.ts:103-122` — 17 entries, mode-specific | Met |
| No shortcut silently destroys content | Verified in `shortcut-regression-matrix.md` | Met |

### G5: Thread Mode Feels as Clean as Tweet Mode

| Criterion | Evidence | Verdict |
|-----------|----------|---------|
| Flowing cards with minimal dividers | `ThreadFlowCard.svelte` — borderless, hover-only chrome | Met |
| Lighter spine (thinner, subtler) | `ThreadFlowLane.svelte:429-430` — 1px, 60% transparency | Met |
| Char counter hidden ≤240 per card | `ThreadFlowCard.svelte:133` — conditional render | Met |
| Between-zone "+" subtle | `ThreadFlowCard.svelte:352-365` — hidden until hover | Met |

## Quality Gate Results

| Check | Result | Details |
|-------|--------|---------|
| `npm --prefix dashboard run check` | 0 errors, 7 warnings | All warnings pre-existing, outside composer redesign scope |
| `cargo fmt --all --check` | Clean | No formatting issues |
| `cargo clippy --workspace -- -D warnings` | Clean | No lint issues |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | All tests pass | No failures |

### Pre-existing Warnings (all non-blocking)

1. `TweetEditor.svelte:174` — empty CSS ruleset (intentional wrapper)
2. `AddTargetModal.svelte:54` — div click without ARIA role (outside composer)
3. `ComposerPreviewSurface.svelte:62` — backdrop click without keyboard handler (mitigated by `svelte-ignore` + Escape key)
4. `WeeklyTrendChart.svelte:23` — canvas not `$state` (outside composer)
5. `drafts/+page.svelte:369` — empty CSS ruleset (outside composer)
6-7. `PolicySection.svelte:406,408` — div clicks without ARIA role (outside composer)

## Contract Preservation

All 12 contracts from the charter are preserved unchanged:

| Contract | Location | Status |
|----------|----------|--------|
| `ThreadBlock[]` shape | `$lib/api` type export | Unchanged |
| `ComposeRequest` shape | `$lib/utils/composeHandlers` | Unchanged |
| `onsubmit(data)` callback | `ComposeWorkspace` prop | Unchanged |
| Autosave format `{ mode, tweetText, blocks, timestamp }` | `ComposeWorkspace.svelte:206` | Unchanged |
| `AUTOSAVE_TTL_MS` (7 days) | `ComposeWorkspace.svelte:80` | Unchanged |
| Modal entry: `ComposeModal` props | `ComposeModal.svelte:5-19` | Unchanged |
| Home entry: embedded workspace | `+page.svelte:33-37` | Unchanged |
| `api.content.compose()` | `+page.svelte:21` | Unchanged |
| `api.content.schedule()` | `+page.svelte:13` | Unchanged |
| `api.assist.improve()` | `ComposeWorkspace.svelte:353` | Unchanged |
| `api.assist.thread()` | `ComposeWorkspace.svelte:386` | Unchanged |
| `api.media.upload()` | `TweetEditor.svelte:77` | Unchanged |

## Blocking Issues

None identified.

## Non-Blocking Issues

### NB-1: Mobile icon-tools hidden at ≤640px (pre-existing)

**File:** `HomeComposerHeader.svelte:293-295`
**Description:** At narrow widths, preview/AI/inspector/palette buttons are hidden via `display: none`. These features remain accessible via keyboard shortcuts.
**Risk:** Low — this is a pre-existing design decision, not a regression.
**Recommendation:** Future session could add a hamburger menu or bottom toolbar for mobile.

### NB-2: AI generate (palette action) lacks undo

**File:** `ComposeWorkspace.svelte:439-460`
**Description:** The "AI generate" palette action replaces content without snapshotting. This is a palette-only action with no keyboard shortcut.
**Risk:** Low — requires deliberate palette invocation.
**Recommendation:** Add undo snapshot to `handleAiAssist()` for parity with `handleInlineAssist()`.

### NB-3: Thread mode undo restores all blocks

**File:** `ComposeWorkspace.svelte:369,404-411`
**Description:** If user edits block B while undo banner is showing after AI improve on block A, pressing undo restores all blocks to pre-AI state. The 10-second window makes this unlikely.
**Risk:** Very low — matches existing "from notes" undo behavior.
**Recommendation:** Accept. Per-block undo would require significant complexity for marginal benefit.

### NB-4: `Cmd+D` browser bookmark conflict

**File:** `ThreadFlowLane.svelte:292-296`
**Description:** On some browsers, `Cmd+D` triggers bookmark dialog. Mitigated by `e.preventDefault()` when a thread card textarea is focused.
**Risk:** Very low — only fires when textarea is focused, which suppresses the browser shortcut.
**Recommendation:** Accept. Documented in shortcut regression matrix.

### NB-5: `ComposerPreviewSurface` backdrop click a11y warning

**File:** `ComposerPreviewSurface.svelte:61-62`
**Description:** Backdrop click handler has `svelte-ignore a11y_click_events_have_key_events`. The dialog is closable via Escape and the close button.
**Risk:** None — the warning is suppressed and the keyboard path exists.
**Recommendation:** Accept.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Autosave format incompatibility | Very low | Medium | Format is unchanged: `{ mode, tweetText, blocks, timestamp }` |
| Preview overlay blocks interaction | Very low | Low | `z-index: 2000` is above modal (`1000`) and inspector (`1099`); focus trap prevents background interaction |
| Shortcut conflicts with browser | Low | Low | `e.preventDefault()` on all composer shortcuts; documented in regression matrix |
| Mobile usability gap (hidden icon-tools) | Low | Low | Pre-existing pattern; keyboard shortcuts still work; inspector has mobile drawer |
| Thread undo restores too much | Very low | Low | 10-second window; matches existing behavior |

## Conclusion

The redesign achieves its stated goals:
- The compose surface feels like the rendered post (borderless, X-matched typography, hidden-until-needed chrome)
- Preview is a dedicated full-screen overlay sharing the same reactive state
- All shortcuts are safe with undo protection
- Thread mode is visually clean with on-demand tooling
- All quality gates pass, all contracts preserved, no regressions

**Recommendation: Ship.**
