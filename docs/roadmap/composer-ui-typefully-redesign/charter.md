# Composer Redesign Charter

## Problem Statement

The current Tuitbot composer feels like a form-filling exercise rather than a writing experience. The editor is a bordered `<textarea>` with generic styling; the preview is a secondary panel rendered inline below the editor; and the header, separator chrome, and always-visible counters compete for attention with the actual content. The gap between "what I'm typing" and "what my post will look like" forces users to mentally translate between two representations instead of writing directly into what feels like the final product.

The target: a Typefully-class writing canvas where the compose surface itself approximates the rendered post, and X-accurate preview lives in a dedicated full-screen mode — not an inline panel.

## Goals

### G1: The Compose Surface Feels Like the Post
The editor should use typography, spacing, and visual rhythm that approximate X's post rendering. No visible borders, no form-input background, no always-visible character counter. The user should feel like they are writing their post, not filling out a textarea.

### G2: X-Accurate Preview in Dedicated Full-Screen Mode
Remove the inline `ThreadPreviewRail`. Replace it with a full-screen overlay triggered by `Cmd+Shift+P` or a button. The overlay reuses the same reactive draft state (no cloning, no route change). The existing `TweetPreview` and `MediaCropPreview` components are rendered inside the overlay with X-branded chrome (avatar gutter, handle, thread connectors).

### G3: Chrome Retreats to Periphery
Toolbars, counters, and action buttons appear on demand (hover, focus, approaching character limit) rather than being always visible. Thread card separators show only the minimum affordance needed; tools appear on hover. The header has fewer always-visible elements.

### G4: All Shortcuts Are Safe
No keyboard shortcut may silently destroy or replace content without an undo path. `Cmd+J` (inline assist) must snapshot content before replacement. Every destructive shortcut either requires a selection or provides undo.

### G5: Thread Mode Feels as Clean as Tweet Mode
Thread cards flow with minimal visual interruption. The spine and dot system is lighter. Separator chrome (char count, drag handle, merge/remove buttons) is hidden until hover. Between-zone "+" buttons are more subtle.

## Non-Goals

### NG1: No Backend or Rust API Changes
All work stays inside `dashboard/`. No new API endpoints, no schema changes, no Rust code modifications. If the frontend is blocked by missing API functionality, the blocker is documented and escalated — not worked around with backend changes.

### NG2: No New API Contracts
The existing contracts are preserved exactly:
- `ThreadBlock[]` shape: `{ id: string; text: string; media_paths: string[]; order: number }`
- `ComposeRequest` shape: passed to `onsubmit` callback
- Autosave format: `{ mode, tweetText, blocks, timestamp }` under `AUTOSAVE_KEY`
- Schedule/submit APIs: `api.content.compose()`, `api.content.schedule()`
- Modal entry points: `ComposeModal` with `open`, `prefillTime`, `prefillDate`, `schedule`, `onclose`, `onsubmit`

### NG3: No Route Changes
The preview overlay is a conditionally-rendered component inside `ComposeWorkspace`, not a separate SvelteKit route. No `+page.svelte` files are added or modified for preview.

### NG4: No contenteditable Rewrite
The editor remains a `<textarea>`. Switching to `contenteditable` would enable richer WYSIWYG (bold, links, hashtag highlighting) but introduces significant complexity (cursor management, paste handling, IME support, accessibility) and is out of scope. Styling improvements to the textarea close most of the visual gap.

### NG5: No Tauri-Specific Changes
The sidecar launch, `localhost:3001` connection, and Tauri bridge are unchanged. Desktop chrome stays as-is.

## Target Interaction Model

### Writing Mode
- **Tweet mode**: A single borderless, transparent-background textarea with 15px font, 1.4 line-height, and generous padding. Character counter appears only when count exceeds 240 (as a subtle ring or number near the bottom-right). Media attach affordance is a small icon, not a labeled button with hint text.
- **Thread mode**: Flowing cards with minimal dividers. Each card is a borderless auto-resizing textarea. The spine is lighter (thinner line, smaller dots). Separator tools (drag, merge, remove) appear on hover only. Character count per card appears only when approaching the limit. Between-zone "+" is a barely-visible affordance until hover.
- **Header (home)**: Simplified to handle + mode indicator + publish pill + a collapsed tool menu. Schedule and inspector are accessible via the tool menu or shortcuts, not always-visible pills.
- **Header (modal)**: Close button, publish button, and a collapsed icon group for preview/inspector/focus.

### Preview Mode
- Triggered by `Cmd+Shift+P` or a preview button
- Opens a full-screen dark overlay centered on the viewport
- Shows `TweetPreview` components (existing) in X-branded chrome: avatar circle, handle, thread connectors, media crops
- Content is read-only — the overlay shares the same `$state` as the editor (no duplication)
- Escape key or close button returns to editing
- In thread mode: scrollable list of all thread posts with connectors

### AI Assist (`Cmd+J`)
- Always takes an undo snapshot before calling the API
- If text is selected: replaces only the selection
- If no text is selected: replaces the entire tweet/block content
- After replacement: shows a 10-second undo banner (matching the existing "from notes" undo pattern)
- Visual feedback during API call: pulsing overlay or opacity reduction on the affected area

## Acceptance Criteria

### Session 2: Live Canvas Surface
- [ ] TweetEditor renders without visible border or distinct background
- [ ] TweetEditor font is 15px with 1.4 line-height
- [ ] Character counter in TweetEditor is hidden when count ≤ 240
- [ ] ThreadFlowCard separator tools are hidden until hover (character count included)
- [ ] ThreadFlowCard character count is hidden when count ≤ 240
- [ ] Lane spine visual weight is reduced (thinner line, smaller dots)
- [ ] HomeComposerHeader has fewer always-visible elements (schedule pill moved behind menu)
- [ ] Media attach section uses a compact icon instead of button + hint text
- [ ] `npm --prefix dashboard run check` passes

### Session 3: Dedicated X Preview
- [ ] `ThreadPreviewRail` is no longer rendered inline in `ComposerCanvas`
- [ ] New `PreviewOverlay.svelte` renders full-screen when `previewMode` is true
- [ ] `PreviewOverlay` receives read-only props from the same `$state` (no duplication)
- [ ] `Cmd+Shift+P` opens the overlay (replaces inline toggle)
- [ ] Escape key closes the overlay
- [ ] Preview button in header/command palette opens the overlay
- [ ] Thread preview shows all non-empty blocks with connectors
- [ ] `npm --prefix dashboard run check` passes

### Session 4: Shortcuts and Safety
- [ ] `Cmd+J` creates an undo snapshot before replacement
- [ ] After `Cmd+J` replacement: 10-second undo banner appears
- [ ] `Cmd+J` with selection: replaces only selection (existing behavior, now with undo)
- [ ] `Cmd+J` without selection: replaces full content (existing behavior, now with undo)
- [ ] `SHORTCUT_CATALOG` is updated to reflect any shortcut changes
- [ ] No shortcut silently destroys content without undo
- [ ] `npm --prefix dashboard run check` passes

### Session 5: Validation and Release
- [ ] All previous acceptance criteria still pass
- [ ] svelte-check reports zero errors
- [ ] Accessibility: all interactive elements have aria-labels, focus is managed correctly
- [ ] Mobile (≤640px): composer is usable, touch targets ≥44px
- [ ] Reduced-motion: no animations when `prefers-reduced-motion: reduce`
- [ ] Autosave and draft recovery still work (manual test)
- [ ] Modal and embedded surfaces both work correctly

## Preserved Contracts

| Contract | Location | Status |
|----------|----------|--------|
| `ThreadBlock[]` shape | `$lib/api` type export | Unchanged |
| `ComposeRequest` shape | `$lib/api` type export | Unchanged |
| `onsubmit(data: ComposeRequest)` callback | `ComposeWorkspace` prop | Unchanged |
| Autosave key + format | `ComposeWorkspace.svelte:78` | Unchanged |
| `AUTOSAVE_TTL_MS` (7 days) | `ComposeWorkspace.svelte:80` | Unchanged |
| Modal entry: `ComposeModal` props | `ComposeModal.svelte:5–19` | Unchanged |
| Home entry: embedded `ComposeWorkspace` | `+page.svelte:33–37` | Unchanged |
| `api.content.compose()` | Called from `buildComposeRequest` | Unchanged |
| `api.content.schedule()` | Called in `+page.svelte:13` | Unchanged |
| `api.assist.improve()` | Called in inline assist | Unchanged |
| `api.assist.thread()` | Called in from-notes generation | Unchanged |
| `api.media.upload()` | Called in `TweetEditor` | Unchanged |

## Timeline

| Session | Focus | Scope |
|---------|-------|-------|
| S1 (this) | Benchmark + charter | Documentation only |
| S2 | Live canvas surface | Restyle editor, reduce chrome, simplify headers |
| S3 | Dedicated X preview | New `PreviewOverlay`, remove inline rail, wire shortcuts |
| S4 | Shortcuts and safety | `Cmd+J` undo, audit all shortcuts, update catalog |
| S5 | Validation and release | QA, accessibility, mobile, final handoff |
