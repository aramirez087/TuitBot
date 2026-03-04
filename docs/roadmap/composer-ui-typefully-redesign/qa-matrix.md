# QA Matrix — Composer Redesign

Validated against source code from Sessions 2–5. Each cell cites the file and line(s) that implement the behavior.

## 1. Feature Flows

### 1.1 Compose Tweet (Tweet Mode)

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Type in textarea | Text updates `tweetText` state | `TweetEditor.svelte:114` — `oninput` calls `onchange` | Pass |
| Character counter hidden ≤240 | Counter not rendered | `TweetEditor.svelte:118` — `{#if tweetChars > 240 \|\| tweetOverLimit}` | Pass |
| Character counter visible >240 | Shows `N/280` | `TweetEditor.svelte:119-127` | Pass |
| Over-limit styling | Red counter + left border glow | `TweetEditor.svelte:196-198,208-211` | Pass |
| Submit via `Cmd+Enter` | Calls `handleSubmit()` | `ComposeWorkspace.svelte:301-302` — tweet mode branch | Pass |
| Submit via publish pill (embedded) | Calls `handleSubmit()` | `HomeComposerHeader.svelte:67` → `onsubmit` | Pass |
| Submit via pill (modal) | Calls `handleSubmit()` | `ComposerCanvas.svelte:41` → `onsubmit` | Pass |
| Submit builds correct payload | `buildComposeRequest()` called | `ComposeWorkspace.svelte:246-248` | Pass |
| Post-submit reset (embedded) | All state cleared | `ComposeWorkspace.svelte:253-268` | Pass |
| Post-submit unmount (modal) | Component unmounts on close | `ComposeModal.svelte:35-44` — conditional render | Pass |

### 1.2 Compose Thread (Thread Mode)

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Switch to thread mode | `mode = 'thread'` | `ComposeWorkspace.svelte:308` — `Cmd+Shift+T` | Pass |
| Default blocks created | 2 empty blocks | `ThreadFlowLane.svelte:18` — `threadOps.createDefaultBlocks()` | Pass |
| Type in thread card | Block text updates | `ThreadFlowCard.svelte:79-83` — `handleInput` → `ontext` | Pass |
| Auto-resize textarea | Height matches content | `ThreadFlowCard.svelte:74-77` — `autoResize()` | Pass |
| Card char counter hidden ≤240 | Counter not shown | `ThreadFlowCard.svelte:133` — `{#if charCount > 240 \|\| overLimit}` | Pass |
| Validation: ≥2 non-empty blocks | `canSubmit` derived | `ThreadFlowLane.svelte:45-49` | Pass |
| Submit via `Cmd+Shift+Enter` | Calls `handleSubmit()` | `ComposeWorkspace.svelte:300` | Pass |
| Screen reader: mode switch announced | `aria-live` region | `ComposeWorkspace.svelte:159-162,466` | Pass |

### 1.3 Thread Split / Merge / Reorder

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| `Cmd+Enter` in card | Split at cursor or add after | `ThreadFlowLane.svelte:237-249` — `e.stopPropagation()` | Pass |
| `Cmd+Shift+S` | Split block at cursor | `ThreadFlowLane.svelte:299-303` | Pass |
| `Cmd+Shift+M` | Merge with next block | `ThreadFlowLane.svelte:306-310` | Pass |
| `Backspace` at pos 0 | Merge with previous | `ThreadFlowLane.svelte:252-263` | Pass |
| `Alt+Arrow` reorder | Move block up/down | `ThreadFlowLane.svelte:276-289` | Pass |
| Drag-and-drop reorder | Block moves to target | `ThreadFlowLane.svelte:193-232` | Pass |
| `Cmd+D` duplicate | Block duplicated | `ThreadFlowLane.svelte:292-296` | Pass |
| `Tab` / `Shift+Tab` | Navigate blocks | `ThreadFlowLane.svelte:266-273` | Pass |
| Merge media overflow error | Error shown for 3s | `ThreadFlowLane.svelte:147-151` | Pass |
| Reorder announcement | Screen reader status | `ThreadFlowLane.svelte:119-122,377` | Pass |

### 1.4 Media Attach

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Click attach icon | File dialog opens | `TweetEditor.svelte:155` → `fileInput?.click()` | Pass |
| Upload calls API | `api.media.upload()` | `TweetEditor.svelte:77` | Pass |
| Preview thumbnail shown | Grid of 80x80 thumbnails | `TweetEditor.svelte:130-148` | Pass |
| Remove media button | Removes from array + revokes URL | `TweetEditor.svelte:97-101` | Pass |
| Max 4 images enforced | `canAttachMore` derived | `TweetEditor.svelte:43` | Pass |
| GIF/video exclusivity | Error if mixing types | `TweetEditor.svelte:62-68` | Pass |
| Compact icon (no hint text) | Icon-only button, 32px | `TweetEditor.svelte:153-161,279-284` | Pass |

### 1.5 Schedule

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Schedule config loaded | `api.content.schedule()` on mount | `+page.svelte:13` | Pass |
| Time picker in inspector | `TimePicker` component | `InspectorContent.svelte:48-51` | Pass |
| Selected time flows to submit | `selectedTime` in `buildComposeRequest` | `ComposeWorkspace.svelte:247` | Pass |
| Publish pill label changes | "Schedule" when time selected | `HomeComposerHeader.svelte:48-50` | Pass |

### 1.6 AI Assist (`Cmd+J`)

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Snapshot before API call (tweet) | `undoSnapshot` set | `ComposeWorkspace.svelte:349` | Pass |
| Snapshot before API call (thread) | `undoSnapshot` set | `ComposeWorkspace.svelte:369` | Pass |
| Selection-only replacement | `tweetText.slice(0,start) + result + slice(end)` | `ComposeWorkspace.svelte:354-355` | Pass |
| Full replacement (no selection) | `tweetText = result.content` | `ComposeWorkspace.svelte:357` | Pass |
| Undo banner shown for 10s | `showUndo = true` + setTimeout | `ComposeWorkspace.svelte:360-362` | Pass |
| Undo restores snapshot | `handleUndo()` restores state | `ComposeWorkspace.svelte:404-411` | Pass |
| Snapshot cleared on failure | `undoSnapshot = null` | `ComposeWorkspace.svelte:365` | Pass |
| Thread mode delegates to lane | `threadFlowRef?.handleInlineAssist()` | `ComposeWorkspace.svelte:371` | Pass |
| Voice cue forwarded | `voiceCue || undefined` passed | `ComposeWorkspace.svelte:353,371` | Pass |

### 1.7 Generate from Notes

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Snapshot before generation | `undoSnapshot` set | `ComposeWorkspace.svelte:383` | Pass |
| Tweet mode: `api.assist.improve()` | Notes expanded to tweet | `ComposeWorkspace.svelte:391-395` | Pass |
| Thread mode: `api.assist.thread()` | Notes split to blocks | `ComposeWorkspace.svelte:386-389` | Pass |
| Undo banner shown | Same 10s mechanism | `ComposeWorkspace.svelte:399-401` | Pass |
| Notes panel closes | `showFromNotes = false` | `ComposeWorkspace.svelte:398` | Pass |

### 1.8 Full-Screen Preview

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| `Cmd+Shift+P` opens overlay | `togglePreview()` | `ComposeWorkspace.svelte:310` | Pass |
| Preview button opens overlay | `ontogglepreview` callback | `HomeComposerHeader.svelte:84`, `ComposerHeaderBar.svelte:38` | Pass |
| Overlay is full-screen | `position: fixed; inset: 0; z-index: 2000` | `ComposerPreviewSurface.svelte:109-112` | Pass |
| Same state, no duplication | Props from workspace `$state` | `ComposeWorkspace.svelte:613-621` | Pass |
| Escape closes | `previewMode = false` | `ComposeWorkspace.svelte:289` | Pass |
| Close button closes | `onclose` callback | `ComposerPreviewSurface.svelte:69` | Pass |
| Focus trap active | `use:focusTrap` | `ComposerPreviewSurface.svelte:59` | Pass |
| Focus restored on close | `triggerElement.focus()` in `onMount` return | `ComposerPreviewSurface.svelte:42-45` | Pass |
| Tweet preview shows content | `TweetPreview` rendered | `ComposerPreviewSurface.svelte:78-85` | Pass |
| Thread preview shows non-empty blocks | Filtered + mapped | `ComposerPreviewSurface.svelte:29-31,90-99` | Pass |
| Thread connectors via index/total | `index={i} total={visibleBlocks.length}` | `ComposerPreviewSurface.svelte:96-97` | Pass |
| Shortcuts blocked while open | Early return in keydown | `ComposeWorkspace.svelte:288-292` | Pass |

### 1.9 Autosave / Recovery

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Debounced autosave on state change | 500ms after `mode`/`tweetText`/`threadBlocks` | `ComposeWorkspace.svelte:155,203-208` | Pass |
| Payload format preserved | `{ mode, tweetText, blocks, timestamp }` | `ComposeWorkspace.svelte:206` | Pass |
| Recovery check on mount | `checkRecovery()` | `ComposeWorkspace.svelte:166` | Pass |
| TTL enforced (7 days) | `AUTOSAVE_TTL_MS` check | `ComposeWorkspace.svelte:221` | Pass |
| Recovery banner shows | `showRecovery = true` | `ComposeWorkspace.svelte:226` | Pass |
| Recover restores state | `recoverDraft()` | `ComposeWorkspace.svelte:230-236` | Pass |
| Dismiss clears storage | `dismissRecovery()` → `clearAutoSave()` | `ComposeWorkspace.svelte:239` | Pass |
| Post-submit clears autosave | `clearAutoSave()` before `onsubmit` | `ComposeWorkspace.svelte:249` | Pass |
| Preview mode not persisted | `previewMode` starts `false` | `ComposeWorkspace.svelte:63,180` | Pass |

## 2. Surface Matrix

| Flow | Embedded (Home) | Modal |
|------|----------------|-------|
| Header | `HomeComposerHeader` — handle, post count, publish pill, icon tools | `ComposerHeaderBar` — close, preview, inspector, focus buttons |
| Editor | `TweetEditor` / `ThreadFlowLane` in `ComposerCanvas` | Same |
| Inspector | Desktop sidebar + mobile drawer | Desktop sidebar only |
| Preview overlay | `ComposerPreviewSurface` at `z-index: 2000` | Same — renders above modal `z-index: 1000` |
| Command palette | Same in both | Same |
| Submit (tweet) | `Cmd+Enter` or publish pill in header | `Cmd+Enter` or submit pill in canvas |
| Submit (thread) | `Cmd+Shift+Enter` or publish pill | `Cmd+Shift+Enter` or submit pill |
| Close/Escape | Dismisses layers only (notes > inspector > focus) | Dismisses layers, then closes modal |
| Post-submit | State resets in-place (component stays) | Component unmounts on close |
| Autosave | Active | Active |
| Recovery | Banner on mount | Banner on mount |

## 3. Desktop Keyboard Paths

Full shortcut coverage per `shortcut-regression-matrix.md`. Summary:

| Shortcut | Tweet Embedded | Tweet Modal | Thread Embedded | Thread Modal | Preview | Palette |
|----------|---------------|-------------|-----------------|--------------|---------|---------|
| `Cmd+Enter` | Submit | Submit | Split/add (card) | Split/add (card) | Blocked | N/A |
| `Cmd+Shift+Enter` | Submit | Submit | Submit thread | Submit thread | Blocked | N/A |
| `Cmd+K` | Palette | Palette | Palette | Palette | Blocked | N/A |
| `Cmd+J` | AI+undo | AI+undo | AI+undo (block) | AI+undo (block) | Blocked | N/A |
| `Cmd+Shift+F` | Blocked | Focus toggle | Blocked | Focus toggle | Blocked | N/A |
| `Cmd+Shift+N` | → tweet | → tweet | → tweet | → tweet | Blocked | N/A |
| `Cmd+Shift+T` | → thread | → thread | → thread | → thread | Blocked | N/A |
| `Cmd+I` | Inspector | Inspector | Inspector | Inspector | Blocked | N/A |
| `Cmd+Shift+P` | Preview | Preview | Preview | Preview | Close | N/A |
| `Escape` | Layers | Layers/close | Layers | Layers/close | Close | Close |
| `Tab` (thread) | — | — | Next block | Next block | — | — |
| `Alt+Arrow` (thread) | — | — | Reorder | Reorder | — | — |
| `Cmd+D` (thread) | — | — | Duplicate | Duplicate | — | — |
| `Cmd+Shift+S` (thread) | — | — | Split | Split | — | — |
| `Cmd+Shift+M` (thread) | — | — | Merge | Merge | — | — |

## 4. Mobile / Narrow-Width (≤640px)

| Component | Behavior at ≤640px | Touch targets (pointer: coarse) | Status |
|-----------|-------------------|--------------------------------|--------|
| `TweetEditor` | Font 16px (iOS zoom prevention) | Remove-media 32px, attach icon 44px | Pass |
| `ThreadFlowCard` | Font 16px, spine dot hidden, border-left replaces dot, between-plus margin reset | Handle/action 44px, between-zone 44px | Pass |
| `ThreadFlowCard` (hover: none) | Sep tools always visible, between-plus always visible | — | Pass |
| `ThreadFlowLane` | `padding-left: 0`, lane spine hidden | — | Pass |
| `ComposerPreviewSurface` | Full-screen (no radius, no margin, 100% height) | Close button 44px | Pass |
| `HomeComposerHeader` | Handle hidden, separator hidden, icon-tools hidden, pill smaller | Icon-btn 44px, cta-pill 44px | Pass |
| `ComposerHeaderBar` | — (uses modal which goes full-screen) | Header-btn 44px | Pass |
| `ComposerCanvas` | Padding reduced, submit pill full-width, safe-area padding | Submit-pill 44px | Pass |
| `ComposerCanvas` (768px) | Inspector sidebar hidden | — | Pass |
| `ComposerShell` | Modal goes full-screen, no border-radius | — | Pass |
| `RecoveryBanner` | — | Buttons 44px | Pass |
| `InspectorContent` | — | AI buttons 44px | Pass |

**Known gap (pre-existing, not a regression):** At ≤640px, `HomeComposerHeader` hides `.icon-tools` (preview, AI, inspector, palette buttons). These features remain accessible via keyboard shortcuts or the mobile inspector drawer. A future session could add a hamburger menu.

## 5. State Restoration

| Scenario | Expected | Evidence | Status |
|----------|----------|----------|--------|
| Preview → Edit round-trip | Editor state unchanged (same `$state` refs) | `ComposerPreviewSurface` reads props, no two-way binding | Pass |
| Autosave → Recovery | Draft restored with correct mode, text, blocks | `ComposeWorkspace.svelte:230-236` | Pass |
| Autosave TTL expired | Draft discarded, storage cleared | `ComposeWorkspace.svelte:221-223` | Pass |
| Undo after AI improve | Previous content restored | `ComposeWorkspace.svelte:404-411` | Pass |
| Mode switch preserves both buffers | Tweet text + thread blocks both maintained | Both are independent `$state` vars | Pass |
| Inspector state persists across sessions | `localStorage` for `tuitbot:inspector:open` | `ComposeWorkspace.svelte:124-135` | Pass |

## 6. Accessibility

| Component | ARIA Pattern | Evidence | Status |
|-----------|-------------|----------|--------|
| `TweetEditor` textarea | `aria-label="Tweet content"` | Line 116 | Pass |
| `TweetEditor` char counter | `aria-live="polite"` + `aria-label="Character count"` | Lines 122-123 | Pass |
| `TweetEditor` attach button | `aria-label` varies by state | Line 158 | Pass |
| `TweetEditor` remove media | `aria-label="Remove media"` | Line 143 | Pass |
| `ThreadFlowCard` textarea | `aria-label="Post N of M"` | Line 127 | Pass |
| `ThreadFlowCard` drag handle | `aria-label` with reorder instructions | Line 145 | Pass |
| `ThreadFlowCard` merge button | `aria-label="Merge post N with post N+1"` | Line 156 | Pass |
| `ThreadFlowCard` remove button | `aria-label="Remove post N"` | Line 164 | Pass |
| `ThreadFlowCard` add-after | `aria-label="Add post after post N"` | Line 175 | Pass |
| `ThreadFlowLane` container | `role="list"` + `aria-label="Thread editor"` | Line 375 | Pass |
| `ThreadFlowLane` announcement | `role="status"` + `aria-live="polite"` | Line 377 | Pass |
| `ComposerPreviewSurface` dialog | `role="dialog"` + `aria-modal="true"` + `aria-label` | Lines 56-58 | Pass |
| `ComposerPreviewSurface` close | `aria-label="Close preview"` | Line 70 | Pass |
| `ComposerPreviewSurface` focus | `use:focusTrap` + focus restore on close | Lines 36-46,59 | Pass |
| `HomeComposerHeader` publish | `aria-label` varies by state | Line 70 | Pass |
| `HomeComposerHeader` preview | `aria-label` varies by state | Line 85 | Pass |
| `HomeComposerHeader` AI | `aria-label="AI improve selection or post"` | Line 99 | Pass |
| `HomeComposerHeader` inspector | `aria-label` varies by state | Line 110 | Pass |
| `HomeComposerHeader` palette | `aria-label="Command palette"` | Line 119 | Pass |
| `HomeComposerHeader` status dot | `aria-label` varies by content state | Line 61 | Pass |
| `ComposeWorkspace` status | `role="status"` + `aria-live="polite"` | Line 466 | Pass |
| `ComposerShell` dialog | `role="dialog"` + `aria-modal="true"` + `aria-label` + `use:focusTrap` | Lines 33-36 | Pass |
| `ComposerHeaderBar` close | `aria-label="Close composer"` | Line 27 | Pass |
| `ComposerHeaderBar` preview | `aria-label` varies by state | Line 39 | Pass |
| `ComposerHeaderBar` inspector | `aria-label` varies by state | Line 55 | Pass |
| `ComposerHeaderBar` focus | `aria-label` varies by state | Line 65 | Pass |
| `RecoveryBanner` | `role="alert"` | `RecoveryBanner.svelte:11` | Pass |
| `ComposerInspector` drawer | `role="complementary"` + `aria-label` | `ComposerInspector.svelte:39` | Pass |
| `ThreadFlowLane` merge error | `role="alert"` | Line 407 | Pass |
| `ThreadFlowLane` validation | `role="status"` + `aria-live="polite"` | Line 410 | Pass |

## 7. Reduced Motion

| Component | Has transitions | `prefers-reduced-motion: reduce` | Status |
|-----------|----------------|----------------------------------|--------|
| `ComposeWorkspace` | `.undo-btn` | Yes (line 674) | Pass |
| `ComposerPreviewSurface` | `.preview-close` | Yes (line 205) | Pass |
| `ThreadFlowCard` | 7 properties (card, area, dot, tools, plus, handle, action) | Yes (line 399) — comprehensive | Pass |
| `HomeComposerHeader` | `.spin-icon`, `.header-dot` | Yes (line 260) | Pass |
| `ComposerShell` | `.modal` width | Yes (line 90) | Pass |
| `ComposerInspector` | `.inspector-backdrop`, `.inspector-drawer` animations | Yes (line 109) | Pass |
| `ComposerHeaderBar` | `.header-btn` | Yes (added Session 5) | Pass |
| `TweetEditor` | `.remove-media-btn`, `.attach-icon-btn` | Yes (added Session 5) | Pass |
| `ComposerCanvas` | `.submit-pill` | Yes (added Session 5) | Pass |
| `VoiceContextPanel` | `.voice-toggle`, `.cue-input`, `.saved-cue-item` | Yes (added Session 5) | Pass |
| `InspectorContent` | `.ai-action-btn` | Yes (added Session 5) | Pass |
