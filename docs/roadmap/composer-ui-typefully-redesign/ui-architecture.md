# UI Architecture: Composer Redesign

## Component Tree: Current State

```
+page.svelte (embedded=true)  OR  ComposeModal.svelte (embedded=false)
└── ComposeWorkspace (695 lines, orchestrator)
    ├── <svelte:window onkeydown> (global shortcut handler)
    ├── sr-only status announcement
    │
    ├── [if embedded] HomeComposerHeader
    │   ├── @handle · post count · content dot
    │   ├── Schedule pill, Publish pill
    │   └── Icon tools: preview, AI, inspector, palette
    │
    ├── [if !embedded] ComposerShell (modal backdrop + dialog)
    │   ├── ComposerHeaderBar
    │   │   ├── Close button
    │   │   └── Preview, Inspector, Focus-mode buttons
    │   └── ...composeBody snippet
    │
    ├── RecoveryBanner (conditional)
    │
    ├── ComposerCanvas (flex layout: main + inspector)
    │   ├── .canvas-main (scrollable, 20px horizontal padding)
    │   │   ├── [if tweet] TweetEditor
    │   │   │   ├── <textarea class="compose-input"> (bordered, 14px, rows=4)
    │   │   │   ├── Character counter (always visible)
    │   │   │   ├── Media preview grid (if has media)
    │   │   │   └── Media attach section (if canAttachMore)
    │   │   │
    │   │   ├── [if thread] ThreadFlowLane
    │   │   │   ├── .lane-spine (vertical 1px line)
    │   │   │   └── ThreadFlowCard[] (per block)
    │   │   │       ├── .spine-dot (10px circle)
    │   │   │       ├── <textarea class="flow-textarea"> (borderless, 14px, auto-resize)
    │   │   │       ├── MediaSlot
    │   │   │       ├── .card-separator (char count + drag/merge/remove tools, 24px)
    │   │   │       └── .between-zone ("+" button, 20px)
    │   │   │
    │   │   ├── [if preview visible] .preview-section
    │   │   │   ├── Preview/Hide toggle button
    │   │   │   └── ThreadPreviewRail
    │   │   │       └── TweetPreview[] (avatar gutter + body + media)
    │   │   │
    │   │   ├── Undo banner (conditional)
    │   │   └── Submit pill (modal only, sticky bottom)
    │   │
    │   └── .canvas-inspector (260px, border-left)
    │       └── InspectorContent (schedule, voice, AI, notes)
    │
    ├── ComposerInspector (mobile drawer, conditional)
    │   └── InspectorContent (duplicate for mobile)
    │
    ├── ComposerTipsTray (embedded, conditional)
    ├── ComposerPromptCard (embedded, conditional)
    └── CommandPalette (conditional overlay)
```

## Component Tree: Target State

Changes are marked with `[CHANGED]`, `[REMOVED]`, or `[NEW]`.

```
+page.svelte (embedded=true)  OR  ComposeModal.svelte (embedded=false)
└── ComposeWorkspace [CHANGED: slimmed, +previewMode state, Cmd+J undo]
    ├── <svelte:window onkeydown> (updated shortcut handler)
    ├── sr-only status announcement
    │
    ├── [if embedded] HomeComposerHeader [CHANGED: simplified]
    │   ├── @handle · post count
    │   ├── Publish pill
    │   └── Collapsed icon menu: schedule, preview, AI, inspector, palette
    │
    ├── [if !embedded] ComposerShell (unchanged)
    │   ├── ComposerHeaderBar [CHANGED: simplified]
    │   │   ├── Close button
    │   │   └── Preview, Inspector, Focus-mode buttons (unchanged)
    │   └── ...composeBody snippet
    │
    ├── RecoveryBanner (unchanged)
    │
    ├── ComposerCanvas (unchanged structure)
    │   ├── .canvas-main
    │   │   ├── [if tweet] TweetEditor [CHANGED: restyled]
    │   │   │   ├── <textarea> (borderless, transparent bg, 15px, 1.4 lh)
    │   │   │   ├── Character counter [CHANGED: hidden when ≤240]
    │   │   │   ├── Media preview grid (unchanged)
    │   │   │   └── Media attach [CHANGED: compact icon]
    │   │   │
    │   │   ├── [if thread] ThreadFlowLane [CHANGED: lighter spine]
    │   │   │   ├── .lane-spine [CHANGED: thinner, subtler color]
    │   │   │   └── ThreadFlowCard[] [CHANGED: reduced chrome]
    │   │   │       ├── .spine-dot [CHANGED: smaller, lighter]
    │   │   │       ├── <textarea> (15px font, 1.4 lh)
    │   │   │       ├── MediaSlot (unchanged)
    │   │   │       ├── .card-separator [CHANGED: char count hidden ≤240, tools hover-only]
    │   │   │       └── .between-zone [CHANGED: subtler affordance]
    │   │   │
    │   │   ├── [REMOVED] .preview-section + ThreadPreviewRail
    │   │   │
    │   │   ├── Undo banner [CHANGED: also shown for Cmd+J undo]
    │   │   └── Submit pill (modal only, unchanged)
    │   │
    │   └── .canvas-inspector (unchanged)
    │       └── InspectorContent (unchanged)
    │
    ├── PreviewOverlay [NEW: full-screen, conditionally rendered]
    │   ├── .overlay-backdrop (dark, full viewport)
    │   ├── .overlay-container (centered, max-width ~600px)
    │   │   ├── .overlay-header (close button, "Preview" label)
    │   │   └── .overlay-scroll
    │   │       └── TweetPreview[] (existing component, reused)
    │   └── Escape key handler (closes overlay)
    │
    ├── ComposerInspector (mobile, unchanged)
    ├── ComposerTipsTray (embedded, unchanged)
    ├── ComposerPromptCard (embedded, unchanged)
    └── CommandPalette [CHANGED: updated preview action description]
```

## New Component: PreviewOverlay

### File
`dashboard/src/lib/components/composer/PreviewOverlay.svelte`

### Props
```typescript
let {
    mode,
    tweetText,
    threadBlocks,
    attachedMedia,
    handle,
    onclose
}: {
    mode: 'tweet' | 'thread';
    tweetText: string;
    threadBlocks: ThreadBlock[];
    attachedMedia: AttachedMedia[];
    handle: string;
    onclose: () => void;
} = $props();
```

All props are read-only (passed from `ComposeWorkspace`'s `$state`). No internal state duplication. The overlay reads the current reactive values directly.

### Internal Behavior
1. Renders a full-viewport dark backdrop (`position: fixed; inset: 0; background: rgba(0,0,0,0.85)`)
2. Centers a content container (max-width ~600px, matching X's tweet width)
3. Header row: "Preview" label + close button
4. Scrollable body:
   - **Tweet mode**: Single `TweetPreview` with the current `tweetText`, `attachedMedia` mapped to `mediaPaths`, and `handle`
   - **Thread mode**: All non-empty `threadBlocks` sorted by order, each rendered as `TweetPreview` with thread connectors
5. Reuses existing `TweetPreview` and `MediaCropPreview` components without modification

### Keyboard Handling
- Escape: calls `onclose()`
- All other keys: no-op (the overlay is read-only)
- Focus is trapped inside the overlay using the existing `focusTrap` action

### Accessibility
- `role="dialog"`, `aria-modal="true"`, `aria-label="Post preview"`
- Close button has `aria-label="Close preview"`
- Focus moves to close button when overlay opens
- Focus returns to the trigger element when overlay closes

## State Flow

### Current
```
ComposeWorkspace.$state
├── mode: 'tweet' | 'thread'
├── tweetText: string
├── threadBlocks: ThreadBlock[]
├── attachedMedia: AttachedMedia[]
├── previewCollapsed: boolean          ← controls inline preview
├── focusMode, paletteOpen, inspectorOpen...
│
├── TweetEditor receives: text, attachedMedia
├── ThreadFlowLane receives: threadBlocks (via initialBlocks)
├── ThreadPreviewRail receives: mode, tweetText, sortedPreviewBlocks
└── Autosave fires on: mode, tweetText, threadBlocks changes
```

### Target
```
ComposeWorkspace.$state
├── mode: 'tweet' | 'thread'
├── tweetText: string
├── threadBlocks: ThreadBlock[]
├── attachedMedia: AttachedMedia[]
├── previewMode: boolean               ← replaces previewCollapsed
├── focusMode, paletteOpen, inspectorOpen...
├── inlineAssistUndo: { text, blocks } | null  ← NEW for Cmd+J safety
│
├── TweetEditor receives: text, attachedMedia (unchanged)
├── ThreadFlowLane receives: threadBlocks (unchanged)
├── PreviewOverlay receives: mode, tweetText, threadBlocks, attachedMedia, handle (read-only)
└── Autosave fires on: mode, tweetText, threadBlocks changes (unchanged)
```

Key changes:
- `previewCollapsed: boolean` → `previewMode: boolean` (inverted semantics: true = overlay open)
- `ThreadPreviewRail` import and inline rendering removed
- `PreviewOverlay` conditionally rendered when `previewMode === true`
- New `inlineAssistUndo` state for `Cmd+J` undo support (Session 4)

### No Draft Duplication
The `PreviewOverlay` receives props bound to the same `$state` variables in `ComposeWorkspace`. Svelte 5's reactivity means the preview always reflects the current draft. If the user somehow edits while preview is open (not expected, but safe), the preview updates in real time. Closing the preview does not discard or merge state — there is only one copy.

### Autosave Unchanged
The autosave `$effect` at `ComposeWorkspace.svelte:155` watches `mode`, `tweetText`, and `threadBlocks`. Since these states are unchanged, autosave behavior is identical. The `previewMode` boolean is not persisted (preview always starts closed).

### Draft Recovery Unchanged
`checkRecovery()` reads from `AUTOSAVE_KEY`, parses the same `{ mode, tweetText, blocks, timestamp }` format, and restores state. No change needed.

## Hotspot Files by Session

### Session 2: Live Canvas Surface

| File | Lines | Change |
|------|-------|--------|
| `TweetEditor.svelte` | 169–209 | Remove border, transparent bg, font 15px, line-height 1.4, hide char counter ≤240 |
| `TweetEditor.svelte` | 149–167 | Compact media attach (icon only, remove hint text) |
| `ThreadFlowCard.svelte` | 182–252 | Lighter spine-dot (8px, thinner border), 15px font, 1.4 line-height |
| `ThreadFlowCard.svelte` | 253–277 | Hide char counter ≤240, keep separator tools hover-only (already are) |
| `ThreadFlowCard.svelte` | 339–383 | Subtler between-zone (smaller plus circle) |
| `ThreadFlowLane.svelte` | 415–477 | Thinner lane-spine, reduced padding-left |
| `HomeComposerHeader.svelte` | 63–148 | Remove schedule pill from main row, consolidate icon tools |
| `HomeComposerHeader.svelte` | 151–338 | Update styles for simplified layout |
| `ComposerHeaderBar.svelte` | 23–73 | Minor: already clean, verify no changes needed |

### Session 3: Dedicated X Preview

| File | Lines | Change |
|------|-------|--------|
| `PreviewOverlay.svelte` | NEW | Full-screen preview overlay component |
| `ComposeWorkspace.svelte` | 63 | `previewCollapsed` → `previewMode` (inverted) |
| `ComposeWorkspace.svelte` | 142–144 | `togglePreview()` sets `previewMode = !previewMode` |
| `ComposeWorkspace.svelte` | 473–487 | Remove inline `.preview-section` and `ThreadPreviewRail` |
| `ComposeWorkspace.svelte` | 12–28 | Remove `ThreadPreviewRail` import, add `PreviewOverlay` import |
| `ComposeWorkspace.svelte` | ~535 | Add `{#if previewMode}<PreviewOverlay .../>` before CommandPalette |
| `HomeComposerHeader.svelte` | 107–116 | Preview button opens overlay (same toggle, new semantics) |
| `ComposerHeaderBar.svelte` | 34–48 | Preview button opens overlay (same toggle, new semantics) |
| `CommandPalette.svelte` | 49 | Update preview action description |

### Session 4: Shortcuts and Safety

| File | Lines | Change |
|------|-------|--------|
| `ComposeWorkspace.svelte` | 331–354 | Add undo snapshot before inline assist API call |
| `ComposeWorkspace.svelte` | 72–76 | Extend undo mechanism to cover inline assist |
| `ComposeWorkspace.svelte` | 489–494 | Show undo banner after inline assist (reuse existing banner) |
| `ThreadFlowLane.svelte` | 321–346 | Add undo snapshot for per-block inline assist |
| `shortcuts.ts` | 103–122 | Update `SHORTCUT_CATALOG` descriptions if any shortcuts change |
| `CommandPalette.svelte` | 43–60 | Verify action descriptions match updated shortcut policy |

### Session 5: Validation

| File | Change |
|------|--------|
| All modified files | svelte-check pass |
| All modified files | Accessibility audit (aria-labels, focus management) |
| All modified files | Mobile responsiveness check (≤640px) |
| All modified files | Reduced-motion check |
| Autosave/recovery | Manual integration test |

## CSS Token Usage

The redesign uses existing design tokens from `dashboard/src/app.css`. No new tokens are introduced.

| Token | Current Usage | Redesign Usage |
|-------|---------------|----------------|
| `--color-surface` | Editor bg, dot bg | Unchanged |
| `--color-base` | Textarea bg (TweetEditor) | Removed — textarea goes transparent |
| `--color-border` | Textarea border | Removed from textarea |
| `--color-border-subtle` | Spine, dots, separators | Lighter application (thinner, more transparent) |
| `--color-text` | Content text | Unchanged |
| `--color-text-subtle` | Counters, hints | Less frequently shown (conditional counters) |
| `--color-text-muted` | Icon buttons | Unchanged |
| `--color-accent` | Focus states, active toggles | Unchanged |
| `--color-danger` | Over-limit states | Unchanged |
| `--color-warning` | Near-limit states | Unchanged |
| `--font-sans` | Editor font | Unchanged |
| `--font-mono` | Counters, handle | Unchanged |

## Contracts Preserved (Evidence)

| Contract | Evidence |
|----------|----------|
| `ThreadBlock[]` | No changes to `$lib/api` types. `ThreadFlowLane` still emits `ThreadBlock[]` via `onchange`. |
| `ComposeRequest` | `buildComposeRequest()` in `$lib/utils/composeHandlers` is not modified. |
| `onsubmit` callback | `ComposeWorkspace` still calls `onsubmit(data)` in `handleSubmit()`. |
| Autosave format | Same `AUTOSAVE_KEY`, same `{ mode, tweetText, blocks, timestamp }` payload. |
| Modal entry | `ComposeModal.svelte` prop interface unchanged — it just renders `ComposeWorkspace`. |
| Home entry | `+page.svelte` passes same props to `ComposeWorkspace`. |
| API calls | `api.content.compose()`, `api.assist.improve()`, `api.assist.thread()`, `api.media.upload()` — all call sites unchanged. |
