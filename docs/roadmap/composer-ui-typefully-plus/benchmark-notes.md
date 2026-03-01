# Typefully Benchmark Analysis

Competitive audit of Typefully's compose experience against Tuitbot's current composer, identifying patterns to emulate, exceed, and skip.

## Typefully's Core UX Model

Typefully treats composing as **writing**, not as **form-filling**. The entire interface is a continuous document editor where tweet boundaries are visual separators (a horizontal line + character count badge), not separate bordered input fields. The result feels like drafting in a note-taking app that happens to produce threads.

Key pillars of their model:

1. **Unibody editor** — One continuous writing surface. No card borders, no per-tweet gutters, no stacked textareas. The separation between tweets exists only as a thin horizontal divider.
2. **Content determines mode** — There is no "tweet vs. thread" toggle the user must choose before writing. A single block of text is a tweet. Adding a separator (double-newline or shortcut) turns it into a thread. Removing all separators collapses it back to a tweet.
3. **Draft-first philosophy** — The compose surface looks like a note app, not a social media dashboard. No prominent headers, minimal chrome, generous whitespace.
4. **Inline preview toggle** — Instead of a permanent side-by-side preview pane, Typefully offers a toggle between "writing mode" and "preview mode" in the same canvas area. Preview replaces the editor rather than sitting next to it.
5. **Keyboard-native flow** — Thread operations (add separator, navigate between tweets, reorder) are primarily keyboard-driven. Mouse affordances exist but feel secondary.

## Feature-by-Feature Comparison

| Feature | Typefully | Tuitbot (current) | Gap |
|---------|-----------|-------------------|-----|
| **Editor model** | Unibody: one continuous document with visual separators | Card-per-tweet: separate bordered textareas with gutters, numbers, drag handles | High — our card model is the single biggest friction source |
| **Mode selection** | Implicit: content determines if it's a tweet or thread | Explicit: user picks "Tweet" or "Thread" tab before writing | High — forces premature decision |
| **Preview** | Inline toggle (edit/preview in same pane) | Side-by-side (50/50 grid, always visible) | Medium — steals half the width for a preview that isn't always useful |
| **Focus mode** | Full-screen, zero-chrome writing canvas | Full-screen but retains header, tabs, footer | Medium — our focus mode isn't truly focused |
| **Thread separator** | Thin line with char count badge; click between tweets to add a new one | Explicit "Add tweet" button at bottom | Medium — less fluid for power users |
| **Auto-split** | Paste long text → auto-splits at word boundaries | Not supported; must manually create cards | Medium — friction for drafts written outside the app |
| **Keyboard shortcuts** | Cmd+Enter for separator, arrow keys for navigation | Good coverage (Cmd+K palette, Alt+arrows for reorder) but card-based | Low — shortcuts exist, just need model update |
| **Drag reorder** | Drag separator handles to reorder tweets | Drag card handles to reorder | Low — functionally equivalent |
| **AI integration** | Basic: suggest, rewrite | Deeper: voice context, inline improve, from-notes generation, per-selection improvement | Tuitbot is ahead |
| **Media attachment** | Per-tweet inline | Per-card inline (MediaSlot component) | Comparable |
| **Scheduling** | Sidebar calendar + time picker | TimePicker in modal body (below editor) | Low — ours is fine, just positioned poorly |
| **Auto-save** | Automatic draft persistence | Automatic with recovery banner | Comparable |
| **Voice/tone** | No built-in voice context | VoiceContextPanel with saved cues, brand voice integration | Tuitbot is ahead |
| **Command palette** | Limited shortcuts | Full Cmd+K palette with thread actions | Tuitbot is ahead |
| **Chrome density** | Minimal: no visible header title, no tab bar, no footer | Dense: "Compose" header, date subtitle, mode tabs, 5-element footer | High — our composer feels like a dashboard form |

## Patterns to Emulate

### 1. Unibody Editor (Priority: Critical)

**What Typefully does**: A single continuous writing surface where tweet breaks are visual separators (a thin horizontal line with a character count badge), not separate form controls. Each "tweet" is a region of text between separators, not an independent component with its own textarea, border, gutter, and action bar.

**Why it matters**: The card-per-tweet model is Tuitbot's single biggest UX friction point. A 5-tweet thread currently shows 5 separate bordered boxes, each with its own textarea, character counter, drag handle, gutter number, and action buttons. This visual noise makes thread drafting feel like form-filling instead of writing.

**How to apply**: Replace `ThreadComposer.svelte`'s card model with a continuous writing surface. Separator affordances replace card borders as the primary interaction target. Internally, the editor still emits `ThreadBlock[]` — the change is purely visual/interaction.

### 2. Content-Determined Mode (Priority: High)

**What Typefully does**: There's no "tweet vs. thread" selector. You just start writing. Adding a separator makes it a thread; removing all separators makes it a tweet. The interface handles both without requiring the user to decide upfront.

**Why it matters**: The current mode tabs force users to decide "am I writing a tweet or a thread?" before they've started writing. This is a premature commitment that breaks writing flow, especially for users who might start with a thought that grows into a thread.

**How to apply**: Remove the mode tabs from the header. A single-segment draft is a tweet; the moment a separator is added, it becomes a thread. The `ComposeModal` continues to track `mode` internally for submission logic, but the UI doesn't expose it as a toggle.

### 3. Inline Preview Toggle (Priority: Medium)

**What Typefully does**: Preview is a mode toggle, not a permanent side-by-side pane. Clicking a preview button swaps the editing canvas for a preview rendering in the same space. Toggling back returns to the editor.

**Why it matters**: The current 50/50 `compose-layout` grid gives half the modal width to preview at all times. In tweet mode, this means half the modal shows a single preview card — massive waste of space. In thread mode, the preview column adds value but still steals width from the writing surface. The preview also doesn't scroll-sync with the editor.

**How to apply**: Replace the side-by-side grid with an inline toggle. The preview button in the header bar switches the canvas between edit mode and preview mode. Preview reuses the existing `TweetPreview` component, just rendered in-place.

### 4. Low-Noise Chrome (Priority: Medium)

**What Typefully does**: The compose surface has almost no visible chrome. No "Compose" heading, no tab bar, no labeled footer. Secondary controls live in a minimal sidebar or appear contextually.

**Why it matters**: `ComposerShell.svelte` currently renders: "Compose" heading + date, focus/close buttons, Tweet/Thread tabs, and a 5-element footer (AI Assist, Notes, spacer, Cancel, Submit). Each of these is a fixed chrome element that competes with the writing surface for attention and space.

**How to apply**: Strip the header to just close button + preview toggle + focus-mode toggle. Move AI and notes to the command palette and an optional inspector rail. Replace the footer with a floating submit button inside the canvas.

## Patterns to Exceed (Where Tuitbot Can Be Better)

### 1. AI Integration Depth

Typefully's AI is basic (suggest, rewrite). Tuitbot already has:
- **Voice context panel** with brand voice, content pillars, and quick cue input
- **Inline improve** with selection-aware rewriting (Cmd+J)
- **From-notes generation** that transforms rough notes into polished tweets or threads
- **Per-selection improvement** in both tweet and thread modes

The opportunity is to surface these better. Move voice context to the inspector rail so it doesn't gate the writing surface. Make Cmd+J more discoverable with a tooltip on first use.

### 2. Command Palette Breadth

Tuitbot's Cmd+K palette already supports thread actions (add-card, duplicate, split, merge, move-up, move-down), mode switching, focus mode, and AI actions. Typefully has no equivalent.

The opportunity is to make the palette even more powerful: add "split at cursor", "merge all", "reverse thread order", "auto-number tweets". These actions are especially valuable in the unibody model where there are fewer visible buttons.

### 3. Thread-to-Tweet Fluidity

Typefully still has some friction when converting a thread back to a single tweet (you have to manually delete all separators). Tuitbot can make this seamless: when a user removes the last separator in a thread, it silently becomes a tweet. The `mode` state tracks this automatically — no user action needed beyond editing.

### 4. Real-Time Inline Preview

Instead of Typefully's toggle-between-modes approach, Tuitbot can show a subtle inline preview below each separator that updates live as the user types. This gives preview feedback without leaving the editor. The full preview toggle is still available for a detailed X-style rendering.

## Patterns to Skip

### 1. Auto-Numbering (e.g., "1/N" suffixes)

Typefully offers automatic thread numbering. This is a polarizing feature — many X users find numbered threads annoying, and it's easy to add manually. Skip for now; can be added later as a setting if user demand emerges.

### 2. Blog Unrolling / URL-to-Thread

Typefully can convert a blog post URL into a thread. This is a substantial feature requiring web scraping, summarization, and thread decomposition. Out of scope for this epic — would need its own session sequence.

### 3. Collaborative Editing / Team Features

Typefully has team plans with collaborative drafting. Tuitbot is a local-first single-user tool. Skip entirely.

### 4. Thread Analytics Post-Publishing

Typefully shows per-tweet engagement in threads after publishing. Tuitbot has its own analytics pipeline. Not relevant to the composer UX.

### 5. Calendar-First Scheduling View

Typefully's calendar is a key feature. Tuitbot already has a content calendar page. The composer's scheduling UX (TimePicker) just needs better positioning, not a reimagining.
