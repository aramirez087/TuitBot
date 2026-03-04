# Session 01 Handoff — Charter and Architecture Audit

## What Changed

Three documentation artifacts created under `docs/roadmap/composer-auto-vault-context/`:

1. **`charter.md`** — Problem statement, target architecture, design decisions (helper placement, keyword sourcing, dual-context improve, fail-open behavior, shape preservation), and explicit non-goals.
2. **`implementation-plan.md`** — Eight-session roadmap with per-session scope, file changes, validation commands, and a risk register covering six identified risks.
3. **`session-01-handoff.md`** — This file.

No source code was created or modified.

## Key Audit Findings

1. All four composer assist handlers (`assist_tweet`, `assist_reply`, `assist_thread`, `assist_improve`) call `ContentGenerator` methods without RAG context.
2. Three of four generator methods already have `_with_context` variants: `generate_tweet_with_context`, `generate_reply_with_context`, `generate_thread_with_context`. Only `improve_draft` lacks one.
3. The autopilot draft workflow (`workflow/draft.rs:51-68`) demonstrates the correct RAG wiring pattern: assemble keywords from `config.business`, call `build_draft_context()`, extract `prompt_block`, pass to `generate_reply_with_context()`.
4. `ContentGenerator` is intentionally DB-unaware (Toolkit layer). RAG resolution must happen at the server or workflow layer.
5. `AppState` (state.rs) has both `db: DbPool` and `config_path: PathBuf`, so the server layer can resolve RAG context.
6. `get_mode()` at `assist.rs:252` already demonstrates the pattern of loading `Config` from `state.config_path` inside a handler.

## Open Risks

1. **Keyword assembly duplication.** `draft.rs:52-54` manually assembles `product_keywords + competitor_keywords + effective_industry_topics()`. The new resolver will need the same assembly. Session 02 must add `BusinessProfile::draft_context_keywords()` to eliminate this duplication.
2. **Missing `improve_draft_with_context()`.** The `ContentGenerator` has no `_with_context` variant for `improve_draft`. Session 02 must add it following the same `_inner` delegation pattern used by `generate_tweet_inner`, `generate_reply_inner`, and `generate_thread_inner`.
3. **No integration tests for assist endpoints.** The current test suite doesn't test the assist handlers at the server level. Session 06 must add these.

## Exact Inputs for Session 02

### Files to Read Before Starting

| File | Why |
|------|-----|
| `docs/roadmap/composer-auto-vault-context/charter.md` | Architecture decisions |
| `docs/roadmap/composer-auto-vault-context/implementation-plan.md` | Session 02 scope |
| `crates/tuitbot-core/src/config/types.rs` | Add `draft_context_keywords()` near line 127 |
| `crates/tuitbot-core/src/content/generator/mod.rs` | Add `improve_draft_with_context()` near line 275 |
| `crates/tuitbot-core/src/workflow/draft.rs` | Refactor keyword assembly at lines 52-54 |

### Session 02 Tasks

1. **Add `BusinessProfile::draft_context_keywords() -> Vec<String>`** in `config/types.rs`.
   - Returns: `product_keywords` + `competitor_keywords` + `effective_industry_topics()`, cloned into a single `Vec<String>`.
   - Add unit test covering: empty profile, profile with only product_keywords, profile with all keyword types.

2. **Add `ContentGenerator::improve_draft_with_context(draft, tone_cue, rag_context)`** in `content/generator/mod.rs`.
   - Create `improve_draft_inner(draft, tone_cue, rag_context)` that contains the current `improve_draft` logic plus `rag_section` injection.
   - Refactor existing `improve_draft()` to delegate to `improve_draft_inner(draft, tone_cue, None)`.
   - RAG context is injected via `Self::format_rag_section(rag_context)` into the system prompt, between `{persona_section}` and the task block.
   - Add test verifying the RAG section appears in the prompt when `rag_context` is `Some`.

3. **Refactor `workflow/draft.rs:52-54`** to call `config.business.draft_context_keywords()`.
   - Replace the three-line keyword assembly with a single call.
   - Verify `cargo test -p tuitbot-core` still passes.

### Quality Gates

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test -p tuitbot-core
cargo clippy --workspace -- -D warnings
```

### Session 02 Deliverable

- `docs/roadmap/composer-auto-vault-context/session-02-handoff.md`
