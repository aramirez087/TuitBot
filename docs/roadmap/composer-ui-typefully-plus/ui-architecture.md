# Composer UI Architecture

Component hierarchy, data flow, interaction model, and specifications for the thread-first unibody redesign.

## Component Hierarchy

### Current Structure

```
ComposeModal.svelte                    ← Orchestrator: state, autosave, AI, submit
├── ComposerShell.svelte               ← Modal chrome: header, tabs, footer
│   └── {children}
│       ├── VoiceContextPanel.svelte   ← Tone cue input (always above editor)
│       ├── compose-layout (grid)
│       │   ├── editor-pane
│       │   │   ├── TweetEditor.svelte ← Single-tweet textarea + media
│       │   │   └── ThreadComposer.svelte ← Card-per-tweet editor
│       │   │       ├── tweet-card × N
│       │   │       │   ├── card-gutter (number, drag handle)
│       │   │       │   ├── card-textarea
│       │   │       │   ├── MediaSlot.svelte
│       │   │       │   └── ThreadCardActions.svelte
│       │   │       └── add-card-btn
│       │   └── preview-pane
│       │       └── ThreadPreviewRail.svelte
│       │           └── TweetPreview.svelte × N
│       ├── FromNotesPanel.svelte
│       ├── undo-banner
│       └── TimePicker.svelte
└── CommandPalette.svelte
```

### Target Structure (After Session 4)

```
ComposeModal.svelte                    ← Orchestrator: state, autosave, AI, submit (unchanged role)
├── ComposerShell.svelte               ← Slimmed: just backdrop + modal container + recovery banner
│   └── {children}
│       ├── ComposerHeaderBar.svelte   ← NEW: close, preview toggle, focus toggle (no title, no tabs)
│       ├── ComposerCanvas.svelte      ← NEW: writing area + optional inspector layout
│       │   ├── canvas-main
│       │   │   ├── ThreadFlowEditor.svelte ← NEW: unibody editor for both tweets and threads
│       │   │   │   ├── editable region × N (no borders — text between separators)
│       │   │   │   ├── ThreadSeparator.svelte × (N-1) ← NEW: visual divider + char count + drag handle
│       │   │   │   └── inline MediaSlot per segment
│       │   │   └── floating-submit-btn
│       │   └── canvas-inspector (collapsible)
│       │       └── ComposerInspector.svelte ← NEW: schedule, voice, notes, preview
│       │           ├── TimePicker.svelte
│       │           ├── VoiceContextPanel.svelte
│       │           └── FromNotesPanel.svelte
│       └── inline-preview (when toggled, replaces canvas-main)
│           └── TweetPreview.svelte × N
└── CommandPalette.svelte
```

### Migration Path (Session by Session)

**Session 2** (Shell Redesign):
- Create `ComposerHeaderBar.svelte` and `ComposerCanvas.svelte`
- Refactor `ComposerShell.svelte` to use them
- `ThreadComposer` and `TweetEditor` still work inside the new canvas — no editor changes
- Remove mode tabs from visible UI; keep mode state in ComposeModal for submission

**Session 3** (Thread Interactions):
- Create `ThreadFlowEditor.svelte` and `ThreadSeparator.svelte`
- Replace `TweetEditor` + `ThreadComposer` with `ThreadFlowEditor` inside `ComposerCanvas`
- `ThreadFlowEditor` emits `ThreadBlock[]` — parent contract unchanged

**Session 4** (Inspector):
- Create `ComposerInspector.svelte`
- Move `VoiceContextPanel`, `TimePicker`, `FromNotesPanel` into the inspector
- Add toggle (Cmd+I) to show/hide inspector rail

## Data Flow

```
User types in ThreadFlowEditor
  ↓
ThreadFlowEditor parses text + separators into ThreadBlock[]
  ↓
ThreadFlowEditor calls onchange(blocks: ThreadBlock[])
  ↓
ComposeModal receives blocks, updates threadBlocks state
  ↓
ComposeModal auto-saves { mode, tweetText, blocks, timestamp } to localStorage
  ↓
On submit: ComposeModal reads threadBlocks, builds ComposeRequest, calls onsubmit()
```

### Key Invariant

`ThreadFlowEditor` emits `ThreadBlock[]` on every change, exactly matching the current `ThreadComposer` contract:

```typescript
interface ThreadBlock {
  id: string;
  text: string;
  media_paths: string[];
  order: number;
}
```

A single-tweet draft emits one block. A thread emits 2+ blocks. The parent (`ComposeModal`) determines `mode` from block count:
- 1 block → `mode = 'tweet'`
- 2+ blocks → `mode = 'thread'`

### Autosave Format (Unchanged)

```json
{
  "mode": "thread",
  "tweetText": "",
  "blocks": [
    { "id": "uuid-1", "text": "First tweet", "media_paths": [], "order": 0 },
    { "id": "uuid-2", "text": "Second tweet", "media_paths": [], "order": 1 }
  ],
  "timestamp": 1709145600000
}
```

The `tweetText` field is kept for backwards compatibility with existing autosaved drafts. `ThreadFlowEditor` can populate it from `blocks[0].text` when there's only one block.

## New Component Specifications

### ComposerHeaderBar.svelte

**Role**: Minimal header bar replacing the current header + tabs.

**Props**:
```typescript
{
  focusMode: boolean;
  previewMode: boolean;
  ontogglefocus: () => void;
  ontogglepreview: () => void;
  onclose: () => void;
}
```

**Rendered elements** (left to right):
- Close button (X icon) — left-aligned
- Spacer
- Preview toggle button (Eye icon) — right-aligned, shows active state when preview is on
- Focus mode toggle button (Maximize/Minimize icon)

**Not rendered**: "Compose" title, date subtitle, mode tabs, any text labels.

**Styling**: No bottom border. Padding: 8px 16px. Background: transparent (inherits from modal).

### ComposerCanvas.svelte

**Role**: Layout wrapper that arranges the writing area and optional inspector rail.

**Props**:
```typescript
{
  inspectorOpen: boolean;
  children: Snippet;          // main writing area content
  inspector?: Snippet;        // inspector rail content
}
```

**Layout**:
- Default: `canvas-main` fills full width
- When inspector is open: `canvas-main` + `canvas-inspector` in a CSS grid (auto 1fr / 280px)
- Mobile (<768px): inspector becomes an overlay/drawer, not a grid column

**Styling**: Padding: 20px. No borders between canvas and inspector on desktop (inspector has its own left border).

### ThreadFlowEditor.svelte (Session 3)

**Role**: Unibody writing surface replacing both `TweetEditor` and `ThreadComposer`.

**Props**:
```typescript
{
  initialBlocks?: ThreadBlock[];
  onchange: (blocks: ThreadBlock[]) => void;
  onvalidchange: (valid: boolean) => void;
}
```

**Internal state**:
- Array of segments, each with an id, text content, and media_paths
- Active segment index (where the cursor is)
- Separator positions (derived from segment boundaries)

**Key behaviors**:
- Renders as a series of `contenteditable` regions separated by `ThreadSeparator` components
- `Cmd+Enter` at any point inserts a new separator, splitting the current segment at the cursor
- `Backspace` at the start of a segment (position 0) merges it with the previous segment
- Emits `ThreadBlock[]` on every change via the `onchange` callback
- Validates: segments with content >= 2 for thread validity; single segment is always valid (tweet mode)
- Each segment has a char count indicator that appears near the separator (not a separate footer)

**Fallback strategy**: If `contenteditable` proves too fragile (selection bugs, IME issues, mobile keyboard problems), fall back to stacked `<textarea>` elements with visual connectors (thin lines between them, no card borders). The interaction model stays the same; only the DOM implementation changes.

### ThreadSeparator.svelte (Session 3)

**Role**: Visual divider between tweet segments in the unibody editor.

**Props**:
```typescript
{
  charCount: number;
  maxChars: number;
  index: number;
  total: number;
  ondragstart: () => void;
  ondragend: () => void;
  onremove: () => void;
}
```

**Rendered elements**:
- Thin horizontal line (1px, `var(--color-border-subtle)`)
- Character count badge (left side): `{charCount}/{maxChars}` for the segment above
- Drag handle (center, visible on hover): grip icon for reordering
- Remove button (right side, visible on hover): merges the segment below with the one above

**Styling**: 32px total height. Line is vertically centered. Badge and handle are 20px tall, centered on the line. Hover reveals drag handle and remove button with 150ms fade.

### ComposerInspector.svelte (Session 4)

**Role**: Collapsible right-side rail for secondary controls.

**Props**:
```typescript
{
  open: boolean;
  children: Snippet;
}
```

**Sections** (stacked vertically):
1. Schedule (TimePicker)
2. Voice context (VoiceContextPanel)
3. From notes (FromNotesPanel)

**Toggle**: Cmd+I keyboard shortcut or a button in ComposerHeaderBar.

**Mobile behavior**: Slides in as a bottom drawer overlay. Backdrop click or swipe-down to dismiss.

**Styling**: Width: 280px. Left border: 1px solid `var(--color-border-subtle)`. Padding: 16px. Background: `var(--color-surface)`.

## Interaction State Machine

```
                    ┌──────────────┐
                    │     IDLE     │  (modal closed)
                    └──────┬───────┘
                           │ open modal
                           ▼
                    ┌──────────────┐
                    │   WRITING    │  (single segment, no separators)
                    │  (tweet)     │
                    └──────┬───────┘
                           │ Cmd+Enter (add separator)
                           ▼
                    ┌──────────────┐
                    │   WRITING    │  (2+ segments, separators visible)
                    │  (thread)    │◄──── remove last separator returns to tweet
                    └──────┬───────┘
                           │ toggle preview
                           ▼
                    ┌──────────────┐
                    │   PREVIEW    │  (editor replaced by TweetPreview cards)
                    │              │◄──── toggle preview returns to writing
                    └──────┬───────┘
                           │ submit
                           ▼
                    ┌──────────────┐
                    │  SUBMITTING  │  (disabled state, spinner)
                    └──────┬───────┘
                           │ success / failure
                           ▼
                    ┌──────────────┐
                    │     IDLE     │  (modal closes on success, error shown on failure)
                    └──────────────┘
```

Focus mode is orthogonal — it can be toggled in any writing/preview state and only affects the modal's fullscreen styling.

Inspector open/close is also orthogonal — it affects layout but not the writing/preview/submit state machine.

## CSS/Token Strategy

### No New Design Tokens

The existing token set in `app.css` covers all needs:
- `--color-surface`, `--color-base` for backgrounds
- `--color-border`, `--color-border-subtle` for separators
- `--color-accent` for active states and primary actions
- `--color-text`, `--color-text-muted`, `--color-text-subtle` for typography
- `--font-sans`, `--font-mono` for typefaces

### New CSS Patterns

1. **Floating submit button**: `position: sticky; bottom: 16px;` inside the canvas-main area. Uses `--color-accent` background, `#fff` text. 40px height, pill-shaped (`border-radius: 20px`).

2. **Separator line**: `border-top: 1px solid var(--color-border-subtle)` with `margin: 8px 0`. Char count badge uses `--color-text-subtle` with `--font-mono`.

3. **Inspector rail**: `border-left: 1px solid var(--color-border-subtle)` on desktop. On mobile, uses fixed positioning with backdrop overlay.

4. **Borderless textarea/contenteditable**: Remove the bordered box appearance of the current card textareas. Use transparent background, no border, padding only. Focus indication via a subtle left accent bar (`border-left: 2px solid var(--color-accent)`) rather than a full border.

## Responsive Breakpoints

| Breakpoint | Layout Change |
|------------|---------------|
| `> 768px` | Full layout: canvas + inspector rail side-by-side |
| `641–768px` | Canvas full-width; inspector as overlay drawer |
| `≤ 640px` | Modal goes full-viewport; inspector as bottom sheet; textarea font bumps to 16px (prevents iOS zoom) |

### Mobile-Specific Behaviors
- Inspector rail collapsed by default; opens as bottom drawer
- Floating submit button anchored to bottom-right with safe area inset
- Touch targets: all interactive elements ≥ 44px
- Thread separators: drag handle hidden on touch (use long-press to enter reorder mode, or Alt+Arrow in on-screen keyboard)

## Files Affected Per Session

### Session 2: Shell Redesign
| Action | File | Estimated Lines |
|--------|------|-----------------|
| Create | `composer/ComposerHeaderBar.svelte` | ~80 |
| Create | `composer/ComposerCanvas.svelte` | ~100 |
| Modify | `composer/ComposerShell.svelte` | Reduce from 516 to ~200 (delegate to HeaderBar + Canvas) |
| Modify | `ComposeModal.svelte` | ~30 lines changed (remove mode tabs prop, adjust layout) |

### Session 3: Thread Interactions
| Action | File | Estimated Lines |
|--------|------|-----------------|
| Create | `composer/ThreadFlowEditor.svelte` | ~350 |
| Create | `composer/ThreadSeparator.svelte` | ~120 |
| Modify | `ComposeModal.svelte` | ~50 lines changed (swap ThreadComposer/TweetEditor for ThreadFlowEditor) |
| Deprecate | `ThreadComposer.svelte` | Kept but no longer imported by ComposeModal |
| Deprecate | `composer/TweetEditor.svelte` | Kept but no longer imported by ComposeModal |

### Session 4: Inspector & Polish
| Action | File | Estimated Lines |
|--------|------|-----------------|
| Create | `composer/ComposerInspector.svelte` | ~150 |
| Modify | `composer/ComposerCanvas.svelte` | ~30 lines changed (inspector slot integration) |
| Modify | `composer/ComposerHeaderBar.svelte` | ~20 lines changed (inspector toggle button) |
| Modify | `ComposeModal.svelte` | ~40 lines changed (move voice/notes/schedule to inspector) |
| Modify | `CommandPalette.svelte` | ~10 lines changed (add inspector toggle action) |
