# Implementation Plan: Composer Overhaul

Three build sessions plus this planning session deliver the composer overhaul.

| Session | Focus | Core risk |
|---------|-------|-----------|
| 01 (this) | Charter, slicing, architecture decisions | None (docs only) |
| 02 | Distraction-free writer: refactor + voice + notes | Refactoring 1,273-line file |
| 03 | X-accurate preview: media grids + tweet preview | X layout fidelity |
| 04 | Validation, fixes, docs, go/no-go | Accumulated edge cases |

---

## Session 02: Distraction-Free Writer

### Mission

Refactor the compose surface into maintainable components, wire voice intelligence into the composer, and enhance the notes-to-content flow.

### Task breakdown

#### Task 2.1: Extract ComposeModal into composer/ subcomponents

**Goal:** Bring `ComposeModal.svelte` (1,273 lines) and `ThreadComposer.svelte` (858 lines) under the 400-line limit.

**New files to create:**
```
dashboard/src/lib/components/composer/
  ComposerShell.svelte        — Modal chrome: backdrop, header, mode tabs, footer, focus mode toggle, escape cascade, keyboard dispatch (~250 lines)
  TweetEditor.svelte           — Tweet textarea, char counter, inline media attachments (~150 lines)
  ThreadEditor.svelte          — Thread card list, drag-drop, keyboard nav, validation display (~350 lines, extracted from ThreadComposer)
  ThreadCardActions.svelte     — Per-card action buttons: duplicate, split, merge, delete (~100 lines)
```

**Files to modify:**
- `ComposeModal.svelte` — Becomes thin orchestrator importing subcomponents (~350 lines)
- `ThreadComposer.svelte` — Replaced by `ThreadEditor.svelte` + `ThreadCardActions.svelte`, original file removed or reduced to re-export

**Strategy:** Extract one section at a time. After each extraction, run `cd dashboard && npm run check`. Verify autosave, recovery, keyboard shortcuts, and command palette still work through the orchestrator.

#### Task 2.2: Add VoiceContextBar

**New file:**
```
dashboard/src/lib/components/composer/VoiceContextBar.svelte  (~120 lines)
```

**Behavior:**
- Collapsible bar below mode tabs, above editor
- Shows active `brand_voice` (truncated), `content_style`, and up to 3 `content_pillars` as chips
- Quick-cue input: a single-line text field with placeholder "Add a tone cue (e.g., 'more casual')"
- Saved cues dropdown: MRU list of up to 5 previous cues from `localStorage`
- Cue value threaded into assist API calls as the `context` parameter

**API changes:**
- Frontend `api.assist.improve(draft, context)` already supports the `context` parameter
- Frontend `api.assist.tweet(topic)` and `api.assist.thread(topic)` need the topic to include cue text — prepend cue to topic string (no API contract change needed)

**Data flow:**
1. User types cue in VoiceContextBar
2. ComposeModal reads cue value
3. On AI Improve: passes cue as `context` to `/api/assist/improve`
4. On AI Assist/Generate: prepends cue to `topic` in `/api/assist/tweet` or `/api/assist/thread`

**Voice settings data source:** The settings store at `dashboard/src/lib/stores/settings.ts` already holds the full `TuitbotConfig` in a writable `config` store. The `business` section contains `brand_voice`, `content_style`, and `content_pillars`. VoiceContextBar can subscribe to this store directly — no additional API call needed if the user has visited Settings at least once. As a fallback, call `api.settings.get()` on modal open to populate the store if it's null.

#### Task 2.3: Wire Winning DNA into assist endpoints

**Files to modify (Rust):**
- `crates/tuitbot-server/src/routes/assist.rs` — Add winning DNA context retrieval to `assist_improve`, `assist_tweet`, and `assist_thread` handlers
- `crates/tuitbot-core/src/content/generator.rs` — Ensure `generate_tweet` and `generate_thread` can accept optional RAG context (check if `generate_reply_with_context` pattern can be generalized)

**Implementation:**
1. In each assist handler, after getting the `ContentGenerator`, call `winning_dna::build_draft_context(&state.db, &topic_keywords, 5, 14.0).await`
2. If the returned `DraftContext.prompt_block` is non-empty, pass it as RAG context to the generator
3. **Pattern to follow:** `ContentGenerator::generate_reply_with_context()` already accepts `rag_context: Option<&str>` and injects it between persona and rules sections. Add analogous methods `generate_tweet_with_context()` and `generate_thread_with_context()` that accept the same parameter and insert the RAG block into the system prompt, mirroring the reply pattern at `generator.rs:77-93`

**Risk:** `build_draft_context` reads from the DB, which assist handlers already access (via `state.db`). The function is async and returns `Result<DraftContext, StorageError>` with graceful degradation — empty prompt_block when no history exists.

**Test impact:** Existing contract tests don't test assist endpoints (they test compose endpoints). Add 1-2 unit tests for the new `_with_context` methods using the existing `MockProvider` pattern in `generator.rs:551-599`.

#### Task 2.4: Enhance notes-to-content flow

**File to modify:**
- `FromNotesPanel.svelte` → move to `composer/NotesToContentPanel.svelte` (~200 lines)

**Enhancements:**
1. Replace `confirm()` dialog with inline "Replace current content?" banner with Replace/Cancel buttons
2. Add loading skeleton during generation (replace textarea with shimmer animation)
3. Add undo: snapshot current content before replacement, show "Undo" button for 10 seconds after generation completes
4. Thread generation: preserve source notes in a collapsed section below the generated content so the user can reference them

#### Task 2.5: Update docs and tests

- Update `docs/composer-mode.md` with VoiceContextBar, enhanced notes panel
- If assist request shape changes (unlikely given D4), update contract tests
- Write `docs/roadmap/typefully-beating-composer/session-02-handoff.md`

### Session 02 file manifest

| Action | File |
|--------|------|
| Create | `dashboard/src/lib/components/composer/ComposerShell.svelte` |
| Create | `dashboard/src/lib/components/composer/TweetEditor.svelte` |
| Create | `dashboard/src/lib/components/composer/ThreadEditor.svelte` |
| Create | `dashboard/src/lib/components/composer/ThreadCardActions.svelte` |
| Create | `dashboard/src/lib/components/composer/VoiceContextBar.svelte` |
| Create | `dashboard/src/lib/components/composer/NotesToContentPanel.svelte` |
| Modify | `dashboard/src/lib/components/ComposeModal.svelte` |
| Modify | `dashboard/src/lib/components/ThreadComposer.svelte` (or remove) |
| Modify | `dashboard/src/lib/components/FromNotesPanel.svelte` (or remove) |
| Modify | `crates/tuitbot-server/src/routes/assist.rs` |
| Modify | `crates/tuitbot-core/src/content/generator.rs` |
| Modify | `docs/composer-mode.md` |
| Create | `docs/roadmap/typefully-beating-composer/session-02-handoff.md` |

### Session 02 quality gates

```bash
cd dashboard && npm run check
cd dashboard && npm run build
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

### Session 02 fallback scope cuts (in priority order)

1. **Cut first:** localStorage cue persistence (keep quick-cue input, skip saved cues dropdown)
2. **Cut second:** Notes panel undo feature (keep replacement confirmation, skip snapshot/undo)
3. **Cut third:** Source notes preservation in thread mode (just generate, don't keep notes reference)

After cuts 1-3, the minimum viable Session 02 is: refactored components + winning DNA wiring + voice context bar (read-only voice summary + ephemeral cue input) + improved notes confirmation.

---

## Session 03: X-Accurate Preview

### Mission

Replace the generic preview rendering with X-accurate layout rules for media grids, add tweet-mode preview, and ensure preview stays synchronized with all thread operations.

### Task breakdown

#### Task 3.1: Extract preview into focused components

**New files:**
```
dashboard/src/lib/components/composer/PreviewRail.svelte         — Preview container for both tweet and thread modes (~150 lines)
dashboard/src/lib/components/composer/TweetPreviewCard.svelte    — Single tweet preview with X-accurate layout (~200 lines)
dashboard/src/lib/components/composer/MediaGrid.svelte           — X-style media grid with 1/2/3/4 layouts (~200 lines)
```

**Files to modify:**
- `TweetPreview.svelte` — Replaced by `TweetPreviewCard.svelte`, original removed or reduced to re-export
- `ComposeModal.svelte` (post-refactor) — Import `PreviewRail` instead of inline preview markup

#### Task 3.2: Implement X-accurate media grid

**`MediaGrid.svelte` layout rules:**

| Count | Grid | Aspect ratios | CSS grid template |
|-------|------|---------------|-------------------|
| 1 | Full width | 16:9 | `1fr` / `1fr` |
| 2 | Side by side | ~1:1 each | `1fr 1fr` / `1fr` |
| 3 | Large left + 2 stacked right | Left ~2:3, right ~1:1 each | `1fr 1fr` / `1fr 1fr`, first-child spans row 1/3 |
| 4 | 2x2 grid | ~1:1 each | `1fr 1fr` / `1fr 1fr` |

**Props:** `mediaPaths: string[]`, `intrinsicDimensions?: Map<string, {w: number, h: number}>`

**Crop indicator:** When an image's intrinsic aspect ratio differs significantly from the display slot (> 30% deviation), show a subtle indicator (e.g., dashed border overlay or small "cropped" badge in corner).

#### Task 3.3: Add tweet-mode preview

Currently, tweet mode shows only a textarea + char counter. Add a preview alongside:
- Desktop: side-by-side layout (editor left, preview right) — same pattern as thread mode
- Mobile: stacked vertically (editor top, preview bottom)

The preview uses `TweetPreviewCard` with the current tweet text and attached media.

#### Task 3.4: Video poster frame

For video media in the grid:
- Render `<video>` element with `preload="metadata"` to get first frame
- Overlay a centered play icon (CSS pseudo-element or SVG)
- No playback controls in preview — just visual representation

#### Task 3.5: Keep preview synchronized

Verify that all thread operations update the preview reactively:
- Text editing: immediate (reactive binding)
- Reorder (drag-drop, Alt+Arrow): immediate (blocks array update triggers re-render)
- Split/Merge: immediate (new blocks trigger re-render)
- Delete: immediate (filtered blocks)
- Recovery: immediate (recovered blocks replace state)
- Media attach/remove: immediate (media_paths update triggers re-render)

No special sync logic should be needed if the preview components are purely derived from the blocks state.

#### Task 3.6: Document fidelity rules and update docs

- Add a "Preview Fidelity" section to `docs/composer-mode.md` documenting what the preview emulates and known limitations
- Write `docs/roadmap/typefully-beating-composer/session-03-handoff.md`

### Session 03 file manifest

| Action | File |
|--------|------|
| Create | `dashboard/src/lib/components/composer/PreviewRail.svelte` |
| Create | `dashboard/src/lib/components/composer/TweetPreviewCard.svelte` |
| Create | `dashboard/src/lib/components/composer/MediaGrid.svelte` |
| Modify | `dashboard/src/lib/components/TweetPreview.svelte` (remove or alias) |
| Modify | `dashboard/src/lib/components/ComposeModal.svelte` (import PreviewRail) |
| Modify | `docs/composer-mode.md` |
| Create | `docs/roadmap/typefully-beating-composer/session-03-handoff.md` |

### Session 03 quality gates

```bash
cd dashboard && npm run check
cd dashboard && npm run build
```

No Rust changes expected in Session 03.

### Session 03 fallback scope cuts (in priority order)

1. **Cut first:** Crop indicator overlay (keep standard object-fit: cover)
2. **Cut second:** Video poster frame (show generic video placeholder icon)
3. **Cut third:** Intrinsic dimension detection (use fixed aspect ratios only)

After all cuts, the minimum viable Session 03 is: X-accurate media grid CSS + tweet-mode preview pane + component extraction.

---

## Session 04: Validation & Go/No-Go

### Mission

Validate the completed overhaul, fix ship-blocking regressions, update docs to match delivered behavior, and produce a release-readiness report.

### Task breakdown

#### Task 4.1: Full CI verification

Run all required checks:
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
cd dashboard && npm run build
```

Fix any failures. Only fix what blocks release — do not refactor or improve unrelated code.

#### Task 4.2: Manual audit checklist

| Flow | What to verify |
|------|----------------|
| Focus mode | Enters/exits cleanly, keyboard shortcuts work, preview visible |
| Notes → tweet | Loading state, confirmation before replacement, undo (if shipped) |
| Notes → thread | Loading state, generates multi-card thread, source notes (if shipped) |
| Voice context bar | Shows voice settings, quick cue modifies AI output |
| Thread restructuring | Reorder (drag + keyboard), split, merge, duplicate, delete |
| Autosave + recovery | Close without saving, reopen, recover, discard |
| Preview fidelity | 1/2/3/4 image grids match X layout, tweet-mode preview works |
| Media in threads | Per-card media, preview updates on attach/remove |
| Mobile layout | Stacked preview, touch targets, font sizes |
| Approval mode | Content routes to approval queue, not posted directly |
| Scheduling | Time picker works, scheduled_for in payload |
| Backward compat | Legacy compose payloads still accepted |

#### Task 4.3: Update documentation

- `docs/composer-mode.md` — Update to reflect all shipped features, remove references to features that were cut
- Ensure keyboard shortcut table is accurate
- Document VoiceContextBar usage
- Document preview fidelity rules and known limitations

#### Task 4.4: Write release-readiness report

Create `docs/roadmap/typefully-beating-composer/release-readiness.md`:
- Explicit GO or NO-GO decision
- Evidence for each differentiator (what shipped, what was cut)
- Known limitations
- Follow-up items for future sessions
- File size audit (all Svelte files under 400 lines, all Rust files under 500 lines)

### Session 04 file manifest

| Action | File |
|--------|------|
| Modify | `docs/composer-mode.md` |
| Create | `docs/roadmap/typefully-beating-composer/release-readiness.md` |
| Create | `docs/roadmap/typefully-beating-composer/session-04-handoff.md` |
| Fix | Various files (targeted regression fixes only) |

### Session 04 quality gates

All CI checks must pass. All audit checklist items must be verified or documented as known limitations.

---

## Architecture Summary

### Component hierarchy (post-refactor)

```
ComposeModal.svelte (orchestrator, ~350 lines)
├── ComposerShell.svelte (modal chrome, focus mode, ~250 lines)
│   ├── VoiceContextBar.svelte (voice summary + cues, ~120 lines)
│   ├── TweetEditor.svelte (tweet mode editor, ~150 lines)
│   │   └── PreviewRail.svelte (tweet preview, ~150 lines)
│   │       └── TweetPreviewCard.svelte (single tweet, ~200 lines)
│   │           └── MediaGrid.svelte (X-style grid, ~200 lines)
│   ├── ThreadEditor.svelte (thread mode editor, ~350 lines)
│   │   ├── ThreadCardActions.svelte (per-card actions, ~100 lines)
│   │   ├── MediaSlot.svelte (per-card media, existing 293 lines)
│   │   └── PreviewRail.svelte (thread preview, reused)
│   └── NotesToContentPanel.svelte (enhanced notes, ~200 lines)
└── CommandPalette.svelte (existing, unchanged)
    TimePicker.svelte (existing, unchanged)
```

### Data flow: Voice context in assist requests

```
VoiceContextBar                    ComposeModal
  ├─ reads settings ──────────────── fetches /api/settings on open
  ├─ displays brand_voice, style ─── read-only display
  └─ quick-cue input ─────────────── passes cue to assist calls
                                         │
                                         ▼
                              /api/assist/improve
                                { draft, context: cue }
                                         │
                                         ▼
                              assist_improve handler
                                ├─ reads cue from context
                                ├─ calls winning_dna::build_draft_context()
                                └─ passes both to ContentGenerator
                                         │
                                         ▼
                              ContentGenerator.generate_tweet()
                                ├─ brand_voice from config
                                ├─ content_style from config
                                ├─ persona context from config
                                ├─ user cue from request
                                └─ winning DNA RAG context
```

### Backward compatibility contract

The `ComposeRequest` type and all `/api/content/compose` behavior are unchanged:

```typescript
interface ComposeRequest {
  content_type: string;  // "tweet" | "thread" — unchanged
  content: string;       // text or JSON array — unchanged
  scheduled_for?: string; // ISO 8601 — unchanged
  media_paths?: string[]; // server paths — unchanged
  blocks?: ThreadBlock[]; // structured blocks — unchanged
}
```

All 20 existing compose contract tests must pass without modification.
