# Hook Miner UI Notes — State Machine and Copy Choices

**Date:** 2026-03-22
**Session:** 04
**Status:** Implemented

---

## 1. UI State Machine

VaultSelectionReview manages the following states. Each state maps to a specific component or rendering branch in the template.

### States

| State | Guard | Rendered Component |
|-------|-------|--------------------|
| `loading` | `loading === true` | Shimmer + "Loading selection..." |
| `expired` | `expired === true` | Expired state with "Browse vault" button |
| `angle_fallback` | `selection && showAngleFallback` | `AngleFallback` |
| `angle_cards` | `selection && (angleResult \|\| angleLoading)` | `AngleCards` |
| `hook_picker` | `selection && (hookOptions \|\| hookLoading)` | `HookPicker` |
| `selection_review` | `selection` (default) | Selection preview + GraphSuggestionCards + VaultFooter |

### State Transitions

```
                    ┌──────────────────────┐
                    │   selection_review    │
                    │                      │
                    │  VaultSelectionReview │
                    │  GraphSuggestionCards │
                    │  VaultFooter         │
                    └──────────┬───────────┘
                               │
                    User clicks "Generate hooks"
                               │
                ┌──────────────┼──────────────┐
                │                             │
    synthesis ON                    synthesis OFF
    + neighbors > 0                 OR neighbors = 0
                │                             │
                ▼                             ▼
    POST /api/assist/angles      POST /api/assist/hooks
                │                             │
        ┌───────┼───────┐                     │
        │               │                     │
    angles OK     fallback_reason             │
        │               │                     │
        ▼               ▼                     ▼
┌──────────────┐ ┌──────────────┐   ┌──────────────┐
│ angle_cards  │ │angle_fallback│   │ hook_picker  │
│              │ │              │   │              │
│ AngleCards   │ │AngleFallback │   │ HookPicker   │
└──────┬───────┘ └──────┬───────┘   └──────────────┘
       │                │
       │         ┌──────┼──────┐
       │         │             │
       │  "Use generic    "Back to
       │   hooks"          related notes"
       │         │             │
       │         ▼             ▼
       │  ┌──────────┐  ┌──────────────┐
       │  │hook_picker│  │selection_    │
       │  └──────────┘  │review        │
       │                └──────────────┘
       │
  ┌────┼────────────┐
  │                  │
"Mine again"   "More hook styles"
  │                  │
  ▼                  ▼
POST /assist/    POST /assist/hooks
angles (retry)   → hook_picker
```

### Transition Guards

| Trigger | Guard | Target State |
|---------|-------|-------------|
| Generate clicked | `synthesisEnabled && acceptedNeighbors.size > 0` | `angle_loading` → `angle_cards` or `angle_fallback` |
| Generate clicked | `!synthesisEnabled \|\| acceptedNeighbors.size === 0` | `hook_loading` → `hook_picker` |
| "Use generic hooks" | Always | `hook_loading` → `hook_picker` |
| "Back to related notes" | Always | `selection_review` |
| "Mine again" | Always | `angle_loading` → `angle_cards` or `angle_fallback` |
| "More hook styles" | Always | `hook_loading` → `hook_picker` |
| "Use this angle" | `selectedIndex !== null` | Calls `ongenerate` → parent handles |
| Back arrow (AngleCards) | Always | `selection_review` |
| Back arrow (HookPicker) | Always | `selection_review` |

### State Preservation

The following state is **never reset** by view transitions:
- `acceptedNeighbors` (Map)
- `dismissedNodeIds` (Set)
- `synthesisEnabled` (boolean)
- `selection` (VaultSelectionResponse)

The following state **is reset** when transitioning back:
- `angleResult`, `angleError`, `angleLoading`, `showAngleFallback`, `angleFallbackReason` — cleared by `handleBackToNeighbors`
- `hookOptions`, `hookError` — cleared by `handleBackFromHooks`

---

## 2. Copy Choices

All final copy strings, confirmed from UX spec section 10.

| Element | Copy | Source |
|---------|------|--------|
| Section header | `MINED ANGLES` | UX spec §10 |
| Loading label | `Mining angles from your notes...` | UX spec §10 |
| Confirm button | `Use this angle` | UX spec §10 |
| Regenerate button | `Mine again` | UX spec §10 |
| Fallback button | `More hook styles` | New — bridges to HookPicker |
| API error message | `Failed to mine angles` | UX spec §10 |
| Error retry button | `Retry` | Matches HookPicker pattern |
| Fallback heading (weak signal) | `NOT ENOUGH SIGNAL` | UX spec §10 |
| Fallback body | `Your selected notes didn't surface enough evidence for mined angles. You can include more related notes or use generic hooks instead.` | UX spec §10 |
| Fallback primary action | `Use generic hooks` | UX spec §10 |
| Fallback secondary action | `← Back to related notes` | UX spec §10 |
| Timeout message | `Mining took too long. Try again or use generic hooks.` | UX spec §10 |
| Parse error message | `Couldn't parse mined angles. Try again or use generic hooks.` | UX spec §10 |
| Back button aria-label | `Back to related notes` | Matches context |
| Card list aria-label | `Mined angle options` | UX spec §12 |

---

## 3. CSS Patterns

### Reused from HookPicker

| AngleCards Class | HookPicker Source | Purpose |
|-----------------|-------------------|---------|
| `.angle-type-pill` | `.hook-style-pill` | Exact same sizing, colors, weight |
| `.angle-seed-text` | `.hook-text` | 13px, line-height 1.45 |
| `.angle-card-footer` | `.hook-card-footer` | Flex row, space-between |
| `.angle-char-count` | `.hook-char-count` | 10px, text-subtle |
| `.angle-confidence` | `.hook-confidence` | Confidence badge styling |
| `.angle-confirm-btn` | `.hook-confirm-btn` | Primary accent button |
| `.angle-format-toggle` | `.hook-format-toggle` | Tweet/Thread radio group |
| `.angle-format-opt` | `.hook-format-opt` | Toggle options |
| `.angle-back` | `.hook-back` | Back arrow button |
| Shimmer keyframe | Same `@keyframes shimmer` | Loading animation |

### New in AngleCards

| Class | Purpose |
|-------|---------|
| `.angle-evidence` | Evidence container with subtle border |
| `.angle-evidence-item` | Flex row for each evidence entry |
| `.angle-evidence-pill` | 9px type pill with dynamic color via CSS var |
| `.angle-evidence-citation` | 11px truncated citation text |
| `.angle-evidence-source` | 10px "from" attribution line |
| `.angle-loading-label` | Loading state label (10px uppercase) |
| `.angle-remine-btn` | "Mine again" button |
| `.angle-fallback-btn` | "More hook styles" button |
| `.shimmer-evidence-block` | Evidence area shimmer placeholder |

### New in AngleFallback

| Class | Purpose |
|-------|---------|
| `.angle-fallback` | Centered flex container |
| `.angle-fallback-heading` | 10px uppercase heading |
| `.angle-fallback-body` | 12px body text |
| `.angle-fallback-primary` | Primary action button (accent) |
| `.angle-fallback-secondary` | Secondary action button (border) |

---

## 4. Decisions Made in This Session

| ID | Decision | Rationale |
|----|----------|-----------|
| D20 | `acceptedNeighborIds` is required positional param in `api.assist.angles()` | Matches Rust endpoint contract. |
| D21 | Response types use `string` not TS enums | Flexible — new types from backend don't break frontend. |
| D22 | Separate `angleStyles.ts` from `hookStyles.ts` | Different domains. Shared `getConfidenceBadge` imported, not duplicated. |
| D23 | `AngleFallback` is a separate component | Keeps template manageable. |
| D24 | "More hook styles" preserves neighbor state | `acceptedNeighbors`/`dismissedNodeIds` are component-level, not view-level. |
| D25 | From Vault chunk-selection path NOT modified | No neighbors = no evidence. |
| D26 | AngleCards shares HookPicker's format toggle/confirm exactly | Visual consistency. |
| D27 | No client-side AbortController timeout in V1 | Simplicity. Fallback path handles errors. |
| D28 | Angle type labels added to `hookStyles.ts` STYLE_LABELS | Downstream compose/provenance uses `hookStyle` field — angle types must display correctly there too. |
