# UX Copy & State Notes — Backlink Synthesizer

## Copy Decisions

### GraphSuggestionCards
| Element | Before | After | Rationale |
|---------|--------|-------|-----------|
| Header | "Related notes" | "Related notes from your vault" | Anchors provenance — user knows these come from *their* knowledge base |
| Action buttons | Intent-specific ("Use as pro-tip", etc.) | Unified "Include" | Reduces decision fatigue; intent badge already communicates role |
| Dismiss | Icon-only (X) | "Skip" with X icon | Explicit label reduces accidental dismissal anxiety |
| Empty (no links) | "No linked notes found" | "This note doesn't link to other indexed notes. You can still generate from this selection alone." | Explains *why* empty + offers next step |
| Empty (not indexed) | "This note isn't indexed yet" | "This note hasn't been indexed yet. Generating from your selected text." | Reassures the system is still working |

### VaultSelectionReview
| Element | Before | After | Rationale |
|---------|--------|-------|-----------|
| Expired | "Selection expired." | "This selection has expired." + "Please send a new selection from Obsidian." | Two-line split: what happened + what to do |
| Toggle | "Related notes ON/OFF" | "✓ Use related notes" | Checkbox metaphor; less binary/robotic |
| Accepted count | "N related note(s) will be included" | "N note(s) included in context" | Shorter, present-tense, less verbose |

### SlotTargetPanel
| Element | Before | After | Rationale |
|---------|--------|-------|-----------|
| Section header | "Refine specific slots" | "Refine specific parts" | "Slots" is dev jargon; "parts" is user-natural |
| Applied list header | "Applied suggestions" | "Applied refinements" | "Refinements" matches the action verb |
| Action button | "Apply" | "Refine" | Matches header language, signals transformation not just insertion |
| Empty state | "Accept related notes above..." | "Include related notes above..." | Matches the "Include" button copy in cards |

## State & Interaction Patterns

### Dismissed Card Recovery
- Dismissed cards move to a collapsed "Show skipped (N)" section
- Each skipped card has an "Undo" button to restore it
- Recovery is session-scoped — refresh clears dismissed state
- Prevents regret from accidental dismissal without cluttering the default view

### Animation
- Cards use `fly` transitions (y: 8px in, y: -8px out, 150ms/120ms)
- `prefers-reduced-motion` respected: duration set to 0
- `matchMedia` guarded for test environments (jsdom compatibility)
- No `flip` animation (cards don't reorder, only add/remove)

### Insert State Flow
- `DraftInsertState` tracks applied refinements with full undo history
- Propagated via callback pattern (`oninsertstatechange`) not `$bindable`
  - Reason: Svelte 5 disallows binding `undefined` to props with default values
- Chain: ComposeWorkspace → ComposerInspector (owns state) → back via callback
- Display chain: ComposeWorkspace → ComposerCanvas → ThreadFlowLane → ThreadFlowCard
