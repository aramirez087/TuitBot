# Hook-First Workflow: Decision Log

## Overview

Session 6 integrates the hook generation engine (Session 5) into the Ghostwriter compose flow. Users now generate 5 hook options, compare them, choose one, and produce a grounded tweet or thread from that hook — all without losing source context.

## Decisions

### D1: Hook step inserted between highlights and generation

The hook picker is a distinct step in the compose flow, not a replacement for highlights. Highlights let users curate *which ideas matter*. Hooks let them choose *the right opening angle*. Both are valuable. The flow is: chunks → highlights → hooks → final content.

**Alternative rejected:** Embedding hook selection inside VaultHighlights. This would conflate two user decisions and bloat the component.

### D2: Single-select hook picker

Users pick exactly one hook to seed downstream generation. Multi-select was rejected because the downstream generation pipeline (`api.assist.improve` / `api.assist.thread`) expects a single seed, not a composite.

### D3: Regenerate-all only (no per-hook regen in v1)

Clicking "Regenerate" re-rolls all 5 hooks. Per-hook regeneration would require a new backend parameter (`preferred_style`) on `POST /api/assist/hooks`, which does not exist yet. Deferred to a future session.

### D4: Hook text passed as highlight context to existing generation pipeline

The selected hook's `text` is passed as `highlights: [hook.text]` to the existing `handleGenerateFromVault` method in `ComposerInspector`. This reuses the highlight-to-content pipeline without any backend changes. The hook text acts as a high-quality, pre-crafted highlight.

### D5: Source context preserved across hook retries

`cachedNodeIds`, `highlightsStep`, and `selectedChunks` are all preserved during hook generation and regeneration. Users can:
- Go back to highlights, toggle highlights on/off, and re-generate hooks
- Regenerate hooks without losing their chunk selections
- Switch between tweet/thread format at the hook step

### D6: Style labels are a UI-layer concern

The backend returns raw `TweetFormat` keys (e.g., `contrarian_take`). Human-readable labels (e.g., "Hot Take") are mapped in `hookStyles.ts`. This keeps the backend format-agnostic and allows the UI to evolve labels independently.

## Component Architecture

```
FromVaultPanel
  ├── VaultNoteList (chunk selection)
  ├── VaultHighlights (highlight curation) → "Find hooks" button
  ├── HookPicker (hook selection) → "Use this hook" button
  └── [generation via ComposerInspector.handleGenerateFromVault]

VaultSelectionReview (Obsidian selection path)
  ├── Selection preview → "Generate hooks" button
  ├── HookPicker (hook selection) → "Use this hook" button
  └── [generation via ongenerate callback]
```

## Files Changed

| File | Change |
|---|---|
| `dashboard/src/lib/utils/hookStyles.ts` | New: style label mapping + confidence badge utility |
| `dashboard/src/lib/components/composer/HookPicker.svelte` | New: hook picker component with card grid, selection, regenerate |
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Added hook step state machine between highlights and generation |
| `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` | Added hook step for Obsidian selection path |
| `dashboard/src/lib/components/composer/VaultHighlights.svelte` | Button label: "Generate tweet/thread" → "Find hooks" |
| `dashboard/src/lib/components/composer/VaultFooter.svelte` | Button label: "Generate from selection" → "Generate hooks" |
| `dashboard/tests/unit/hookStyles.test.ts` | New: 12 tests for style labels + confidence badges |
| `dashboard/tests/unit/HookPicker.test.ts` | New: 28 tests for hook picker behavior |
| `dashboard/tests/unit/VaultHighlights.test.ts` | Updated button label assertions |
| `dashboard/tests/unit/FromVaultPanel.test.ts` | Updated button label + added hooks mock |
