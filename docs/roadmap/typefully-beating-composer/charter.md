# Composer Overhaul Charter

## Objective

Make Tuitbot's composer clearly stronger than Typefully on two pillars:

1. **Distraction-free writing assistance** that turns rough notes into polished threads while reinforcing the user's voice
2. **High-fidelity thread preview** that shows tweet breaks and media crop behavior before publishing

Typefully is a typewriter. Tuitbot is a growth co-pilot. The autonomous engagement loop is the moat — this overhaul makes the manual composition surface match that ambition.

---

## Pillar 1: Distraction-Free Writing Assistance

### Current state

The composer has focus mode (fullscreen), a from-notes panel, AI Improve (`Cmd+J`), and AI Assist (generate from scratch). 14 keyboard shortcuts cover all compose operations. Auto-save with recovery is in place.

### Gaps

| Gap | Impact |
|-----|--------|
| Voice settings invisible during composition | User cannot see or steer what the AI will produce |
| No quick-cue overrides per request | User must leave composer to change persona settings |
| Winning DNA disconnected from assist | Assist endpoints don't use historical performance patterns |
| From-notes panel uses destructive replacement | `confirm()` dialog is the only protection — no undo |
| ComposeModal at 1,273 lines | Adding features makes the file harder to maintain |

### V1 definition: "Learns your voice"

"Voice learning" in v1 means **transparent voice context threading** — the AI's knowledge of your voice is visible, steerable, and draws from your actual posting history.

**Included in v1:**

1. **Voice context bar** — A collapsible panel in the composer showing the active `brand_voice`, `content_style`, and up to 3 `content_pillars`. The user sees exactly what context the AI will use.

2. **Quick-cue input** — A one-line text field (e.g., "make it more casual", "add a hot take", "shorter sentences") appended to the LLM prompt alongside the configured voice. Lets the user steer tone per-request without leaving the composer.

3. **Winning DNA integration** — Wire the existing `winning_dna::build_draft_context()` into the `/api/assist/improve`, `/api/assist/tweet`, and `/api/assist/thread` endpoints. Generated content draws from patterns in the user's historically successful posts. This is the actual learning — it already exists in core but isn't connected to the compose assist flow.

4. **Saved cue shortcuts** — Frequently-used quick cues saved to `localStorage` (top 5, MRU). Builds a personal shortcut palette over time. Not stored server-side.

**Explicitly NOT in v1:**

- Auto-updating `brand_voice` config fields based on published content
- Implicit style fingerprinting (sentence length, emoji frequency, etc.)
- Engagement-weighted archetype/format selection probabilities
- Server-side cue persistence or cue sharing across devices

**Rationale:** Uses existing infrastructure (persona config, winning DNA, content generator `context` parameter). Requires no DB migrations. Delivers visible value immediately.

---

## Pillar 2: High-Fidelity Thread Preview

### Current state

`TweetPreview.svelte` (191 lines) renders avatar placeholder, handle, text, and a basic media grid. Thread connectors link cards. Media uses fixed `16:9` aspect ratio with `object-fit: cover` regardless of image count or dimensions.

### Gaps

| Gap | Impact |
|-----|--------|
| Media grid doesn't match X layouts | 1/2/3/4 image arrangements differ from actual X presentation |
| No intrinsic dimension awareness | Portrait vs landscape images crop identically |
| Tweet mode has no preview | Only threads get a preview pane |
| Video shows raw `<video>` element | No play button overlay or poster frame |
| No crop expectation signal | Users surprised by how images appear after posting |

### V1 fidelity rules

The preview emulates X's layout rules for text and images in threads and single tweets.

**Included in v1:**

1. **X-accurate media grids:**
   - 1 image: 16:9 landscape crop (X's default for uploaded images)
   - 2 images: side-by-side, each cropped to approximately square
   - 3 images: large left column (spanning full height) + two stacked right
   - 4 images: 2x2 grid, each approximately square

2. **Crop indicator** — For images where the preview crops significantly (e.g., portrait photos in landscape slots), show a subtle visual indicator so the user knows content will be hidden.

3. **Tweet-mode preview** — Add a preview pane for single tweets (not just threads), showing avatar + text + media in X style. Appears alongside the textarea in standard mode, below in mobile.

4. **Video poster frame** — Show first frame (or blank placeholder) with a centered play icon overlay, matching X's presentation.

**Explicitly NOT in v1:**

- URL unfurling / link card preview
- GIF animation toggle in preview
- Dark/light theme preview switching (follows app theme)
- Quote-tweet preview embedding
- Poll preview rendering

**Rationale:** The 4 media grid patterns cover the vast majority of composed content. Link cards require external URL fetching and parsing which adds latency and complexity. The priority is making the common case accurate.

---

## Hard Constraints

1. **File size limits** — No Svelte file above 400 lines. No Rust file above 500 lines. Extract into submodules/subcomponents.
2. **Backward compatibility** — `ComposeRequest` interface and all existing compose contract tests must continue to pass unchanged.
3. **Composer mode guarantees** — Approval mode behavior, scheduling flow, drafts, and Discovery Feed integration remain intact.
4. **No placeholder UX** — Every visible control must be functional. No "coming soon" buttons or undocumented behavior.
5. **Architecture layers** — New domain logic in `tuitbot-core`. Server crate stays thin. Dashboard follows Svelte 5 patterns.

## Non-Goals

- SSR preview rendering
- Third-party composer integrations
- Analytics-driven content generation tuning
- Multi-account composer
- Collaborative editing
