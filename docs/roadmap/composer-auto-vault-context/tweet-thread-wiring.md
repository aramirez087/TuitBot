# Tweet & Thread Assist — Vault Context Wiring

Documents how `/api/assist/tweet` and `/api/assist/thread` now automatically inject vault-derived RAG context into generation prompts, matching the pattern used by autopilot drafting.

## Wiring Pattern

Both handlers follow the same two-step addition after the existing `get_generator()` call:

```rust
let gen = get_generator(&state, &ctx.account_id).await?;
let rag_context = resolve_composer_rag_context(&state, &ctx.account_id).await;

let output = gen
    .generate_X_with_context(&body.topic, None, rag_context.as_deref())
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;
```

### Call Ordering

1. **`get_generator()`** — validates LLM is configured; fails fast with `ApiError::BadRequest` if not.
2. **`resolve_composer_rag_context()`** — loads config from disk, extracts keywords, queries vault. Returns `Option<String>`.
3. **`generate_X_with_context()`** — passes `rag_context.as_deref()` (`Option<&str>`) to the generator. `None` falls back to base generation (no RAG).

The generator check runs before the resolver because it's cheaper (in-memory map lookup vs. disk I/O + DB query) and provides better UX on failure — if the LLM isn't configured, the user gets an immediate error without waiting for a pointless RAG resolution.

## Handler-Specific Notes

### `assist_tweet`

```rust
gen.generate_tweet_with_context(&body.topic, None, rag_context.as_deref())
```

- **`format: None`** — the composer UI doesn't expose tweet format selection. `None` preserves the same behavior as the previous `generate_tweet()` call, which delegates to `generate_tweet_inner(topic, None, None)`.

### `assist_thread`

```rust
gen.generate_thread_with_context(&body.topic, None, rag_context.as_deref())
```

- **`structure: None`** — same rationale. The composer UI doesn't expose thread structure selection. `None` preserves existing behavior.

## Response Shape Compatibility

No request or response structs changed:

| Endpoint | Request | Response |
|----------|---------|----------|
| `POST /api/assist/tweet` | `{ topic: String }` | `{ content: String, topic: String }` |
| `POST /api/assist/thread` | `{ topic: String }` | `{ tweets: Vec<String>, topic: String }` |

The frontend client (`dashboard/src/lib/api.ts`) requires no changes.

## Comparison with Autopilot Draft Wiring

The autopilot draft workflow in `workflow/draft.rs` uses the same RAG primitives:

| Aspect | Autopilot (`draft.rs`) | Composer (`assist.rs`) |
|--------|----------------------|----------------------|
| Keywords source | `config.business.draft_context_keywords()` | Same |
| Context builder | `winning_dna::build_draft_context()` | Same (via resolver) |
| Constants | `MAX_ANCESTORS=5`, `RECENCY_HALF_LIFE_DAYS=14.0` | Same |
| Fail-open | Logs warning, proceeds without context | Same |
| Generator method | `generate_tweet_with_context()` / `generate_thread_with_context()` | Same |

The key difference is encapsulation: the autopilot workflow calls `build_draft_context()` directly because it already has the config in scope, while the composer handlers delegate to `resolve_composer_rag_context()` which handles config loading and fail-open logic as a self-contained unit.

## Dead-Code Suppression Removal

Session 03 added `#[allow(dead_code)]` on `resolve_composer_rag_context()` and `#[allow(unused_imports)]` on the `winning_dna` import because the resolver had no live callers. With both handlers now calling the resolver, these suppressions are removed. Clippy passes clean.
