# Automatic Vault Context — Implementation Plan

## Session Map

### Session 01: Charter and Architecture Audit (this session)

**Scope:** Documentation only. Audit the composer assist stack, trace the RAG context gap, produce charter and implementation plan.

**Deliverables:**
- `docs/roadmap/composer-auto-vault-context/charter.md`
- `docs/roadmap/composer-auto-vault-context/implementation-plan.md`
- `docs/roadmap/composer-auto-vault-context/session-01-handoff.md`

**Validation:** File existence, decision traceability to codebase lines.

---

### Session 02: Core Contract Changes

**Scope:** Add shared keyword helper and `improve_draft_with_context()` to `tuitbot-core`. Refactor `draft.rs` to use the shared helper.

**Key changes:**
1. Add `BusinessProfile::draft_context_keywords() -> Vec<String>` in `config/types.rs` (line ~127+). This method returns the merged keyword set: `product_keywords + competitor_keywords + effective_industry_topics()`.
2. Add `ContentGenerator::improve_draft_with_context(draft, tone_cue, rag_context)` in `content/generator/mod.rs` (line ~275+). Delegates to an `improve_draft_inner()` that accepts optional RAG context, injected via `format_rag_section()`.
3. Refactor `workflow/draft.rs:52-54` to call `config.business.draft_context_keywords()` instead of manually assembling the keyword vector.

**Files modified:**
- `crates/tuitbot-core/src/config/types.rs` — add `draft_context_keywords()`
- `crates/tuitbot-core/src/content/generator/mod.rs` — add `improve_draft_with_context()` + `improve_draft_inner()`
- `crates/tuitbot-core/src/workflow/draft.rs` — refactor to use `draft_context_keywords()`

**Validation:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test -p tuitbot-core
cargo clippy --workspace -- -D warnings
```

**Tests to add:**
- Unit test for `BusinessProfile::draft_context_keywords()` covering empty/non-empty keyword combinations.
- Unit test in `content/generator/tests.rs` for `improve_draft_with_context()` verifying RAG section appears in the prompt.

---

### Session 03: Server-Side Resolver

**Scope:** Implement `resolve_composer_rag_context()` in `routes/assist.rs`. No handler wiring yet — just the helper function with its own unit/integration test.

**Key changes:**
1. Add `resolve_composer_rag_context(state, account_id) -> Option<String>` as an async helper in `assist.rs`.
2. Uses `Config::load()` from `state.config_path`, `config.business.draft_context_keywords()`, and `winning_dna::build_draft_context()`.
3. Catches all errors with `warn` logging, returns `None`.

**Files modified:**
- `crates/tuitbot-server/src/routes/assist.rs` — add resolver helper

**Validation:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test -p tuitbot-server
cargo clippy --workspace -- -D warnings
```

**Tests to add:**
- Fail-open assertion: resolver returns `None` when config is missing or DB has no data.

---

### Session 04: Wire Tweet and Thread Handlers

**Scope:** Update `assist_tweet` and `assist_thread` to call the resolver and pass RAG context to `_with_context` generator variants.

**Key changes:**
1. `assist_tweet` calls `resolve_composer_rag_context()`, then `gen.generate_tweet_with_context(&topic, None, rag_context.as_deref())`.
2. `assist_thread` calls `resolve_composer_rag_context()`, then `gen.generate_thread_with_context(&topic, None, rag_context.as_deref())`.

**Files modified:**
- `crates/tuitbot-server/src/routes/assist.rs` — update `assist_tweet`, `assist_thread`

**Validation:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

### Session 05: Wire Improve Handler

**Scope:** Update `assist_improve` to call the resolver and pass both vault context and user-supplied tone cue.

**Key changes:**
1. `assist_improve` calls `resolve_composer_rag_context()`, then `gen.improve_draft_with_context(&draft, tone_cue, rag_context.as_deref())`.

**Files modified:**
- `crates/tuitbot-server/src/routes/assist.rs` — update `assist_improve`

**Validation:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

### Session 06: Regression Tests

**Scope:** Full server-level test harness exercising all four assist endpoints with mock LLM and populated/empty vault data.

**Key changes:**
1. Add integration tests verifying RAG context is passed when vault data exists.
2. Add integration tests verifying fail-open when vault data is empty.
3. Add test verifying `improve` endpoint correctly combines vault context and user tone cue.

**Files modified:**
- `crates/tuitbot-server/src/routes/assist.rs` (or a dedicated test module)

**Validation:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

### Session 07: Documentation Update

**Scope:** Update `docs/composer-mode.md` to describe automatic vault context. Add release notes.

**Files modified:**
- `docs/composer-mode.md` — add vault context section

**Validation:** Documentation review, no code changes.

---

### Session 08: Final Validation

**Scope:** Full CI pass, go/no-go decision.

**Validation:**
```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Risk Register

### R1: Config Read Failure

**Risk:** `Config::load()` fails because `config_path` is inaccessible.
**Likelihood:** Low — server startup already reads config successfully.
**Mitigation:** Resolver catches errors, logs at `warn`, returns `None`. Generation proceeds without RAG.

### R2: Empty Vault (New Accounts)

**Risk:** New users with no posting history and no ingested notes get `None` context.
**Likelihood:** High for new accounts.
**Impact:** None — generation quality is unchanged from current behavior. The feature is additive.
**Mitigation:** `build_draft_context()` already handles empty data gracefully (returns empty `prompt_block`).

### R3: Prompt Length Overflow

**Risk:** RAG block combined with existing system prompt sections exceeds LLM context window.
**Likelihood:** Very low — RAG block is capped at `RAG_MAX_CHARS = 2000` chars (~500 tokens). Total system prompts are well under 4K tokens.
**Mitigation:** Character cap in `format_ancestors_prompt()` and `format_seeds_prompt()`.

### R4: Keyword Assembly Duplication

**Risk:** Keywords are assembled in both `draft.rs` and the new resolver, diverging over time.
**Likelihood:** Medium if not addressed.
**Mitigation:** Session 02 adds `BusinessProfile::draft_context_keywords()` as single source of truth, then both call sites use it.

### R5: Latency Increase for Assist Calls

**Risk:** Additional DB query + config file read adds latency to interactive composer requests.
**Likelihood:** Certain — but magnitude is <15ms total.
**Mitigation:** SQLite WAL reads are fast. Config file reads are sub-millisecond. Both are bounded. No user-perceptible impact.

### R6: `improve_draft_with_context()` Prompt Quality

**Risk:** Injecting RAG context into the improve prompt dilutes the sharpening objective.
**Likelihood:** Low — RAG section is placed in the system prompt header, not in the task instructions.
**Mitigation:** RAG section uses the same `format_rag_section()` as tweet/thread. The "improve" task instruction remains dominant. If quality degrades, the RAG section can be adjusted without API changes.

## Validation Strategy

Each session runs the CI checklist before handoff:

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Session 06 adds integration tests specific to the feature. Session 08 performs a final go/no-go pass.
