# Home Surface Plan

Acceptance criteria, layout specification, and implementation details for the composer-first home experience.

## Acceptance Criteria

### AC1: Full-page dark canvas with the writing lane as the first visible surface

**Requirement**: When the app loads at `/` with the default preference, the user sees a full-page writing canvas — not an analytics dashboard, not a modal, not a splash screen.

**Testable assertions**:
- Fresh install: navigating to `/` renders `HomeComposerSurface` with a blinking cursor in the first textarea
- The canvas background uses `--color-base` (dark theme) with no card-like containers around the writing area
- The writing lane occupies the main content area (right of sidebar), not a floating modal
- The sidebar remains visible and functional while the compose surface is active
- Time from route load to visible cursor is under 200ms (no intermediate loading states unless preference fetch is slow)

### AC2: Centered thread lane with a left spine, avatar pucks, and low-noise separators

**Requirement**: The writing area is a centered column with a vertical "spine" connecting small avatar circles that anchor each tweet segment.

**Testable assertions**:
- The writing lane has `max-width: 860px` and is horizontally centered within the main content area
- Each tweet segment displays a 16–20px avatar circle on the left, aligned to the top of the segment's writing area
- A 1px vertical line in `--color-border-subtle` connects avatar circles from top to bottom of the thread
- Between-segment separators show character count and hover-reveal tools (merge, remove, drag handle)
- In single-tweet mode, the avatar puck appears but no separator is visible
- The spine replaces the existing `border-left: 2px solid` accent bar (not additive)
- On mobile (< 640px), avatar pucks shrink or hide; the accent bar returns for simplicity

### AC3: Top-right Schedule and Publish pill actions with secondary icon tools

**Requirement**: The primary submit actions are a cluster of pill-shaped buttons in the top-right corner, replacing the floating submit pill from the modal.

**Testable assertions**:
- `Schedule` pill is warm-toned (orange/amber), rendered on the right side of the header
- `Publish` pill is cool-toned (`--color-accent`), adjacent to Schedule
- When a time is selected, `Publish` changes label to `Schedule` and the Schedule pill shows the selected time
- Icon tools (preview, inspector, AI, command palette) are 28px quiet buttons right-aligned after the pills
- The floating submit pill from `ComposerCanvas` is hidden when in full-page (`embedded`) mode
- `Cmd+Enter` submits (same behavior as modal context)
- All buttons meet 44px touch targets on mobile (`@media (pointer: coarse)`)

### AC4: Inline prompt module and dismissible getting-started tips that appear only when useful

**Requirement**: New users see contextual help that disappears after use; experienced users see a clean surface.

**Testable assertions**:
- On first use (no prior autosave, no preference set), a dismissible "getting started" card appears below the first textarea with 2–3 quick tips (keyboard shortcuts, thread splitting, AI assist)
- The card has a "Got it" dismiss button; dismissal persists via `persistSet('home_tips_dismissed', true)`
- After dismissal, the card never appears again
- When the writing area is empty, a subtle placeholder prompt appears (e.g., "What's on your mind?" or "Start typing...") — same as current `TweetEditor` placeholder
- When the user has content, no tips or prompts overlay the writing area
- The "From Notes" panel in the inspector serves as the primary AI prompt surface (no new prompt module outside the inspector)

### AC5: Analytics available as an alternate home surface, not the default

**Requirement**: The analytics dashboard is accessible but not the landing page for new installs.

**Testable assertions**:
- When `home_surface` preference is `'analytics'`, navigating to `/` renders `AnalyticsDashboard` with stat cards, chart, topics, and performance — identical to the current home route output
- `AnalyticsDashboard` is a component extraction of the current `+page.svelte` content (no redesign)
- The analytics dashboard still loads data via `loadDashboard()` and `startAutoRefresh()`, with cleanup on unmount
- Switching between surfaces does not cause data loss (compose autosave persists; analytics state is independent)
- The analytics surface is also accessible via a direct route or sidebar if needed (exact mechanism TBD in Session 9)

### AC6: A persisted UI preference named `home_surface` with `composer` as the fresh-install default

**Requirement**: The home surface choice is persisted across sessions and controllable from Settings.

**Testable assertions**:
- `persistGet('home_surface', 'composer')` returns `'composer'` on fresh install (no prior state)
- `persistSet('home_surface', 'analytics')` causes subsequent loads of `/` to render the analytics dashboard
- The Settings page has a dropdown or radio group under a visible section (e.g., "Appearance") with two options: "Composer" (default) and "Analytics Dashboard"
- Changing the setting takes effect on next navigation to `/` (no full reload needed)
- The preference is stored in Tauri's `ui-state.json` via `@tauri-apps/plugin-store`; in browser-only mode (no Tauri), falls back to in-memory default (`'composer'`)
- No backend API is called to read or write this preference

## Layout Specification

### Desktop (> 1120px)

```
┌──────────────────────────────────────────────────────────┐
│  Sidebar (220px)  │           Main Content Area          │
│                   │ ┌──────────────────────────────────┐ │
│  [Home]           │ │  HomeComposerHeader              │ │
│  [Activity]       │ │  [                   ] [Schedule]│ │
│  [Approval]       │ │  [                   ] [Publish] │ │
│  [Content]        │ │  [   icon tools...   ]           │ │
│  [Drafts]         │ ├──────────────────────────────────┤ │
│  [Discovery]      │ │                                  │ │
│  [Targets]        │ │   ┌───┐                          │ │
│  [Strategy]       │ │   │ o │  Start your thread...    │ │
│  [Costs]          │ │   │ | │                          │ │
│  [Observability]  │ │   │ o │  Tweet 2...              │ │
│  [Settings]       │ │   │ | │                          │ │
│                   │ │   │ o │  Tweet 3...        [Insp]│ │
│                   │ │   └───┘                   [ector]│ │
│                   │ │         (860px centered)   [Rail]│ │
│                   │ │                            (260) │ │
│                   │ └──────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

**Dimensions**:
- Writing lane: `max-width: 860px`, centered via `margin: 0 auto`
- Inspector rail: `width: 260px`, collapsible, right-aligned
- When inspector is open, writing lane shrinks from its max to fit (flex layout)
- Padding: 24px horizontal on main content (inherits from `+layout.svelte`)
- Avatar spine: 32px wide (16px avatar + 8px margin each side)

### Tablet (769–1120px)

- Writing lane: fills available width minus padding
- Inspector: collapsible overlay (not inline rail)
- Avatar spine: same as desktop

### Mobile (641–768px)

- Writing lane: full width minus 16px horizontal padding
- Inspector: bottom drawer overlay (same as current `ComposerInspector` mobile behavior)
- Avatar spine: reduced to 12px avatars or hidden

### Small mobile (≤ 640px)

- Writing lane: full viewport width minus 12px padding
- Textarea font: 16px (prevents iOS zoom)
- Inspector: full-width bottom sheet
- Avatar spine: hidden; accent bar returns
- Action cluster: Schedule and Publish stack vertically or collapse to single "Post" button

## Component Map

### Reused Components (no changes)

| Component | Role |
|-----------|------|
| `ComposerCanvas.svelte` | Writing area + inspector flex layout |
| `TweetEditor.svelte` | Single-tweet editing |
| `ThreadComposer.svelte` | Thread orchestrator |
| `ThreadPreviewRail.svelte` | Collapsible preview |
| `TimePicker.svelte` | Schedule time selection |
| `VoiceContextPanel.svelte` | Voice/tone cue |
| `FromNotesPanel.svelte` | Notes-to-content AI |
| `CommandPalette.svelte` | Keyboard command palette |
| `ComposerInspector.svelte` | Mobile inspector drawer |

### Modified Components

| Component | Change |
|-----------|--------|
| `ThreadFlowLane.svelte` | Add `avatarSpine` prop; render avatar circles and connecting line |
| `ThreadFlowCard.svelte` | Add `avatarPuck` prop; replace left accent bar with avatar circle when spine is active |
| `ComposerCanvas.svelte` | Add `embedded` prop to conditionally hide floating submit pill |

### New Components

| Component | Role | Estimated Lines |
|-----------|------|-----------------|
| `ComposeWorkspace.svelte` | Shared compose orchestrator (state, autosave, AI, submit) | ~400 |
| `HomeComposerSurface.svelte` | Full-page compose wrapper; uses ComposeWorkspace without modal shell | ~120 |
| `HomeComposerHeader.svelte` | Action cluster: Schedule pill, Publish pill, icon tools | ~150 |
| `AnalyticsDashboard.svelte` | Extract of current +page.svelte analytics content | ~100 |

### Extracted Component

| Component | Source |
|-----------|--------|
| `AnalyticsDashboard.svelte` | Extracted from `(app)/+page.svelte` lines 1–193 (entire current file content) |

## Interaction Specification

### Keyboard Shortcuts (full-page composer context)

| Shortcut | Action | Notes |
|----------|--------|-------|
| `Cmd+Enter` | Submit/schedule | Same as modal |
| `Cmd+K` | Open command palette | Same as modal |
| `Cmd+J` | AI improve (selection-aware) | Same as modal |
| `Cmd+Shift+N` | Switch to tweet mode | Same as modal |
| `Cmd+Shift+T` | Switch to thread mode | Same as modal |
| `Cmd+I` | Toggle inspector rail | Same as modal |
| `Cmd+Shift+P` | Toggle preview | Same as modal |
| `Cmd+Shift+F` | No action (no focus mode in full-page) | Absorbed silently |
| `Escape` | Close from-notes panel → close mobile inspector → no further action | Does NOT navigate away |
| `Cmd+N` | No action on home route (already in compose) | Modal compose disabled when on `/` with composer surface |

### CTA Hierarchy

1. **Primary**: `Publish` pill — cool-toned, prominent, right-side of header
2. **Secondary**: `Schedule` pill — warm-toned, adjacent to Publish, appears when time is selected or always visible
3. **Tertiary**: Icon tools — quiet 28px buttons for preview, inspector, AI, palette
4. **Quaternary**: Inspector actions — AI Improve, From Notes, voice cue (inside collapsible rail)

## Preference System

### Storage

```typescript
// Key
const HOME_SURFACE_KEY = 'home_surface';

// Type
type HomeSurface = 'composer' | 'analytics';

// Read (async, defaults to 'composer')
const surface = await persistGet<HomeSurface>(HOME_SURFACE_KEY, 'composer');

// Write (async)
await persistSet(HOME_SURFACE_KEY, 'analytics');
```

### Settings UI

A new "Appearance" section in the Settings page (or placed in the "Business Profile" section if a dedicated section feels excessive):

```svelte
<div class="field-row">
  <label for="home-surface">Home Surface</label>
  <select id="home-surface" bind:value={$draft.home_surface}>
    <option value="composer">Composer (write-first)</option>
    <option value="analytics">Analytics Dashboard</option>
  </select>
  <p class="field-hint">Choose what you see when you open Tuitbot</p>
</div>
```

The preference is saved via `persistSet` (not the server settings API) since it is UI-only and not part of the automation configuration.

### Sync Behavior

- Preference is device-local (Tauri plugin-store per installation)
- No cross-device sync (local-first tool — each device has its own preference)
- In browser-only mode (no Tauri), defaults to `'composer'` and cannot be persisted across sessions

## Progressive Disclosure

### Getting-Started Tips

**When shown**: First visit to the home composer when no autosaved draft exists and `home_tips_dismissed` is not set.

**Content** (3 tips):
1. "Press `Cmd+Enter` to split into a thread" + visual hint
2. "Open the command palette with `Cmd+K` for all actions"
3. "AI can improve your writing — select text and press `Cmd+J`"

**Dismissal**: "Got it" button sets `persistSet('home_tips_dismissed', true)`. Card fades out with 200ms transition.

**Layout**: Appears as a subtle card below the first textarea, inside the writing lane, with `--color-surface` background and `--color-border-subtle` border. Max-width matches writing lane.

### Prompt Cards

No separate prompt card system. The "From Notes" panel in the inspector serves this role. The empty-state placeholder text in the textarea ("What's on your mind?") provides the initial prompt.

### Inspector Panels

Same as modal context: Schedule, Voice, AI sections stacked vertically. No changes to inspector content for the home surface — only the header and wrapper change.
