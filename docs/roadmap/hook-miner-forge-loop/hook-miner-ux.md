# Hook Miner UX Spec

**Date:** 2026-03-22
**Session:** 02
**Status:** Locked for implementation (S03-S05)

---

## 1. Overview

Hook Miner replaces the first step of the Ghostwriter compose flow. Instead of generating 5 generic hook styles (`question`, `contrarian_take`, `tip`, `stat`, `teaser`), the system mines 3 evidence-backed angles from the user's accepted graph neighbors.

**What changes:**
- The "Generate" button on `VaultFooter` calls `POST /api/assist/angles` (not `/api/assist/hooks`) when the user has accepted at least one neighbor and synthesis is enabled.
- The `HookPicker` component is replaced in the flow by a new `AngleCards` component that shows mined angles with evidence citations.
- The existing `HookPicker` remains unchanged as the fallback when evidence is insufficient.

**What does not change:**
- Selection review (`VaultSelectionReview`) is unchanged.
- Graph suggestion cards (`GraphSuggestionCards`) accept/dismiss/restore flow is unchanged.
- Draft generation (`ongenerate` callback) is unchanged.
- Approval queue, publish, and loopback pipelines are unchanged.

---

## 2. Entry Points

### Obsidian Selection Flow
`send-selection` from Obsidian plugin -> `VaultSelectionReview` -> accept neighbors -> "Generate" -> **AngleCards** or fallback.

### From Vault Flow
`FromVaultPanel` -> search notes -> select chunks -> "Generate" -> hooks (unchanged, no neighbor context). If the user enters `FromVaultPanel` via a `selectionSessionId`, it delegates to `VaultSelectionReview`, which follows the Obsidian Selection flow above.

The From Vault chunk-selection path does not gain angle mining in V1. It lacks graph neighbors, so it always uses the existing generic hook path.

---

## 3. Flow Diagram

```
Selection arrives from Obsidian
        |
        v
[VaultSelectionReview]
  - Shows selection preview
  - Shows GraphSuggestionCards
  - User accepts/dismisses neighbors
        |
        v
User clicks "Generate" on VaultFooter
        |
        +-- synthesisEnabled AND acceptedNeighbors.size > 0?
        |       |
        |       YES -> POST /api/assist/angles
        |               |
        |               +-- response.fallback_reason is non-null?
        |               |       |
        |               |       YES -> [FallbackState]
        |               |               |
        |               |               +-- "Use generic hooks" -> POST /api/assist/hooks -> [HookPicker]
        |               |               |
        |               |               +-- "Back to related notes" -> [GraphSuggestionCards]
        |               |
        |               +-- response.angles present?
        |                       |
        |                       YES -> [AngleCards]
        |                               |
        |                               +-- User selects angle -> "Use this angle"
        |                               |       -> ongenerate(nodeIds, format, [seed_text], angle_type, neighborProv)
        |                               |
        |                               +-- "Mine again" -> POST /api/assist/angles (retry)
        |
        +-- else (synthesis off or 0 neighbors)
                |
                POST /api/assist/hooks -> [HookPicker] (existing flow, unchanged)
```

---

## 4. Angle Card Anatomy

Each angle card follows the visual language of the existing `hook-card` but adds an evidence section.

```
+---------------------------------------------------------------+
|  [STORY]                                                       |
|                                                                |
|  I spent 3 months migrating from X to Y. Here's what nobody   |
|  tells you about the hidden costs...                           |
|                                                                |
|  +-- Evidence -----------------------------------------------+ |
|  | [data_point]  "migration cost 3.2x the initial est..."    | |
|  |               from "Migration Retrospective"               | |
|  |                                                            | |
|  | [contradiction]  "vendor claimed 2-week migration..."      | |
|  |                  from "Vendor Evaluation Notes"             | |
|  +------------------------------------------------------------+ |
|                                                                |
|  145 chars                                           [high]    |
+---------------------------------------------------------------+
```

### Layout specification

| Element | Visual Treatment |
|---------|-----------------|
| **Angle type pill** | Top-left. Same CSS as `.hook-style-pill`: `color-mix(accent 12%)` background, accent text, 10px uppercase, 600 weight. Values: `STORY`, `LISTICLE`, `HOT TAKE`. |
| **Seed text** | Body text. Same CSS as `.hook-text`: 13px, `--color-text`, `line-height: 1.45`. Max 280 chars (tweet-length). |
| **Evidence section** | Below seed text, inside a subtle container (`1px border --color-border-subtle`, `border-radius: 4px`, `padding: 6px 8px`). |
| **Evidence type pill** | Inside evidence section. 9px uppercase, 500 weight. Colors by type: `contradiction` = `--color-warning`, `data_point` = `--color-accent`, `aha_moment` = `--color-success`. |
| **Citation text** | Inline after pill. 11px, `--color-text-muted`, max 40 chars with ellipsis truncation. |
| **Source attribution** | Below citation. 10px, `--color-text-subtle`. Format: `from "Note Title"`. |
| **Card footer** | Same layout as `.hook-card-footer`: char count left, confidence badge right. Reuses `getConfidenceBadge()`. |
| **Selection** | Click-to-select, same pattern as HookPicker: `selected` class adds accent border + `6%` accent background. ARIA `role="option"`, `aria-selected`. |
| **Rationale** | Shown as `title` attribute on the angle type pill. 1-sentence explanation of why this angle was chosen. Not visible by default — appears on hover/long-press. |

### Card count

Always 3 cards max, 0 minimum (0 = fallback state). Cards with 0 evidence items are filtered server-side and never rendered.

---

## 5. Loading States

3 shimmer cards (matching the count of expected angles, not 5 like HookPicker). Uses the same shimmer animation pattern as `.hook-card-shimmer`.

```
+---------------------------------------------------------------+
|  [shimmer-pill 60px]                                           |
|  [shimmer-line 100%]                                           |
|  [shimmer-line 75%]                                            |
|  [shimmer-evidence-block 90% height:40px]                      |
|  [shimmer-footer 40%]                                          |
+---------------------------------------------------------------+
```

Label above the shimmer cards: **"Mining angles from your notes..."** in 10px uppercase, `--color-text-subtle`, same style as `.graph-suggestions-label`.

---

## 6. Fallback States

### 6.1 Weak Signal (evidence threshold not met)

Triggered when `response.fallback_reason` is non-null. The user sees a non-alarming informational state.

```
+---------------------------------------------------------------+
|                                                                |
|  NOT ENOUGH SIGNAL                                             |
|                                                                |
|  Your selected notes didn't surface enough evidence for        |
|  mined angles. You can include more related notes or use       |
|  generic hooks instead.                                        |
|                                                                |
|  [Use generic hooks]            [<- Back to related notes]     |
|                                                                |
+---------------------------------------------------------------+
```

| Element | Spec |
|---------|------|
| Heading | `NOT ENOUGH SIGNAL` — 10px uppercase, 600 weight, `--color-text-subtle` |
| Body | 12px, `--color-text-muted`, `line-height: 1.5` |
| Primary action | `Use generic hooks` — calls `POST /api/assist/hooks`, transitions to `HookPicker` |
| Secondary action | `<- Back to related notes` — returns to `GraphSuggestionCards` so user can accept more neighbors |

**State preservation:** Accepted neighbors, dismissed neighbors, synthesis toggle state, and selection context are all preserved across both actions. The user can go back to neighbors, accept more, and try "Generate" again.

### 6.2 No Neighbors Accepted

When `synthesisEnabled` is true but `acceptedNeighbors.size === 0`, the "Generate" button calls the existing `POST /api/assist/hooks` directly. The user never sees the fallback state — they get generic hooks immediately. This is the same behavior as today.

### 6.3 Synthesis Disabled

When `synthesisEnabled` is false, the "Generate" button calls `POST /api/assist/hooks` directly. Same as today.

---

## 7. Error and Recovery States

### 7.1 API Error (network failure, 5xx)

Same pattern as HookPicker's `.hook-error`: red banner with error message + "Retry" button.

```
+---------------------------------------------------------------+
| [!] Failed to mine angles                          [Retry]     |
+---------------------------------------------------------------+
```

### 7.2 Timeout

If the angle mining request exceeds the client-side timeout (15s):

```
+---------------------------------------------------------------+
|                                                                |
|  Mining took too long. Try again or use generic hooks.         |
|                                                                |
|  [Mine again]                   [Use generic hooks]            |
|                                                                |
+---------------------------------------------------------------+
```

### 7.3 Malformed Response

If the server returns angles that fail client-side validation (missing required fields):

```
+---------------------------------------------------------------+
|                                                                |
|  Couldn't parse mined angles. Try again or use generic hooks.  |
|                                                                |
|  [Mine again]                   [Use generic hooks]            |
|                                                                |
+---------------------------------------------------------------+
```

### 7.4 All Angles Filtered (0 evidence each)

If the server returns `angles: []` with no `fallback_reason`, the client treats this as a fallback state with the weak-signal messaging from section 6.1.

---

## 8. Format Toggle

The Tweet/Thread toggle from HookPicker is preserved identically in `AngleCards`. It appears in the footer row alongside the "Mine again" button. When the user selects an angle and clicks "Use this angle", the current `outputFormat` (tweet or thread) is passed to `ongenerate`.

---

## 9. How Accepted Neighbors Influence Angles

The evidence pipeline is directional, not insertional:

1. **Accept:** User clicks "Include" on a `GraphSuggestionCard`. That neighbor's `node_id`, `snippet`, `heading_path`, and `best_chunk_id` are stored in the `acceptedNeighbors` map.

2. **Extract:** When "Generate" is clicked, the API receives `accepted_neighbor_ids`. The server fetches full neighbor content and extracts evidence items (contradictions, data points, aha moments) using LLM + regex pre-filtering.

3. **Generate angles:** The LLM receives the selection text + extracted evidence items and generates up to 3 angles, each referencing specific evidence from specific neighbors.

4. **Show cards:** The frontend renders angle cards with evidence badges that cite the source neighbor by note title.

5. **Select angle -> draft:** When the user picks an angle, its `seed_text` becomes the `highlights` parameter for `ongenerate`. The `angle_type` becomes the `hookStyle` parameter. The accepted neighbors' `node_id`s remain in `nodeIds` (for RAG context during draft generation). Evidence citations feed into `neighborProvenance`.

6. **Draft generation:** The existing `ongenerate -> api.assist.tweet/thread` pipeline handles the rest. The draft generator uses the angle's seed text as the hook and the neighbor content as RAG context. The draft generator decides what to weave in. No neighbor text is ever pasted verbatim into the draft.

**No auto-insertion:** At no point does accepted neighbor content automatically appear in the draft textarea. The user's voice/persona settings govern the draft's tone. The influence path is: `evidence -> angle -> hook -> draft`, not `evidence -> draft`.

---

## 10. Exact UX Copy

| Element | Copy |
|---------|------|
| Section header (replaces "Choose a Hook") | `MINED ANGLES` |
| Angle card title format | `{AngleType}` pill only (seed text is the body, not the title) |
| Evidence badge format | `{evidence_type}` pill + truncated citation (40 chars max) |
| Evidence source attribution | `from "{note_title}"` in `--color-text-subtle` |
| Fallback state heading | `NOT ENOUGH SIGNAL` |
| Fallback state body | `Your selected notes didn't surface enough evidence for mined angles. You can include more related notes or use generic hooks instead.` |
| Fallback primary action | `Use generic hooks` |
| Fallback secondary action | `<- Back to related notes` |
| Loading state label | `Mining angles from your notes...` |
| Timeout message | `Mining took too long. Try again or use generic hooks.` |
| Parse error message | `Couldn't parse mined angles. Try again or use generic hooks.` |
| API error message | `Failed to mine angles` (with Retry button) |
| Regenerate button | `Mine again` |
| Confirm button | `Use this angle` |
| Back button (to selection review) | Same `<-` arrow icon as HookPicker |

---

## 11. Non-Goals for V1

- No drag-to-reorder angles
- No inline evidence editing or expansion
- No "explain this evidence" tooltip or modal
- No multi-select (pick exactly one angle per generation)
- No angle history or bookmarking
- No evidence quality override by user (thresholds are server-side)
- No angle mining for the From Vault chunk-selection path (no neighbors = no evidence)
- No custom angle type creation
- No evidence preview before generating angles
- No A/B testing of angle types (all 3 shown equally)

---

## 12. Accessibility

Inherit all patterns from `HookPicker`:

- `role="listbox"` on the angle card list, `role="option"` on each card
- `aria-selected` on selected card
- `aria-label="Mined angle options"` on the list
- `tabindex="0"` on each card for keyboard navigation
- `Enter` or `Space` to select a card
- Evidence section: `aria-label="Evidence from {note_title}"` on each evidence item
- Loading state: `role="status"`, `aria-label="Mining angles"`
- Fallback state: `role="status"`
- Error state: `role="alert"`
- `@media (prefers-reduced-motion: reduce)` disables shimmer animations
- `@media (pointer: coarse)` increases touch targets to 44px minimum
