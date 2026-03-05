# Server RAG Contract — Composer Context Resolver

Documents the `resolve_composer_rag_context()` helper added in Session 03.

## Signature

```rust
async fn resolve_composer_rag_context(
    state: &AppState,
    _account_id: &str,
) -> Option<String>
```

**Location:** `crates/tuitbot-server/src/routes/assist.rs` (private helper, co-located with handlers)

**Parameters:**
- `state` — Server application state providing DB pool and config path.
- `_account_id` — Reserved for multi-account config loading (unused; config is loaded from `state.config_path`).

**Returns:** `Option<String>` — `Some(prompt_block)` when vault data yields non-empty context, `None` otherwise.

## Workflow

1. **Load config** from `state.config_path` (same disk-read pattern as `get_mode`).
2. **Extract keywords** via `config.business.draft_context_keywords()` (Session 02 API).
3. **Early return** `None` if keywords are empty (no topics for retrieval).
4. **Query vault** via `winning_dna::build_draft_context(db, keywords, MAX_ANCESTORS, RECENCY_HALF_LIFE_DAYS)`.
5. **Return** `Some(prompt_block)` if non-empty, `None` if empty (cold-start with no seeds).

## Constants

| Constant | Value | Source |
|----------|-------|--------|
| `MAX_ANCESTORS` | 5 | `winning_dna::MAX_ANCESTORS` |
| `RECENCY_HALF_LIFE_DAYS` | 14.0 | `winning_dna::RECENCY_HALF_LIFE_DAYS` |

No new magic numbers introduced. Both constants are shared with the autopilot draft workflow (`workflow/draft.rs`).

## Fail-Open Contract

The resolver never returns `Err`. All failure modes produce `None` with a `tracing::warn!` log:

| Failure Mode | Log Message | Return |
|-------------|-------------|--------|
| Config file missing or malformed | `composer RAG: failed to load config: {err}` | `None` |
| DB query error | `composer RAG: failed to build draft context: {err}` | `None` |
| Empty keywords (no product/competitor/industry keywords configured) | *(no log — expected cold-start)* | `None` |
| Empty prompt block (no ancestors and no seeds in DB) | *(no log — expected cold-start)* | `None` |

**Rationale:** Composer generation must succeed without vault context. RAG augmentation is additive — its absence degrades quality but never blocks the user.

## Logging Decisions

- **Level:** `warn` for errors (config/DB failures are operational issues worth alerting on).
- **Content:** Error messages only. No note content, no keyword lists, no prompt block text logged. This aligns with the charter's privacy constraint that logging stays free of vault note content.
- **Silent returns:** Empty keywords and empty prompt blocks return `None` without logging because these are expected states (new account, empty DB) rather than operational failures.

## Integration Points (for Session 04)

The resolver is ready to be called from three handlers:

| Handler | How to integrate |
|---------|-----------------|
| `assist_tweet` | Call resolver, pass result to `generate_tweet_with_context()` (to be added) |
| `assist_thread` | Call resolver, pass result to `generate_thread_with_context()` (to be added) |
| `assist_improve` | Call resolver, pass result to `improve_draft_with_context()` (already exists) |

Pattern for each handler:
```rust
let rag_context = resolve_composer_rag_context(&state, &ctx.account_id).await;
let output = gen.generate_tweet_with_context(&body.topic, rag_context.as_deref()).await...;
```

The `_with_context` variants accept `Option<&str>` — `None` falls back to base generation (no RAG), matching the existing `improve_draft_with_context` pattern from Session 02.

## Test Coverage

| Test | What it verifies |
|------|-----------------|
| `resolve_rag_returns_none_when_config_missing` | Config load error → fail-open `None` |
| `resolve_rag_returns_none_when_db_empty` | Valid config, empty DB → `None` (no ancestors/seeds) |
| `resolve_rag_returns_none_when_no_keywords` | Empty business profile → early return `None` |
