# Automatic Vault Context in Composer Generation — Charter

## Problem Statement

The autopilot draft workflow (`workflow/draft.rs:51-68`) builds full RAG context from winning ancestors and content seeds before generating replies. This context — drawn from the user's historically best-performing tweets and ingested vault notes — meaningfully improves generation quality by grounding output in proven patterns.

The composer assist endpoints (`routes/assist.rs:52-168`) bypass this entirely. All four generation handlers call `ContentGenerator` methods without any RAG context:

| Endpoint | Handler | Generator Method | RAG? |
|----------|---------|-----------------|------|
| `POST /api/assist/tweet` | `assist_tweet` (line 52) | `generate_tweet(&topic)` | No |
| `POST /api/assist/reply` | `assist_reply` (line 87) | `generate_reply(...)` | No |
| `POST /api/assist/thread` | `assist_thread` (line 119) | `generate_thread(&topic)` | No |
| `POST /api/assist/improve` | `assist_improve` (line 153) | `improve_draft(draft, context)` | No |

This means a user composing content manually gets lower-quality generation than the autopilot loop — exactly backwards from what the product should deliver.

## Goal

Automatically and transparently inject vault context (winning ancestors + content seeds) into all composer generation flows, without changing any request/response API shapes.

## Architecture

### Target Flow

```
Composer assist request
  -> resolve_composer_rag_context(state, account_id)
       -> Load Config from state.config_path
       -> Extract merged keywords from config.business
       -> winning_dna::build_draft_context(db, keywords, MAX_ANCESTORS, RECENCY_HALF_LIFE_DAYS)
       -> Return Option<String> (prompt_block or None on error)
  -> Pass rag_context to _with_context generator variant
  -> Return response (unchanged shape)
```

### Shared Helper: `resolve_composer_rag_context()`

A single async helper in `routes/assist.rs` that all assist handlers call before generation.

**Signature:**
```rust
async fn resolve_composer_rag_context(
    state: &AppState,
    _account_id: &str,
) -> Option<String>
```

**Behavior:**
1. Load `Config` from `state.config_path` (same pattern as `get_mode` at `assist.rs:252`).
2. Assemble merged keywords from `config.business` (product + competitor + industry topics).
3. Call `winning_dna::build_draft_context(db, keywords, MAX_ANCESTORS, RECENCY_HALF_LIFE_DAYS)`.
4. Extract `DraftContext.prompt_block`, return `Some(block)` if non-empty, `None` otherwise.
5. Catch all errors, log at `warn` level, return `None`.

**Why this placement:**
- `ContentGenerator` is a Toolkit-layer struct: stateless, DB-unaware, holds only `Box<dyn LlmProvider>` + `BusinessProfile`. Adding DB access would violate the three-layer architecture (Toolkit <- Workflow <- Autopilot).
- The server layer already has `state.db` and `state.config_path`. The helper mirrors how `draft.rs` assembles context at the Workflow layer.
- A single helper eliminates duplication across all four assist handlers.

### Keyword Sourcing

Keywords are assembled identically to `draft.rs:52-54`:

```rust
let mut keywords: Vec<String> = config.business.product_keywords.clone();
keywords.extend(config.business.competitor_keywords.clone());
keywords.extend(config.business.effective_industry_topics().to_vec());
```

To eliminate this duplication, Session 02 will add a `BusinessProfile::draft_context_keywords()` method that both `draft.rs` and the resolver call.

### Generator Method Changes

Three of the four generation types already have `_with_context` variants that accept `Option<&str>` for RAG context:

| Method | `_with_context` variant exists? |
|--------|-------------------------------|
| `generate_tweet` | Yes: `generate_tweet_with_context(topic, format, rag_context)` |
| `generate_reply` | Yes: `generate_reply_with_context(...)` |
| `generate_thread` | Yes: `generate_thread_with_context(topic, structure, rag_context)` |
| `improve_draft` | **No** — only `improve_draft(draft, tone_cue)` |

Session 02 must add `improve_draft_with_context(draft, tone_cue, rag_context)` to `ContentGenerator`.

### Dual-Context Design for `improve_draft`

The improve endpoint accepts an optional `context` field that serves as a tone/style directive. Automatic vault context occupies a separate prompt slot:

```
System prompt:
  You are {product_name}'s social media voice. {description}.
  {voice_section}
  {persona_section}
  {rag_section}           <-- Automatic vault context (new)

Task section:
  Rewrite and improve the draft tweet below.
  Tone/style directive: {user_context}    <-- User-supplied quick cue (existing)
```

RAG context is injected via the system prompt's `rag_section` (same as tweet/thread). User-supplied `context` remains a task-level directive. They occupy separate prompt slots and cannot conflict.

## Design Decisions

### 1. Fail-Open Behavior

The resolver catches all errors from `build_draft_context()`, logs at `warn` level, and returns `None`. Handlers check for `Some(prompt_block)` and pass it to `_with_context` variants. On `None`, they fall through to the plain variants (or pass `None` to the `_with_context` call).

This matches `draft.rs:56-63` which uses `.ok()` to silently discard RAG errors.

**Rationale:** Generation must never fail due to RAG unavailability. New accounts with no vault data or posting history should get unchanged behavior.

### 2. Config Re-Read Per Call

The resolver loads `Config` from the filesystem on each assist call (same pattern as `get_mode` at `assist.rs:252`).

**Tradeoff accepted:** File reads are sub-millisecond. Caching would add invalidation complexity (config can change via `settings` endpoints or TOML editing). The simple approach is correct and fast enough.

### 3. Request/Response Shape Preservation

Zero changes to request or response JSON schemas. All four assist endpoints keep their current `Deserialize`/`Serialize` types unchanged. The RAG context is resolved server-side and injected into the generator — completely transparent to clients.

### 4. RAG Latency Budget

`build_draft_context()` executes one SQL query against the local SQLite WAL database. Typical latency is <10ms. For composer assist calls (interactive, user-initiated), this is well within acceptable latency budgets.

### 5. Prompt Length Budget

RAG blocks are capped at `RAG_MAX_CHARS = 2000` characters (~500 tokens). Combined with existing system prompt sections (voice, persona, audience, content style), the total stays well within LLM context limits for all supported providers.

## Non-Goals

- **No new UI elements, toggles, or note pickers.** The vault context is injected automatically and transparently.
- **No manual vault-note selection in composer.** Users do not choose which notes to include.
- **No new API endpoints.** This augments existing endpoints only.
- **No request/response schema changes.** Client code requires zero updates.
- **No changes to `/api/assist/reply`.** Reply assist is used for discovery feed replies, which have a different context model. Could be enhanced later but is out of scope.
- **No multi-account redesign.** The resolver works with the per-account `ContentGenerator` already in `AppState.content_generators`.
- **No logging of note content.** Beyond what existing prompt assembly already permits (debug-level `has_rag_context` booleans).
- **No feature flag or toggle.** The feature is always-on once shipped. Fail-open behavior means it's a no-op when data is unavailable.

## Key File References

| File | Path | Role |
|------|------|------|
| Assist routes | `crates/tuitbot-server/src/routes/assist.rs` | 4 composer endpoints, resolver target |
| Winning DNA | `crates/tuitbot-core/src/context/winning_dna.rs` | RAG retrieval + prompt formatting |
| Content generator | `crates/tuitbot-core/src/content/generator/mod.rs` | LLM prompt assembly, `_with_context` variants |
| Draft workflow | `crates/tuitbot-core/src/workflow/draft.rs` | Autopilot draft step, reference RAG wiring |
| Config types | `crates/tuitbot-core/src/config/types.rs` | `BusinessProfile`, keyword accessors |
| App state | `crates/tuitbot-server/src/state.rs` | `AppState` with `db`, `config_path`, `content_generators` |
| Composer docs | `docs/composer-mode.md` | Composer documentation (update in Session 07) |
