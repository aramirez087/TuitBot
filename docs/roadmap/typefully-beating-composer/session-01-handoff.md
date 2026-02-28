# Session 01 Handoff

## What changed

Three documents created (no code changes):

1. **`charter.md`** — Project charter defining the two differentiators, v1 voice learning definition, v1 preview fidelity rules, hard constraints, and non-goals.
2. **`implementation-plan.md`** — Session-by-session breakdown (02/03/04) with task lists, file manifests, quality gates, fallback scope cuts, and architecture diagrams.
3. **`session-01-handoff.md`** — This document.

## Key decisions

### D1: V1 voice learning = transparent context threading

"Learns your voice" means the AI's knowledge is visible (voice context bar), steerable (quick-cue input), and historical (winning DNA RAG injection). It does NOT mean implicit model training, auto-updating config fields, or style fingerprinting.

### D2: V1 preview fidelity = X media grids + tweet-mode preview

Preview emulates X's 1/2/3/4 image grid layouts with crop indicators. Tweet mode gets its own preview pane. Link cards, GIF animation, and theme switching are deferred.

### D3: Refactor into composer/ directory

`ComposeModal.svelte` (1,273 lines) splits into 6 subcomponents in `dashboard/src/lib/components/composer/`. `ThreadComposer.svelte` (858 lines) splits into `ThreadEditor.svelte` + `ThreadCardActions.svelte`. All files stay under 400 lines.

### D4: No compose contract changes

`ComposeRequest` interface and all 20 existing contract tests remain unchanged. Voice cues use the existing `context` parameter on `/api/assist/improve`. Winning DNA wiring is server-side and transparent to clients.

## Open questions

None blocking Session 02. All architectural decisions are made.

## What remains

| Session | Scope |
|---------|-------|
| 02 | Component refactor + voice context bar + winning DNA wiring + notes enhancement |
| 03 | X-accurate media grids + tweet-mode preview + preview sync |
| 04 | Full validation, regression fixes, doc updates, go/no-go report |

## Session 02 must read first

### Roadmap documents
- `docs/roadmap/typefully-beating-composer/charter.md`
- `docs/roadmap/typefully-beating-composer/implementation-plan.md`
- `docs/roadmap/typefully-beating-composer/session-01-handoff.md`

### Dashboard (to refactor)
- `dashboard/src/lib/components/ComposeModal.svelte` — 1,273 lines, primary refactoring target
- `dashboard/src/lib/components/ThreadComposer.svelte` — 858 lines, secondary refactoring target
- `dashboard/src/lib/components/FromNotesPanel.svelte` — enhance into NotesToContentPanel
- `dashboard/src/lib/components/MediaSlot.svelte` — per-card media, used by ThreadEditor
- `dashboard/src/lib/components/TweetPreview.svelte` — context for preview pane
- `dashboard/src/routes/(app)/settings/ContentPersonaSection.svelte` — voice field names for VoiceContextBar

### API layer and stores
- `dashboard/src/lib/api.ts` — `ComposeRequest`, assist methods, settings fetch
- `dashboard/src/lib/stores/settings.ts` — writable `config` store holds `TuitbotConfig` including `business.brand_voice`, `business.content_style`, `business.content_pillars`

### Rust (voice + winning DNA wiring)
- `crates/tuitbot-core/src/content/generator.rs` — where voice/persona/RAG context is injected into LLM prompts
- `crates/tuitbot-core/src/context/winning_dna.rs` — `build_draft_context()` to wire into assist endpoints
- `crates/tuitbot-core/src/config/types.rs` — `BusinessProfile` struct with voice fields
- `crates/tuitbot-server/src/routes/assist.rs` — assist endpoints to add winning DNA context
- `crates/tuitbot-server/tests/compose_contract_tests.rs` — verify no regressions

### Reference
- `docs/composer-mode.md` — current documented behavior to preserve
- `CLAUDE.md` — architecture rules and CI checklist
