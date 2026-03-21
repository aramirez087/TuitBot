# Session 06 Handoff: Hook-First Compose Workflow

## What Changed

The Ghostwriter compose flow now includes a hook selection step. After extracting highlights from vault chunks (or receiving an Obsidian selection), users generate 5 differentiated hook options, compare them side-by-side, choose one, and produce a grounded tweet or thread from the chosen hook. Source context (node IDs, highlights, selections) is preserved across retries.

### Files Created

| File | Purpose |
|---|---|
| `dashboard/src/lib/utils/hookStyles.ts` | Style label mapping (`contrarian_take` → "Hot Take") and confidence badge utility |
| `dashboard/src/lib/components/composer/HookPicker.svelte` | Hook picker component: card grid, single-select, regenerate, format toggle, loading/error states |
| `dashboard/tests/unit/hookStyles.test.ts` | 12 tests for style labels and confidence badges |
| `dashboard/tests/unit/HookPicker.test.ts` | 28 tests for hook picker rendering, selection, confirm, regenerate, keyboard nav, loading, error |
| `docs/roadmap/obsidian-ghostwriter-edge/hook-first-workflow.md` | Decision log for this session (6 decisions) |
| `docs/roadmap/obsidian-ghostwriter-edge/session-06-handoff.md` | This file |

### Files Modified

| File | Change |
|---|---|
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Added hook step state machine: hookOptions, hookLoading, hookError state; handleGenerateHooksFromHighlights, handleHookSelected, handleRegenerateHooks, handleBackToHighlights functions; HookPicker in template between highlights and chunk selection |
| `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` | Added hook step: hookOptions/hookLoading/hookError state; handleGenerate now generates hooks instead of content directly; handleHookSelected, handleRegenerateHooks, handleBackFromHooks functions; HookPicker in template |
| `dashboard/src/lib/components/composer/VaultHighlights.svelte` | Button label changed from "Generate tweet"/"Generate thread" to "Find hooks" |
| `dashboard/src/lib/components/composer/VaultFooter.svelte` | Button label changed from "Generate from selection" to "Generate hooks" |
| `dashboard/tests/unit/VaultHighlights.test.ts` | Updated 2 tests for new "Find hooks" button label |
| `dashboard/tests/unit/FromVaultPanel.test.ts` | Updated "Generate from selection" → "Generate hooks" test; added `api.assist.hooks` mock |

## Decisions Made

See `hook-first-workflow.md` for full decision log (6 decisions).

Key decisions:
1. **Hook step is between highlights and generation** — not replacing either step
2. **Single-select** — one hook seeds one piece of content
3. **Regenerate-all only** — no per-hook regen (requires backend `preferred_style` param)
4. **Hook text passed as highlight context** — reuses existing `handleGenerateFromVault` pipeline
5. **Source context preserved** — node IDs, highlights, selections survive hook retries
6. **Style labels in UI layer** — `hookStyles.ts` maps raw backend keys to human labels

## Exit Criteria Met

- [x] User can generate 5 hooks from highlights (chunk selection path)
- [x] User can generate 5 hooks from Obsidian selection (selection path)
- [x] User can compare hooks side-by-side with style labels and confidence badges
- [x] User can select one hook and produce a grounded draft
- [x] User can regenerate all hooks without losing source context
- [x] User can go back to highlights/selection from hook picker
- [x] User can switch between tweet/thread format at hook step
- [x] Loading state shows shimmer cards during hook generation
- [x] Error state shows inline error with retry button
- [x] Keyboard navigation (Enter/Space) works for hook selection
- [x] `npm --prefix dashboard run check` passes (0 errors)
- [x] `npm --prefix dashboard run test:unit:run` passes (673 tests, 34 suites)
- [x] 40 new tests (28 HookPicker + 12 hookStyles)
- [x] All existing tests updated and passing

## What Session 7 Needs

1. **ProvenanceRef pipeline integration** (carried from Session 4 → 5 → 6): Ensure ProvenanceRef is properly attached through the hook → compose → API chain. Currently the hook text flows as a highlight, but no explicit ProvenanceRef with node_id/chunk_id/source_path is being constructed from the hook generation context.
2. **Per-hook regeneration**: Requires backend `preferred_style` parameter on `POST /api/assist/hooks`. Users want to keep a style but get different text.
3. **Hook preference persistence**: Track which hook styles users choose most often; surface preferred styles first.
4. **Hook analytics**: Which styles get chosen, rejected, regenerated — feed into recommendation engine.
5. **`vault_selections::cleanup_expired()` wiring** (carried from Session 4 → 5 → 6): Wire into the server's hourly cleanup loop.
6. **End-to-end flow verification**: Verify the full path from Obsidian selection → hook generation → hook selection → content generation → compose/schedule works with real LLM backend.

## Open Risks

1. **Hook generation latency**: The LLM call for hooks adds ~2-4s to the compose flow. The shimmer loading state mitigates perceived latency, but power users may want hooks pre-generated in the background when highlights are being reviewed.
2. **Regenerate-all discards all hooks**: If a user likes 4/5 hooks but wants to re-roll one, they lose all 5. Per-hook regen is the fix (Session 7 item 2).
3. **Selection TTL vs. hook generation time** (carried from Session 4): If a user takes >30 minutes after selection before generating hooks, the selection is expired server-side. The frontend mitigates by fetching immediately, but `session_id` resolution will return None.
4. **No `ComposerInspector` or `ComposeWorkspace` changes needed**: The hook text flows through the existing highlight context pipeline. If future sessions change the generation signature, this integration point may need updating.
