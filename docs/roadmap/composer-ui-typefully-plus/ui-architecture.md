# Composer UI Architecture

Component hierarchy, data flow, interaction model, and specifications for the thread-first unibody redesign.

## Component Hierarchy

### Pre-Phase-1 Structure (historical)

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

### Current Structure (After Phase 1, shipped)

```
ComposeModal.svelte (679 lines)        ← Orchestrator: state, autosave, AI, submit
├── ComposerShell.svelte (151 lines)   ← Backdrop + modal container + recovery banner
│   └── {children}
│       ├── ComposerHeaderBar.svelte (123 lines) ← Close, preview, inspector, focus toggles
│       ├── ComposerCanvas.svelte (164 lines)    ← Writing area + inspector flex layout
│       │   ├── canvas-main
│       │   │   ├── TweetEditor.svelte (327 lines) ← Single-tweet textarea + media
│       │   │   ├── ThreadComposer.svelte (433 lines) ← Thread orchestrator
│       │   │   │   └── ThreadFlowLane.svelte (137 lines) ← Thread card container
│       │   │   │       └── ThreadFlowCard.svelte (295 lines) × N ← Borderless left-accent cards
│       │   │   ├── ThreadPreviewRail.svelte      ← Collapsible preview section
│       │   │   ├── undo-banner
│       │   │   └── floating-submit-pill (sticky)
│       │   └── canvas-inspector (260px collapsible rail)
│       │       ├── TimePicker.svelte
│       │       ├── VoiceContextPanel.svelte
│       │       ├── FromNotesPanel.svelte
│       │       └── AI action buttons
│       └── ComposerInspector.svelte (116 lines, mobile drawer only)
└── CommandPalette.svelte (345 lines)
```

### Target Structure (After Phase 2)

```
+page.svelte (home route)              ← Dual-surface renderer
├── {#if homeSurface === 'composer'}
│   └── HomeComposerSurface.svelte     ← Full-page compose wrapper (no modal shell)
│       ├── HomeComposerHeader.svelte  ← Action cluster: Schedule pill, Publish pill, icon tools
│       └── ComposeWorkspace.svelte    ← SHARED orchestrator (extracted from ComposeModal)
│           ├── ComposerCanvas.svelte  ← Reused: writing area + inspector flex layout
│           │   ├── canvas-main
│           │   │   ├── TweetEditor / ThreadComposer (mode-switched)
│           │   │   │   └── ThreadFlowLane.svelte (with avatar spine modifications)
│           │   │   │       └── ThreadFlowCard.svelte (with avatar puck)
│           │   │   ├── ThreadPreviewRail.svelte
│           │   │   └── undo-banner
│           │   └── canvas-inspector
│           │       ├── TimePicker.svelte
│           │       ├── VoiceContextPanel.svelte
│           │       ├── FromNotesPanel.svelte
│           │       └── AI action buttons
│           └── CommandPalette.svelte
├── {#if homeSurface === 'analytics'}
│   └── AnalyticsDashboard.svelte      ← Extracted from current +page.svelte
│       ├── StatCard × 4
│       ├── FollowerChart
│       ├── TopTopics
│       └── RecentPerformance
└── (surface switcher lives in Settings)

ComposeModal.svelte (modal, other routes) ← Thin shell delegating to shared workspace
├── ComposerShell.svelte               ← Backdrop + dialog + recovery banner
│   └── ComposerHeaderBar.svelte       ← Close, preview, inspector, focus toggles
│       └── ComposeWorkspace.svelte    ← SAME shared orchestrator
└── (onclose bound to modal close)
```

### Migration Path

**Phase 1 (Sessions 2–5, shipped):**
- Created `ComposerHeaderBar`, `ComposerCanvas`, `ComposerInspector`, `ThreadFlowLane`, `ThreadFlowCard`, `ThreadPreviewRail`
- Refactored `ComposerShell` to thin wrapper
- Moved inspector to collapsible rail with `Cmd+I` toggle
- `ComposeModal` remains the single orchestrator (679 lines)

**Phase 2 (Sessions 7–9, planned):**

**Session 7** (ComposeWorkspace Extraction):
- Extract state management and handlers from `ComposeModal.svelte` into `ComposeWorkspace.svelte`
- `ComposeWorkspace` encapsulates: mode, text/blocks, autosave, AI handlers, submit flow, inspector content
- `ComposeModal` becomes a thin wrapper: `ComposerShell` + `ComposerHeaderBar` + `ComposeWorkspace` with `onclose` bound
- External API of `ComposeModal` (`open`, `onclose`, `onsubmit`, `schedule`, `prefillTime`, `prefillDate`) is preserved

**Session 8** (Full-Page Composer Home):
- Create `HomeComposerSurface.svelte` — full-page wrapper using `ComposeWorkspace`
- Create `HomeComposerHeader.svelte` — action cluster with Schedule/Publish pills and icon tools
- Modify `+page.svelte` to render dual surfaces based on `home_surface` preference
- Add avatar spine to `ThreadFlowLane`/`ThreadFlowCard` for the full-page context

**Session 9** (Settings Override & Polish):
- Extract current analytics into `AnalyticsDashboard.svelte`
- Add `home_surface` dropdown to Settings page
- Update `Sidebar.svelte` — change "Dashboard" label to "Home"
- Responsive QA for full-page composer at all breakpoints

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

## Home Surface Architecture

### Dual-Surface Home Renderer

The `+page.svelte` home route renders one of two surfaces based on a persisted preference:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { persistGet } from '$lib/stores/persistence';
  import HomeComposerSurface from '$lib/components/HomeComposerSurface.svelte';
  import AnalyticsDashboard from '$lib/components/AnalyticsDashboard.svelte';

  let homeSurface = $state<'composer' | 'analytics'>('composer'); // sync default

  onMount(async () => {
    homeSurface = await persistGet('home_surface', 'composer');
  });
</script>

{#if homeSurface === 'composer'}
  <HomeComposerSurface />
{:else}
  <AnalyticsDashboard />
{/if}
```

The synchronous default is `'composer'` to prevent layout shift while the Tauri store loads asynchronously.

### Shared ComposeWorkspace Extraction

`ComposeWorkspace.svelte` is the critical new component. It contains everything that `ComposeModal.svelte` currently orchestrates:

**State extracted from ComposeModal (lines 40–64):**
- `mode: 'tweet' | 'thread'`
- `tweetText`, `threadBlocks`, `threadValid`
- `selectedTime`, `submitting`, `submitError`
- `attachedMedia`, `focusMode`, `paletteOpen`
- `voiceCue`, `previewCollapsed`, `inspectorOpen`
- `showFromNotes`, `assisting`
- `undoSnapshot`, `showUndo`

**Handlers extracted:**
- `handleSubmit()` — builds `ComposeRequest`, calls `onsubmit`
- `handleInlineAssist()` — selection-aware AI improvement
- `handleAiAssist()` — full AI generation/improvement
- `handleGenerateFromNotes()` — notes-to-content pipeline
- `handleUndo()` — revert notes generation
- `autoSave()`, `clearAutoSave()`, `checkRecovery()`, `recoverDraft()` — autosave lifecycle
- `toggleInspector()`, `togglePreview()`, `toggleFocusMode()`
- `handlePaletteAction()` — command palette dispatch

**Props interface:**
```typescript
{
  schedule: ScheduleConfig | null;
  onsubmit: (data: ComposeRequest) => void;
  onclose?: () => void;         // absent in full-page mode
  prefillTime?: string | null;
  prefillDate?: Date | null;
  embedded?: boolean;           // true for full-page, false for modal
}
```

When `embedded` is true:
- No `onclose` callback (no modal to close)
- Escape key does not close (no modal to dismiss)
- Focus mode is not available (already full-page)
- Recovery banner appears inline instead of in a shell header

When `embedded` is false (modal context):
- `onclose` is required and bound to modal close
- Escape chain: fromNotes → mobile inspector → focus mode → close modal
- Recovery banner delegates to `ComposerShell`

### `home_surface` Preference Data Flow

```
App loads
  ↓
+page.svelte onMount()
  ↓
persistGet('home_surface', 'composer') → async read from Tauri plugin-store
  ↓
Sets local state: homeSurface = result
  ↓
Renders HomeComposerSurface or AnalyticsDashboard

Settings page
  ↓
User changes "Home Surface" dropdown
  ↓
persistSet('home_surface', value) → async write to Tauri plugin-store
  ↓
Next navigation to / picks up new value via persistGet
```

No backend API is involved. The preference is stored in `ui-state.json` via `@tauri-apps/plugin-store`, with in-memory fallback for browser-only mode. This matches the existing pattern used by `sidebar_collapsed`.

### HomeComposerHeader Action Cluster

The full-page home surface uses a different header than the modal. Instead of close/preview/inspector/focus toggles, it has:

**Left side:**
- (empty — no close button needed)

**Right side (action cluster):**
1. `Schedule` pill — warm color (`--color-warning` or orange-tinted). Opens time picker inline or in inspector.
2. `Publish` pill — cool color (`--color-accent`). Submits immediately (or routes to approval queue).
3. Icon tools (quiet, 28px buttons):
   - Preview toggle (Eye)
   - Inspector toggle (PanelRight)
   - AI assist (Sparkles)
   - Command palette (Search or `⌘K` badge)

This cluster replaces the floating submit pill from `ComposerCanvas` when in full-page mode. The `ComposerCanvas` submit pill is conditionally hidden via the `embedded` prop.

### Avatar Spine for Full-Page Thread Lane

In the full-page context, the thread lane gains a left "spine" with avatar pucks:

- Each `ThreadFlowCard` shows a small avatar circle (16–20px) aligned to the left of the writing area
- A thin vertical line (1px, `--color-border-subtle`) connects the avatar circles
- The first card's avatar sits at the top of the spine; subsequent cards' avatars align to their writing area tops
- The spine replaces the current `border-left: 2px solid` accent bar on `ThreadFlowCard`
- The avatar image comes from the user's X profile (already available in the accounts store)

This is a CSS/template enhancement to `ThreadFlowLane` and `ThreadFlowCard`, gated by an `avatarSpine` prop so the modal context can keep the simpler accent-bar style.

### Responsive Behavior for Full-Page Composer

| Breakpoint | Layout |
|------------|--------|
| `> 1120px` | 860px writing lane centered + 260px inspector rail visible by default |
| `769–1120px` | Writing lane fills available width (with padding) + inspector as collapsible overlay |
| `641–768px` | Writing lane full-width + inspector as bottom drawer |
| `≤ 640px` | Full-viewport writing lane + inspector as bottom sheet; textarea font 16px |

The full-page composer does not have a modal boundary, so it uses the `main-content` area dimensions directly. The `max-width: 860px; margin: 0 auto;` centering on the writing lane provides the calm, focused canvas.

## Files Affected Per Session

### Phase 1 Sessions (shipped)

| Session | Files Created/Modified |
|---------|----------------------|
| **2** | `ComposerHeaderBar.svelte`, `ComposerCanvas.svelte`, refactored `ComposerShell.svelte`, `ComposeModal.svelte` |
| **3** | `ThreadFlowLane.svelte`, `ThreadFlowCard.svelte`, `ThreadComposer.svelte`, `ComposeModal.svelte` |
| **4** | `ComposerInspector.svelte`, `ThreadPreviewRail.svelte`, `ComposerCanvas.svelte`, `ComposerHeaderBar.svelte`, `ComposeModal.svelte` |
| **5** | `release-readiness.md` (no code changes) |

### Phase 2 Sessions (planned)

#### Session 7: ComposeWorkspace Extraction
| Action | File | Estimated Lines |
|--------|------|-----------------|
| Create | `composer/ComposeWorkspace.svelte` | ~400 (state, handlers, template with Canvas + inspector snippets) |
| Modify | `ComposeModal.svelte` | Reduce from 679 to ~80 (thin wrapper: Shell + HeaderBar + ComposeWorkspace) |
| Test | Verify modal still opens, autosaves, submits, and recovers exactly as before |

#### Session 8: Full-Page Composer Home
| Action | File | Estimated Lines |
|--------|------|-----------------|
| Create | `HomeComposerSurface.svelte` | ~120 (full-page wrapper, no modal chrome) |
| Create | `HomeComposerHeader.svelte` | ~150 (action cluster with Schedule/Publish pills) |
| Modify | `+page.svelte` (home route) | ~40 (dual-surface renderer) |
| Modify | `ThreadFlowLane.svelte` | ~20 (avatar spine prop) |
| Modify | `ThreadFlowCard.svelte` | ~30 (avatar puck, spine line CSS) |

#### Session 9: Settings Override & Polish
| Action | File | Estimated Lines |
|--------|------|-----------------|
| Create | `AnalyticsDashboard.svelte` | ~100 (extract from current +page.svelte) |
| Modify | `settings/+page.svelte` | ~25 (add home-surface toggle in new Appearance section) |
| Modify | `Sidebar.svelte` | ~5 (rename "Dashboard" to "Home") |
| Test | Full responsive QA, preference persistence, surface switching |
