# Session 01 Handoff

## What Changed

Four documentation files created under `docs/roadmap/composer-ui-typefully-redesign/`:

| File | Purpose |
|------|---------|
| `benchmark-notes.md` | Gap analysis: current composer vs. Typefully-class target. Audits all three surfaces (home embedded, modal, focus mode), catalogs chrome density, visual rhythm mismatches, and shortcut safety issues. |
| `charter.md` | Redesign charter: 5 goals, 5 non-goals, target interaction model, acceptance criteria per session, preserved contracts, and session timeline (S2–S5). |
| `ui-architecture.md` | Technical architecture: current and target component trees, `PreviewOverlay` design (props, behavior, keyboard handling, accessibility), state flow changes, hotspot files by session, CSS token plan, contract preservation evidence. |
| `session-01-handoff.md` | This file. |

**No source code was modified.** This is a documentation-only session.

## Decisions Made

### D1: Textarea, not contenteditable
The editor stays a `<textarea>`. A `contenteditable` rewrite would enable richer WYSIWYG but introduces cursor management, paste handling, IME, and accessibility complexity that outweighs the visual gain. Styling changes to the textarea (borderless, larger font, transparent background) close most of the perceived gap.

### D2: PreviewOverlay is a sibling component, not a route
The full-screen preview is a conditionally-rendered `PreviewOverlay.svelte` inside `ComposeWorkspace`, not a new SvelteKit route. This avoids draft state serialization/deserialization and keeps the preview tightly coupled to the editor's reactive state.

### D3: Character counter visibility threshold at 240
Character counters (both TweetEditor and ThreadFlowCard) are hidden when the count is ≤ 240 characters (out of 280 max). This removes always-visible chrome while giving the user ~40 characters of warning before the limit. The 240 threshold was chosen because:
- X's own composer shows the ring indicator at approximately the same ratio
- It gives enough buffer for the user to notice and adjust
- It keeps the writing surface clean for most drafts

### D4: HomeComposerHeader simplification
The schedule pill is removed from the always-visible header row and made accessible via inspector or command palette. The header retains: handle, post count indicator, publish pill, and a compact icon group. This reduces the header from 10 interactive elements to ~6.

### D5: Inline preview removed entirely
The `ThreadPreviewRail` is removed from the canvas, not just hidden. The inline preview competed for vertical space, duplicated content, and added chrome (toggle button, "Preview" label). The replacement is the dedicated `PreviewOverlay`.

## Open Questions (with Recommendations)

### Q1: Should thread cards show character count only when approaching the limit?
**Recommendation: Yes.** Hide the count when ≤ 240 characters, same as tweet mode. This is a simple derived check: `const showCount = $derived(charCount > 240 || overLimit)`. The separator row still exists for hover tools, but the count text is conditionally rendered.

### Q2: Should the preview overlay include an explicit "Edit" button?
**Recommendation: Yes.** Escape closes the overlay, but a visible "Back to editing" or close (X) button is important for discoverability, especially for users who don't know the keyboard shortcut. The overlay header should have a close button.

### Q3: Should the preview overlay support editing (making it a split view)?
**Recommendation: No.** The overlay is read-only. Adding editing capability would require cursor synchronization between two textareas, which is complex and fragile. The intended flow is: write → preview → close preview → continue writing.

### Q4: Should focus mode (full-viewport modal) incorporate the preview overlay styling?
**Recommendation: No, keep them separate.** Focus mode is about maximizing writing space. Preview mode is about seeing the rendered post. They serve different purposes and should remain independent states. A user in focus mode can still open preview overlay on top.

## Shortcut Policy

This is the definitive shortcut policy to implement across Sessions 2–4.

### Safe Shortcuts (no content mutation)
| Combo | Action | Scope |
|-------|--------|-------|
| `Cmd+K` | Open command palette | Always |
| `Cmd+I` | Toggle inspector panel | Always |
| `Cmd+Shift+P` | Open/close preview overlay | Always |
| `Cmd+Shift+F` | Toggle focus mode | Modal only |
| `Cmd+Shift+N` | Switch to tweet mode | Always |
| `Cmd+Shift+T` | Switch to thread mode | Always |
| `Escape` | Close: from-notes → mobile inspector → focus mode → modal | Cascading |
| `Tab` / `Shift+Tab` | Navigate between thread cards | Thread only |
| `Alt+ArrowUp` / `Alt+ArrowDown` | Reorder thread card | Thread only |

### Content-Mutating Shortcuts (require undo safety)
| Combo | Action | Undo | Scope |
|-------|--------|------|-------|
| `Cmd+J` | Improve with AI (selection or full content) | 10-second undo banner | Always |
| `Cmd+Enter` | Submit (tweet mode) / Split at cursor (thread mode) | Split: merge-back via `Cmd+Shift+M` | Mode-dependent |
| `Cmd+Shift+Enter` | Submit (always) | N/A (intentional publish) | Always |
| `Cmd+D` | Duplicate thread card | Remove via Trash | Thread only |
| `Cmd+Shift+S` | Split thread card at cursor | Merge via `Cmd+Shift+M` | Thread only |
| `Cmd+Shift+M` | Merge with next thread card | Re-split at old boundary not tracked | Thread only |
| `Backspace` at pos 0 | Merge with previous card | Re-split not tracked | Thread only |

### Key Rule
**No shortcut may silently destroy content without an undo path.** The `Cmd+J` undo banner is the primary new mechanism. Thread operations (split/merge) are reversible via their inverse operations, which is acceptable.

## Session 02 Inputs

### Starting Files
Session 2 should read these documents first:
- `docs/roadmap/composer-ui-typefully-redesign/charter.md` — goals, acceptance criteria
- `docs/roadmap/composer-ui-typefully-redesign/ui-architecture.md` — target component tree, hotspot table

### First Three Tasks (ordered)

**Task 1: Restyle TweetEditor**
- File: `dashboard/src/lib/components/composer/TweetEditor.svelte`
- Changes:
  - `.compose-input`: Remove `border`, set `background: transparent`, `font-size: 15px`, `line-height: 1.4`, `padding: 12px 0`, remove `border-radius`
  - `.char-counter`: Wrap in `{#if tweetChars > 240}` conditional
  - `.media-attach-section`: Replace button+hint with compact icon-only affordance
  - `.compose-input:focus`: Remove `border-color` change (no border to change), add subtle left-border or bottom-underline if needed for focus indication
- Validation: `npm --prefix dashboard run check`

**Task 2: Reduce ThreadFlowCard chrome**
- File: `dashboard/src/lib/components/composer/ThreadFlowCard.svelte`
- Changes:
  - `.flow-textarea`: `font-size: 15px`, `line-height: 1.4`
  - `.spine-dot`: Reduce to 8px, thinner border (1px instead of 2px)
  - `.sep-char-count`: Wrap in `{#if charCount > 240 || overLimit}` conditional
  - `.between-zone`: Reduce height to 12px, smaller plus circle (14px)
  - `.card-separator`: Reduce height to 20px when char count is hidden
- Also: `dashboard/src/lib/components/composer/ThreadFlowLane.svelte`
  - `.lane-spine`: Reduce to `width: 0.5px` or use `border-left` for crispness
  - Reduce `padding-left` from 40px to 32px
  - Adjust `.spine-dot` `left` offset to match new padding
- Validation: `npm --prefix dashboard run check`

**Task 3: Simplify HomeComposerHeader**
- File: `dashboard/src/lib/components/composer/HomeComposerHeader.svelte`
- Changes:
  - Remove schedule pill from the main header row
  - Keep: handle, post count, publish pill, and 3 icon tools (preview, inspector, palette)
  - AI improve button stays (it's a key affordance)
  - Reduce padding: `8px 16px` instead of `10px 20px`
- Validation: `npm --prefix dashboard run check`

### Validation Command
```bash
npm --prefix dashboard run check
```

This must pass with zero errors before Session 2 creates its handoff.
