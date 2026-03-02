# Benchmark Notes: Current Composer vs. Typefully-Class Target

## Surfaces Audited

### 1. Home Embedded Composer
- Entry: `+page.svelte` renders `ComposeWorkspace` with `embedded=true`
- Container: `.home-composer` wraps at `max-width: 860px`, centered, fills viewport minus 120px
- Header: `HomeComposerHeader` with handle, post count, schedule pill, publish pill, 4 icon tools
- Canvas: `ComposerCanvas` with 20px horizontal padding, 4px top padding on first child
- Editor: `TweetEditor` (tweet mode) or `ThreadFlowLane` (thread mode)
- Inline preview: `ThreadPreviewRail` below editor, toggled by `previewCollapsed` state
- Inspector: 260px side panel via `ComposerCanvas.canvas-inspector`

### 2. Modal Composer
- Entry: `ComposeModal.svelte` renders `ComposeWorkspace` with `embedded=false`
- Shell: `ComposerShell` — fixed backdrop, 640px modal (900px with inspector), `border-radius: 12px`, `box-shadow: 0 16px 48px`
- Header: `ComposerHeaderBar` with close, preview, inspector, and focus-mode buttons
- Same canvas/editor/preview internals as home composer
- Submit button: sticky-bottom `submit-pill` inside `ComposerCanvas`

### 3. Focus Mode (Modal Variant)
- Triggered by `Cmd+Shift+F` or command palette; sets `focusMode=true`
- `ComposerShell.modal.focus-mode` goes full-viewport: 100vw/100vh, border-radius 0
- Same internals; only the modal chrome changes

---

## Gap 1: Editor Does Not Resemble the Post

### Current TweetEditor (`TweetEditor.svelte:108–126`)
- Standard bordered `<textarea>` with:
  - `border: 1px solid var(--color-border)` — visible box around content
  - `border-radius: 6px` — rounded form-input look
  - `background: var(--color-base)` — distinct from surrounding surface
  - `padding: 10px 12px` — tight, form-like padding
  - `font-size: 14px` — smaller than X's post rendering (~15px)
  - `line-height: 1.5` — tighter than X's post spacing
  - `rows={4}` — fixed initial height, manual `resize: vertical`
- Character counter always visible below (`{tweetChars}/{TWEET_MAX}`)
- Media attach section always rendered when `canAttachMore` is true (lines 149–167)
- Placeholder: generic "What's on your mind?"

### Current ThreadFlowCard (`ThreadFlowCard.svelte:117–128`)
- Borderless textarea: `border: none`, `background: transparent`, `padding: 10px 0`
- Better than TweetEditor but still:
  - `font-size: 14px` (same undersized font)
  - `min-height: 72px` with auto-resize
  - Wrapped in heavy card chrome (see Gap 2)

### X Post Reference
- X renders posts at approximately 15px font, `-apple-system` / system font stack
- No visible container border around individual tweets
- Avatar gutter (48px circle) + body content, no character counter visible
- Line-height approximately 1.4 (20px for 15px text)

### Gap Summary
| Property | TweetEditor | ThreadFlowCard | X Post |
|----------|-------------|----------------|--------|
| Font size | 14px | 14px | ~15px |
| Border | 1px solid | none | none |
| Background | distinct (base) | transparent | transparent |
| Padding | 10px 12px | 10px 0 | 0 (content flows) |
| Char counter | always visible | always visible (separator) | not shown |
| Container feel | form input | card-in-lane | inline content |

---

## Gap 2: Chrome Density

### HomeComposerHeader (`HomeComposerHeader.svelte:63–148`)
Always-visible elements in a single 10px-padded row:
1. `@handle` text (mono font, 13px)
2. Dot separator
3. Post count label ("1 tweet" / "3 posts")
4. Content indicator dot (green when has content)
5. Schedule pill button (icon + label, 36px height)
6. Publish pill button (icon + label, 36px height)
7. Preview toggle icon button (Eye/EyeOff)
8. AI improve icon button (Sparkles)
9. Inspector toggle icon button (PanelRight)
10. Command palette icon button (Search)

Total: 10 interactive/informational elements in the header. Cognitive load is high before the user starts writing.

### ComposerHeaderBar (Modal, `ComposerHeaderBar.svelte:23–73`)
4 buttons in a row: close, preview, inspector, focus-mode. Lighter than home but still present.

### ThreadFlowCard Separator (`ThreadFlowCard.svelte:132–168`)
Per-card separator row (24px height) contains:
- Character count (`{charCount}/{MAX_TWEET_CHARS}`, always shown)
- Drag handle (GripVertical icon, hidden until hover)
- Merge button (hidden until hover, only when >2 blocks)
- Remove button (hidden until hover, only when >2 blocks)

Plus between-zone "+" button (20px height, hidden until hover):
- Total non-writing chrome between cards: **44px minimum** (24px separator + 20px between-zone)

### ThreadFlowLane Spine (`ThreadFlowLane.svelte:376–432`)
- Vertical `.lane-spine`: 1px line, left 19px, spans top-to-bottom
- Per-card `.spine-dot`: 10px circle at `left: -28px, top: 14px`, with border + background transitions
- Entire lane has `padding-left: 40px` to accommodate spine

### Inline Preview Section (`ComposeWorkspace.svelte:473–487`)
- `margin-top: 16px`, `padding-top: 16px`, `border-top: 1px solid` — adds separator line
- Toggle button: "Preview" label + "Hide" text, 11px uppercase
- Full `ThreadPreviewRail` rendered inline below editor

### Inspector Side Panel (`ComposerCanvas.svelte:86–93`)
- 260px fixed width, border-left separator, background tint
- Contains schedule, voice context, AI, and notes sections
- Always rendered when open (not collapsible per-section)

### Chrome Density Summary
| Element | Height/Width | Always Visible | Purpose |
|---------|-------------|----------------|---------|
| Home header row | ~56px (36px pills + 10px padding × 2) | Yes | Metadata + actions |
| Modal header bar | ~44px (28px buttons + 8px padding × 2) | Yes | Close + toggles |
| Char counter (tweet) | ~18px | Yes | Character limit |
| Card separator | 24px per card | Yes (count); hover (tools) | Count + actions |
| Between-zone | 20px per card | Hover only | Add card |
| Preview section | ~16px toggle + content | When open | Inline preview |
| Inspector panel | 260px wide | When open | Schedule/AI/voice |

---

## Gap 3: Preview Is Inline, Not Dedicated

### Current Behavior
- `ThreadPreviewRail` (`ThreadPreviewRail.svelte:1–90`) renders inside `ComposerCanvas` below the editor
- Toggled by `previewCollapsed` state (`ComposeWorkspace.svelte:63`)
- Toggle bound to `Cmd+Shift+P` (`ComposeWorkspace.svelte:302`) and eye icon button
- When visible: shows "Preview" header label + full `TweetPreview` components
- Preview scroll area: `max-height: 400px`, overflow-y auto
- In tweet mode: shows single `TweetPreview`; in thread mode: shows all non-empty blocks

### Problems
1. **Competes for vertical space**: Preview and editor share the same scrollable column. Writing a long thread pushes the preview off-screen.
2. **Not X-accurate**: `TweetPreview` uses 13px font, simplified layout — does not match X's actual rendering.
3. **Duplicates content inline**: Shows the same text below where the user just typed it, adding visual noise without adding insight.
4. **"Preview" label + "Hide" button**: Additional chrome that says "this is separate from your content" — the opposite of a WYSIWYG feel.

### Target Behavior
- The compose surface itself should approximate the rendered post (fonts, spacing, visual rhythm)
- X-accurate preview lives in a dedicated full-screen overlay, opened via `Cmd+Shift+P` or button
- No inline preview panel — the editor IS the preview during writing

---

## Gap 4: Visual Rhythm

### TweetEditor
- `rows={4}` textarea: fixed 4 rows (~84px at 14px + 1.5 line-height), then manual resize
- No avatar gutter during writing (present only in `TweetPreview`)
- Character counter immediately below textarea with 4px margin
- Media thumbnails (80px × 80px) below with 12px margin

### ThreadFlowCard
- `min-height: 72px` auto-resizing textarea
- Spine dot at top-left creates visual anchor per card
- Separator (24px) + between-zone (20px) = 44px gap between writing areas
- Writing flow: type → glance at separator chrome → next card. The chrome interrupts flow.

### What Typefully-Class Feels Like
- Writing area fills the surface with generous vertical space
- Cards flow seamlessly with minimal dividers (subtle line or gap, not action toolbars)
- Character count appears only when approaching the limit (e.g., >240 chars)
- Media previews are contextual, not always-rendered sections
- The overall sensation: "I am writing my post" not "I am filling out a form"

---

## Gap 5: Shortcut Safety

### `Cmd+J` (Inline Assist) — Full Trace

**Tweet Mode** (`ComposeWorkspace.svelte:298,331–354`):
1. `handleKeydown` catches `Cmd+J`, calls `handleInlineAssist()`
2. Gets `textarea` via `document.querySelector('.compose-input')`
3. Reads `selectionStart` and `selectionEnd`
4. **If text is selected** (`start !== end`): improves only the selection, splices result back
5. **If no text is selected** (`start === end`): sets `selectedText = tweetText` (entire content), then `tweetText = result.content` — **replaces ALL content**
6. No `undoSnapshot` is taken (that mechanism only covers "from notes" generation at `ComposeWorkspace.svelte:356–377`)
7. No confirmation dialog, no visual indicator of scope

**Thread Mode** (`ThreadFlowLane.svelte:321–346`):
1. `ComposeWorkspace.handleInlineAssist()` delegates to `threadFlowRef?.handleInlineAssist(voiceCue)`
2. Gets focused block's textarea and selection range
3. Same logic: if no selection, `selectedText = block.text` → entire block replaced
4. No undo snapshot, no confirmation

**Risk**: User presses `Cmd+J` while cursor is in the textarea but nothing is selected. The entire tweet or focused thread block is silently replaced by an AI rewrite. The old content is gone with no undo path.

### Other Shortcut Concerns
- `Cmd+Enter`: Overloaded — submits in tweet mode, splits at cursor in thread mode. Documented in `SHORTCUT_CATALOG` as "Publish (tweet) / Split below (thread)". While intentional, the dual behavior can surprise users switching between modes.
- `Cmd+Shift+Enter`: Always submits. This is safe and unambiguous.
- `Cmd+D` (thread duplicate): Creates a copy of the focused block. Low risk since content is added, not removed.

---

## Summary: Priority Gaps

| # | Gap | Severity | Fix Session |
|---|-----|----------|-------------|
| 1 | Editor looks like a form input, not a post | High | S2 |
| 2 | Chrome density overwhelms the writing surface | High | S2 |
| 3 | Preview is inline instead of dedicated overlay | Medium | S3 |
| 4 | Thread cards have heavy separator chrome | Medium | S2 |
| 5 | `Cmd+J` silently replaces content without undo | High | S4 |
| 6 | Character counter always visible (noise) | Low | S2 |
| 7 | Visual rhythm doesn't match post rendering | Medium | S2 |
