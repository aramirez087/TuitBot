# Composer Auto-Vault Context — Test Matrix

Comprehensive coverage map for the automatic vault context feature across all three composer assist endpoints.

## HTTP-Level Integration Tests (Session 06)

File: `crates/tuitbot-server/tests/assist_rag_tests.rs`

| # | Test Name | Endpoint | Generator | Vault Data | body.context | Assertion |
|---|-----------|----------|-----------|------------|--------------|-----------|
| 1 | `tweet_with_rag_context` | POST /api/assist/tweet | Mock | Seeds (cold-start) | — | 200 OK, `content` present, prompt contains `"Relevant ideas"` |
| 2 | `tweet_without_rag_context` | POST /api/assist/tweet | Mock | Empty DB | — | 200 OK, `content` present, prompt does NOT contain vault markers |
| 3 | `tweet_no_generator_returns_400` | POST /api/assist/tweet | None | — | — | 400 Bad Request, error mentions LLM config |
| 4 | `thread_with_rag_context` | POST /api/assist/thread | Mock | Seeds (cold-start) | — | 200 OK, `tweets` array ≥5, prompt contains `"Relevant ideas"` |
| 5 | `thread_without_rag_context` | POST /api/assist/thread | Mock | Empty DB | — | 200 OK, `tweets` array ≥5, no vault markers |
| 6 | `thread_no_generator_returns_400` | POST /api/assist/thread | None | — | — | 400 Bad Request |
| 7 | `improve_with_rag_context` | POST /api/assist/improve | Mock | Seeds (cold-start) | None | 200 OK, `content` present, prompt contains `"Relevant ideas"` |
| 8 | `improve_without_rag_context` | POST /api/assist/improve | Mock | Empty DB | None | 200 OK, `content` present, no vault markers |
| 9 | `improve_no_generator_returns_400` | POST /api/assist/improve | None | — | — | 400 Bad Request |
| 10 | `improve_dual_context` | POST /api/assist/improve | Mock | Seeds (cold-start) | `"Be punchy"` | 200 OK, prompt contains both `"Relevant ideas"` AND `"Be punchy"` |
| 11 | `improve_tone_only_no_vault` | POST /api/assist/improve | Mock | Empty DB | `"Be casual"` | 200 OK, prompt contains `"Be casual"`, no vault markers |
| 12 | `tweet_with_ancestors_context` | POST /api/assist/tweet | Mock | Ancestors (perf data) | — | 200 OK, prompt contains `"Winning patterns"` |

## Layer Coverage Map

| Layer | Tests | Session | File |
|-------|-------|---------|------|
| **Generator (unit)** | `generate_tweet_with_context_*`, `generate_thread_with_context_*`, `improve_draft_with_context_*`, prompt capture tests | 02 | `crates/tuitbot-core/src/content/generator/tests.rs` |
| **Resolver (unit)** | `resolve_rag_returns_none_when_config_missing`, `resolve_rag_returns_none_when_db_empty`, `resolve_rag_returns_none_when_no_keywords` | 03 | `crates/tuitbot-server/src/routes/assist.rs` (inline `#[cfg(test)]`) |
| **Winning DNA (unit + DB)** | `build_draft_context_*`, `format_ancestors_prompt_*`, `format_seeds_*`, scoring tests | 01 | `crates/tuitbot-core/src/context/winning_dna.rs` |
| **HTTP integration** | All 12 tests above | 06 | `crates/tuitbot-server/tests/assist_rag_tests.rs` |

## What's Covered

- **Full HTTP stack**: Request → auth middleware → handler → resolver (config load + DB query) → generator (mock LLM) → response serialization.
- **Both vault data paths**: Cold-start seeds (`"Relevant ideas"` header) and winning ancestors (`"Winning patterns"` header).
- **Fail-open behavior**: Generation succeeds when vault data is absent (empty DB, no seeds, no ancestors).
- **No-LLM short-circuit**: 400 error returned before resolver runs when no generator is configured.
- **Dual-context independence**: Both user-supplied tone cue and automatic vault context appear in the prompt without interference.
- **API contract stability**: Response shapes (`{ content }`, `{ tweets, topic }`) are unchanged.

## What's Not Covered (and Why)

| Gap | Reason |
|-----|--------|
| `assist_reply` handler | Intentionally excluded from composer RAG — uses different generation path |
| Real LLM integration | Would require API keys; mock provider validates prompt assembly instead |
| Frontend E2E | Backend-only feature; frontend sends unchanged payloads |
| Concurrent vault writes | Single-user desktop app; concurrency not a concern |
| Config hot-reload | Resolver re-reads config on each call; no caching to test |
