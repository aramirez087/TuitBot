# Improve Flow — Vault Context Wiring

Documents how `/api/assist/improve` now automatically injects vault-derived RAG context into draft improvement prompts, completing the trio of composer assist handlers (tweet, thread, improve).

## Dual-Context Model

The improve handler has two independent context channels:

| Parameter | Source | Semantics | Prompt placement |
|-----------|--------|-----------|-----------------|
| `body.context` (tone cue) | Frontend — user-supplied | Binding directive: "MUST follow" instruction for tone, style, or task (e.g., "more casual", "expand these rough notes...") | After the task description as `Tone/style directive (MUST follow): {cue}` |
| `rag_context` | Server-side — vault resolver | Background knowledge: winning tweet patterns, engagement data | Before the task description as a knowledge context block via `format_rag_section()` |

Both are `Option<&str>`. All four combinations (both present, either absent, both absent) are valid and tested at the generator level.

## Wiring Pattern

Identical to the tweet/thread pattern documented in `tweet-thread-wiring.md`:

```rust
let gen = get_generator(&state, &ctx.account_id).await?;
let rag_context = resolve_composer_rag_context(&state, &ctx.account_id).await;

let output = gen
    .improve_draft_with_context(&body.draft, body.context.as_deref(), rag_context.as_deref())
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;
```

### Call Ordering

1. **`get_generator()`** — validates LLM is configured; fails fast with `ApiError::BadRequest`.
2. **`resolve_composer_rag_context()`** — loads config, extracts keywords, queries vault. Returns `Option<String>`. Fail-open on any error.
3. **`improve_draft_with_context()`** — passes both `body.context.as_deref()` (tone cue) and `rag_context.as_deref()` (vault data).

## Frontend Call Sites

All five call sites that invoke `api.assist.improve()` — none required payload changes:

| Call site | File | `body.draft` | `body.context` | Effect of new RAG |
|-----------|------|-------------|----------------|-------------------|
| Inline improve (`Cmd+J`) — tweet mode | `ComposeWorkspace.svelte` | Selected text or full tweet | Voice cue or `undefined` | Draft improved with persona + vault knowledge |
| Inline improve (`Cmd+J`) — thread mode | `ThreadFlowLane.svelte` | Selected text or card text | Voice cue or `undefined` | Same |
| From-notes | `ComposeWorkspace.svelte` | Raw notes input | `"${voiceCue}. Expand these rough notes..."` or `"Expand these rough notes..."` | Notes expanded with task instruction + vault knowledge |
| AI Improve button | `ComposeWorkspace.svelte` | Full tweet text | Voice cue or `undefined` | Same as inline improve on full text |
| Drafts page improve | `drafts/+page.svelte` | Draft content | `undefined` | Draft improved with vault knowledge (no voice cue) |

### Request/Response Shape — Unchanged

| Field | Type | Change |
|-------|------|--------|
| `POST /api/assist/improve` request | `{ draft: String, context?: String }` | No change |
| `POST /api/assist/improve` response | `{ content: String }` | No change |

## Prompt Assembly

In `improve_draft_inner`, the system prompt structure after the change:

```
You are {product_name}'s social media voice. {product_description}.
{voice_section}         ← brand_voice from config
{persona_section}       ← opinions, experiences, pillars
{rag_section}           ← vault-derived winning patterns (or empty string)

Task: Rewrite and improve the draft tweet below...
{tone_instruction}      ← user-supplied context as "Tone/style directive (MUST follow): {cue}"

Rules:
- Maximum 280 characters.
- Do not use hashtags.
- Output only the improved tweet text, nothing else.
```

The `rag_section` provides background knowledge (examples of winning tweets). The `tone_instruction` is a binding directive. They occupy separate sections and don't conflict.

## Behavior Matrix

| `body.context` | `rag_context` | Result |
|----------------|---------------|--------|
| Present | Present | Both injected — RAG as knowledge block, context as tone directive |
| Present | `None` | Tone directive only — identical to pre-change behavior |
| `None` | Present | RAG knowledge only — useful for drafts page improve |
| `None` | `None` | Base improve — identical to pre-change behavior |

## Handler Symmetry — All Three Wired

| Handler | Generator method | Context channels | Wired in |
|---------|-----------------|------------------|----------|
| `assist_tweet` | `generate_tweet_with_context(topic, None, rag)` | RAG only | Session 04 |
| `assist_thread` | `generate_thread_with_context(topic, None, rag)` | RAG only | Session 04 |
| `assist_improve` | `improve_draft_with_context(draft, tone_cue, rag)` | Tone cue + RAG | Session 05 |

The improve handler is the only one with dual context channels because it's the only handler that accepts user-supplied directives alongside automatic vault context.

## Fail-Open Behavior

When vault data is unavailable (no config, no keywords, empty DB), `rag_context` resolves to `None`. The generator's `format_rag_section(None)` returns an empty string. The prompt is identical to the pre-change version. No degradation in any call site.
