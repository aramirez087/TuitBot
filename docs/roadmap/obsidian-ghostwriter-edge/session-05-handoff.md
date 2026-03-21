# Session 05 Handoff: Hook Generation Engine

## What Changed

Backend now supports generating 5 differentiated hook options from Ghostwriter context via `POST /api/assist/hooks`. The endpoint accepts topic + optional selection session ID or node IDs, resolves RAG context through the existing pipeline, and returns styled hook options with metadata for downstream UI ranking.

### Files Modified

| File | Change |
|---|---|
| `crates/tuitbot-core/src/content/generator/mod.rs` | Added `HookOption`, `HookGenerationOutput` types; `generate_hooks()`, `select_hook_styles()`, `build_hook_options()` methods on `ContentGenerator` |
| `crates/tuitbot-core/src/content/generator/parser.rs` | Added `parse_hooks_response()` function |
| `crates/tuitbot-core/src/content/generator/tests.rs` | Added 15 tests for hook generation, parsing, serialization, and confidence heuristics |
| `crates/tuitbot-server/src/routes/rag_helpers.rs` | Added `resolve_selection_rag_context()` and `SelectionRagContext` for Ghostwriter selection → RAG resolution |
| `crates/tuitbot-server/src/lib.rs` | Registered `POST /assist/hooks` route |
| `dashboard/src/lib/api/types.ts` | Added `HookOption` and `AssistHooksResponse` interfaces |
| `dashboard/src/lib/api/client.ts` | Added `api.assist.hooks()` method |

### Files Created

| File | Purpose |
|---|---|
| `crates/tuitbot-server/src/routes/assist/hooks.rs` | Hook generation endpoint handler + DTOs + 4 server-layer tests |
| `docs/roadmap/obsidian-ghostwriter-edge/hook-generation-contract.md` | Decision log, wire format, parser contract, privacy analysis |
| `docs/roadmap/obsidian-ghostwriter-edge/session-05-handoff.md` | This file |

### Structural Changes

| Change | Reason |
|---|---|
| `assist.rs` → `assist/mod.rs` + `assist/hooks.rs` | `assist.rs` was at 508 lines (over 500-line limit). Module directory split follows existing patterns (`vault/`, etc.) |

## Decisions Made

See `hook-generation-contract.md` for full decision log (6 decisions).

Key decisions:
1. **Reuse TweetFormat taxonomy** — hooks use the same 7-variant `TweetFormat` enum as standalone tweets; no new style system
2. **Single LLM call** — all 5 hooks generated in one structured prompt call (not 5 parallel calls); cheaper and naturally differentiated
3. **Always include Question + ContrarianTake** — these two formats are always selected as they have the strongest hook energy; remaining 3 are randomized
4. **Selection context via existing RAG pipeline** — new `resolve_selection_rag_context` helper resolves `session_id` → selection → node IDs → `build_draft_context_with_selection`; raw `selected_text` is injected as additional prompt context when no indexed node exists
5. **Confidence heuristic** — `"high"` if hook is <= 240 chars, `"medium"` otherwise; intentionally simple, no engagement prediction
6. **Module split for assist routes** — `assist.rs` (508 lines) → `assist/mod.rs` + `assist/hooks.rs` to stay under the 500-line file limit

## Exit Criteria Met

- [x] `POST /api/assist/hooks` returns 3-5 differentiated hook options
- [x] Each hook tagged with `style`, `text`, `char_count`, `confidence`
- [x] `session_id` parameter resolves Ghostwriter selection context
- [x] `selected_node_ids` parameter resolves indexed vault context
- [x] Falls back gracefully when no context is available
- [x] Parser handles edge cases: missing STYLE, empty hooks, trailing separator
- [x] Retry logic when fewer than 3 hooks returned
- [x] Frontend types and API client method in place
- [x] 19 new tests (15 core + 4 server) all pass
- [x] `cargo fmt --all --check` passes
- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] `RUSTFLAGS="-D warnings" cargo test --workspace` passes (567 tests)
- [x] `npm --prefix dashboard run check` passes

## What Session 6 Needs

1. **Hook picker UI**: Wire `FromVaultPanel` "Generate from selection" button to `api.assist.hooks()`. Build a hook picker component showing 5 styled options with confidence indicators. User selects one and it flows into the compose workspace.
2. **Style display names**: The `style` field returns raw format names like `contrarian_take`. Session 6 should add human-readable labels (e.g., "Hot Take") in the UI layer.
3. **Hook → compose flow**: Selected hook should populate the compose workspace as a tweet draft (or thread opener if the user switches to thread mode).
4. **Regenerate individual hook**: Consider allowing regeneration of a single hook style if the user likes the style but not the specific text.
5. **ProvenanceRef pipeline integration** (carried from Session 4): Ensure ProvenanceRef is properly attached through the inspector → workspace → API chain when composing from a hook.
6. **Cleanup task wiring** (carried from Session 4): `vault_selections::cleanup_expired()` should be wired into the server's hourly cleanup loop.

## Open Risks

1. **LLM output parsing fragility**: The `STYLE: / HOOK:` format depends on LLM compliance. The parser is lenient (falls back to "general" style, skips empty hooks), and retry logic handles under-production, but adversarial or very different LLM backends may need format tuning.
2. **Style randomization means non-deterministic output**: The 3 random styles change per request. This is intentional (variety) but means the same topic may produce different style mixes. If users expect consistency, a future iteration could accept preferred styles as input.
3. **Selection TTL vs. hook generation time** (carried from Session 4): If a user takes >30 minutes after selection before generating hooks, the selection is expired server-side. The frontend mitigates by fetching immediately, but `session_id` resolution will return None.
