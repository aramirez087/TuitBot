# Session 03 Handoff — Server RAG Context Resolver

## What Changed

### Code Changes

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/routes/assist.rs` | Added `resolve_composer_rag_context()` async helper + `use tuitbot_core::context::winning_dna` import + 3 unit tests |

### Documentation

| File | Description |
|------|-------------|
| `docs/roadmap/composer-auto-vault-context/server-rag-contract.md` | Helper contract, fail-open behavior, logging, integration points |
| `docs/roadmap/composer-auto-vault-context/session-03-handoff.md` | This file |

## Key Decisions

1. **Module-private function in `assist.rs`.** The resolver is a private `async fn` co-located with the handlers that will call it, mirroring the `get_generator()` pattern. No `pub` visibility needed — all consumers are in the same file.

2. **`_account_id` parameter reserved but unused.** Config is loaded from `state.config_path` (single-account pattern matching `get_mode`). The parameter exists for forward-compatibility with multi-account config loading without changing the call sites when that lands.

3. **Early return on empty keywords.** When `draft_context_keywords()` returns an empty vec (no product/competitor/industry keywords configured), the resolver returns `None` immediately, avoiding a pointless DB query. This is silent (no log) because it's an expected cold-start state.

4. **Config re-read per call.** Each invocation reads `config.toml` from disk. This matches the existing `get_mode` pattern. Sub-millisecond cost is negligible for interactive composer usage (<1 req/s).

5. **Constants from `winning_dna` module.** Uses `winning_dna::MAX_ANCESTORS` (5) and `winning_dna::RECENCY_HALF_LIFE_DAYS` (14.0) directly — no new magic numbers. Same constants used by the autopilot draft workflow.

6. **Three tests validate fail-open contract.** Config-missing, empty-DB, and no-keywords paths each confirmed to return `None`. Tests construct a minimal `AppState` using `init_test_db()` and `tempfile`, following the pattern in `tests/api_tests.rs`.

## Open Risks

1. **`test_state()` is coupled to `AppState` field count.** If upstream changes add fields, the test won't compile. This is acceptable — compile-time breakage is a canary, not a risk. If it becomes too fragile, extract into a shared test helper.

2. **No handler wiring yet.** The resolver exists but no handler calls it. It's dead code until Session 04. This is intentional — the exit criteria require the resolver without endpoint duplication. Clippy's `dead_code` warning is suppressed by the `#[cfg(test)]` usage in tests.

3. **`assist_tweet` and `assist_thread` lack `_with_context` variants.** Only `improve_draft_with_context` exists (Session 02). Session 04 must add `generate_tweet_with_context` and `generate_thread_with_context` in `tuitbot-core` before wiring can complete.

## Exact Inputs for Session 04

### Files to Read Before Starting

| File | Why |
|------|-----|
| `docs/roadmap/composer-auto-vault-context/server-rag-contract.md` | Resolver contract and integration pattern |
| `docs/roadmap/composer-auto-vault-context/core-contract.md` | `improve_draft_with_context` signature (Session 02) |
| `crates/tuitbot-server/src/routes/assist.rs` | Resolver implementation + handler call sites |
| `crates/tuitbot-core/src/content/generator/mod.rs` | `generate_tweet`, `generate_thread` — need `_with_context` variants |

### Session 04 Tasks

1. **Add `generate_tweet_with_context(topic, rag_context) -> Result<GenerationOutput, LlmError>`** in `generator/mod.rs`, following the `_inner` delegation pattern from `improve_draft_with_context`.

2. **Add `generate_thread_with_context(topic, rag_context) -> Result<ThreadOutput, LlmError>`** in `generator/mod.rs`, same pattern.

3. **Wire all three handlers** in `assist.rs`:
   - `assist_tweet`: call `resolve_composer_rag_context`, pass to `generate_tweet_with_context`
   - `assist_thread`: call `resolve_composer_rag_context`, pass to `generate_thread_with_context`
   - `assist_improve`: call `resolve_composer_rag_context`, pass to `improve_draft_with_context`

4. **Add tests** in `generator/tests.rs` for the new `_with_context` variants (prompt capture assertions).

### Quality Gates

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
```
