# Composer Auto-Vault Context — Release Notes

## Summary

Tuitbot's AI Assist endpoints (`/api/assist/tweet`, `/api/assist/thread`, `/api/assist/improve`) now automatically enrich LLM prompts with context from the user's vault. When generating or improving content, the system retrieves winning historical tweets and relevant content seeds, producing output that is grounded in the user's own voice and proven engagement patterns — with zero additional user interaction.

## Scope

### Affected endpoints

| Endpoint | Vault context behavior |
|----------|----------------------|
| `POST /api/assist/tweet` | Automatic — winning patterns and relevant ideas injected into system prompt |
| `POST /api/assist/thread` | Automatic — same as tweet |
| `POST /api/assist/improve` | Automatic — vault context plus user-supplied tone cue (if provided) coexist in the prompt |

### Excluded endpoints

| Endpoint | Reason |
|----------|--------|
| `POST /api/assist/reply` | Uses a different generation path (`generate_reply`) that is optimized for conversational replies to specific tweets. Vault context is not applicable here. |
| `GET /api/assist/topics` | Read-only analytics query — no LLM generation involved. |
| `GET /api/assist/optimal-times` | Read-only analytics query — no LLM generation involved. |

## User-Visible Outcome

- **Higher quality output.** Generated tweets and threads reflect the user's historically best-performing content and vault notes, producing more relevant and on-brand results.
- **No workflow changes.** The composer UI, quick-cue input, "From Notes" panel, and all keyboard shortcuts work exactly as before. The enrichment happens transparently on the backend.
- **No new configuration.** If the user has already configured `product_keywords`, `competitor_keywords`, or `industry_topics` in their business profile, vault context activates automatically.

## How It Works (Technical)

1. **Resolver** (`resolve_composer_rag_context` in `assist.rs`): Loads keyword config, queries the database, returns an optional prompt block.
2. **Winning DNA module** (`context/winning_dna.rs`): Retrieves and scores historical tweet performance (ancestors) and content source seeds. Formats them into a prompt block capped at 2000 characters (`RAG_MAX_CHARS`).
3. **Generator** (`_with_context` methods in `content/generator.rs`): Accepts the optional RAG context string and injects it into the LLM system prompt alongside voice, persona, and content style instructions.
4. **Server handlers**: Each handler calls the resolver, then passes the result to the generator. The response shape is unchanged.

The resolver → winning_dna → generator pipeline is reusable: a single `resolve_composer_rag_context()` helper is called by all three handlers, avoiding code duplication.

## Fallback Behavior

The feature is designed to fail open. Generation always succeeds without vault data:

| Condition | Behavior |
|-----------|----------|
| No keywords configured in `config.toml` | Resolver returns `None`; generation proceeds without vault context |
| Empty database (fresh account) | Resolver returns `None`; generation proceeds without vault context |
| Config file unreadable | Resolver logs a warning and returns `None`; generation proceeds |
| Database query error | Resolver logs a warning and returns `None`; generation proceeds |
| No matching seeds or ancestors | Resolver returns `None`; generation proceeds without vault context |

In all cases, the response status code and shape are identical to a call without vault data.

## API Compatibility

- **Request shapes unchanged.** `AssistTweetRequest`, `AssistThreadRequest`, and `AssistImproveRequest` have no new fields.
- **Response shapes unchanged.** `AssistTweetResponse`, `AssistThreadResponse`, and `AssistImproveResponse` have no new fields.
- **No new endpoints.** No new routes were added.
- **No frontend changes.** The dashboard sends the same payloads as before; enrichment is entirely server-side.

## Known Limitations

1. **`assist_reply` not included.** This is intentional — reply generation uses `generate_reply()`, which is optimized for conversational responses to specific tweets and does not benefit from the same vault context pipeline.

2. **No UI toggle to disable vault context.** The feature is always active when keywords are configured and vault data exists. A future session could add a config flag or UI toggle if users request opt-out.

3. **No caching of resolved context.** The resolver re-reads `config.toml` and queries the database on every assist call. This is acceptable for interactive use (human typing speed) but would need caching if used in high-throughput batch scenarios.

4. **Ancestors test seeder uses raw SQL.** The `original_tweets` table has no public `insert_original_tweet()` helper. The test seeder in `assist_rag_tests.rs` uses a raw SQL INSERT, which is fragile if the schema changes.

5. **RAG block size is fixed at 2000 characters.** The cap is not user-configurable. This is sufficient for ~500 tokens of context but may need tuning for models with larger context windows.

## Session History

| Session | Deliverable | Status |
|---------|-------------|--------|
| 01 | Winning DNA module — retrieval, scoring, formatting (`context/winning_dna.rs`) | Done |
| 02 | Generator `_with_context` methods (`content/generator.rs`) | Done |
| 03 | Server-side resolver (`resolve_composer_rag_context` in `assist.rs`) | Done |
| 04 | Wire tweet and thread handlers to resolver + generator | Done |
| 05 | Wire improve handler with dual-context (vault + tone cue) | Done |
| 06 | HTTP integration tests (12 tests) + test matrix documentation | Done |
| 07 | Product docs, release notes, and final handoff | Done |
